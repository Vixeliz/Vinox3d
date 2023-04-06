use std::io::Cursor;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use bevy::prelude::*;
use rusqlite::*;
use serde::{Deserialize, Serialize};
use vinox_common::{
    ecs::bundles::Inventory,
    world::chunks::{positions::ChunkPos, storage::RawChunk},
};
use zstd::stream::{copy_decode, copy_encode};

#[derive(Component, Default, Serialize, Deserialize, Debug, Clone)]
pub struct SavedPlayer {
    pub inventory: Inventory,
    pub position: [i32; 3],
}

#[derive(Resource, Deref, DerefMut, Default)]
pub struct ChunksToSave(pub Vec<(ChunkPos, RawChunk)>);

#[derive(Resource, Deref, DerefMut, Default)]
pub struct FirstSaves(pub Vec<(String, SavedPlayer, String)>);

#[derive(Resource, Deref, DerefMut, Default)]
pub struct PlayersToSave(pub Vec<(String, SavedPlayer)>);

#[derive(Resource, Serialize, Deserialize, Clone)]
pub struct WorldInfo {
    pub name: String,
    pub seed: u32,
    pub damage: bool,
}

#[derive(Resource)]
pub struct WorldDatabase {
    pub connection: Pool<SqliteConnectionManager>,
}

pub fn create_database(database: &Connection) {
    database
        .execute(
            " create table if not exists blocks (
            posx integer not null,
            posy integer not null,
            posz integer not null,
            data blob,
            PRIMARY KEY (posx, posy, posz)
        )",
            [],
        )
        .unwrap();
    database
        .execute(
            " create table if not exists players(
            name varchar(255) not null,
            password varchar(255) not null,
            data blob,
            PRIMARY KEY (name)
        )",
            [],
        )
        .unwrap();
}

pub fn save_chunks(chunks: &ChunksToSave, database: &Connection) {
    database.execute("BEGIN;", []).unwrap();
    for (chunk_pos, raw_chunk) in chunks.iter() {
        if let Ok(raw_chunk_bin) = bincode::serialize(raw_chunk) {
            let mut final_chunk = Cursor::new(raw_chunk_bin);
            let mut output = Cursor::new(Vec::new());
            copy_encode(&mut final_chunk, &mut output, 0).unwrap();
            database
                .execute(
                    "REPLACE INTO blocks (posx, posy, posz, data) values (?1, ?2, ?3, ?4)",
                    params![
                        &chunk_pos.x,
                        &chunk_pos.y,
                        &chunk_pos.z,
                        &output.get_ref().clone(),
                    ],
                )
                .unwrap();
        }
    }
    database.execute("COMMIT;", []).unwrap();
}

pub fn save_passwords(players_to_save: &FirstSaves, database: &Connection) {
    database.execute("BEGIN;", []).unwrap();
    for (user_name, player, password) in players_to_save.iter() {
        if let Ok(player_bin) = bincode::serialize(player) {
            database
                .execute(
                    "REPLACE INTO players (name, data, password) values (?1, ?2, ?3)",
                    params![&user_name, &player_bin.clone(), &password],
                )
                .unwrap();
        }
    }
    database.execute("COMMIT;", []).unwrap();
}

pub fn save_players(players_to_save: &PlayersToSave, database: &Connection) {
    database.execute("BEGIN;", []).unwrap();
    for (user_name, player) in players_to_save.iter() {
        if let Ok(player_bin) = bincode::serialize(player) {
            database
                .execute(
                    "UPDATE players SET data = ?2 WHERE name = ?1",
                    params![&user_name, &player_bin.clone(),],
                )
                .unwrap();
        }
    }
    database.execute("COMMIT;", []).unwrap();
}

pub fn load_player(name: String, database: &Connection) -> Option<(SavedPlayer, String)> {
    let stmt = database.prepare("SELECT name, data, password FROM players WHERE name=:name;");
    if let Ok(mut stmt) = stmt {
        let name_result: Result<Vec<u8>, _> =
            stmt.query_row(&[(":name", &name)], |row| Ok(row.get(1).unwrap()));
        let password_result: Result<String, _> =
            stmt.query_row(&[(":name", &name)], |row| Ok(row.get(2).unwrap()));
        let final_player = if let Ok(name_row) = name_result {
            Some(bincode::deserialize(&name_row).unwrap())
        } else {
            None
        };
        let password = if let Ok(password) = password_result {
            Some(password)
        } else {
            None
        };

        if let Some(final_player) = final_player {
            if let Some(password) = password {
                return Some((final_player, password));
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    None
}

pub fn load_chunk(chunk_pos: ChunkPos, database: &Connection) -> Option<RawChunk> {
    let stmt = database.prepare(
        "SELECT posx, posy, posz, data FROM blocks WHERE posx=:posx AND posy=:posy AND posz=:posz;",
    );
    if let Ok(mut stmt) = stmt {
        let chunk_result: Result<Vec<u8>, _> = stmt.query_row(
            &[
                (":posx", &chunk_pos.x),
                (":posy", &chunk_pos.y),
                (":posz", &chunk_pos.z),
            ],
            |row| Ok(row.get(3).unwrap()),
        );
        if let Ok(chunk_row) = chunk_result {
            let mut temp_output = Cursor::new(Vec::new());
            copy_decode(&chunk_row[..], &mut temp_output).unwrap();
            let final_chunk = bincode::deserialize(temp_output.get_ref()).unwrap();
            return Some(final_chunk);
        }
    }

    None
}
