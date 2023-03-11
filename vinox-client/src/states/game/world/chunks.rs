use bevy::prelude::*;
use vinox_common::world::chunks::{
    ecs::ViewRadius, positions::world_to_chunk, storage::CHUNK_SIZE,
};

#[derive(Component)]
pub struct ControlledPlayer;

#[derive(Default, Resource)]
pub struct PlayerChunk {
    pub chunk_pos: IVec3,
}

#[derive(Default, Resource)]
pub struct PlayerBlock {
    pub pos: IVec3,
}

impl PlayerChunk {
    pub fn is_in_radius(&self, pos: IVec3, view_radius: &ViewRadius) -> bool {
        for x in -view_radius.horizontal..view_radius.horizontal {
            for z in -view_radius.horizontal..view_radius.horizontal {
                if x.pow(2) + z.pow(2) >= view_radius.horizontal.pow(2) {
                    continue;
                }
                let delta: IVec3 = pos - self.chunk_pos;
                return delta.x.pow(2) + delta.z.pow(2)
                    > view_radius.horizontal.pow(2) * (CHUNK_SIZE as i32).pow(2)
                    || delta.y.pow(2) > view_radius.vertical.pow(2) * (CHUNK_SIZE as i32).pow(2);
            }
        }
        false
    }
}

pub fn update_player_location(
    player_query: Query<&Transform, With<ControlledPlayer>>,
    mut player_chunk: ResMut<PlayerChunk>,
    mut player_block: ResMut<PlayerBlock>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        let new_chunk = world_to_chunk(player_transform.translation);
        if new_chunk != player_chunk.chunk_pos {
            player_chunk.chunk_pos = new_chunk;
        }
        if player_transform.translation.floor().as_ivec3() != player_block.pos {
            player_block.pos = player_transform.translation.floor().as_ivec3();
        }
    }
}
