use bevy::prelude::*;
use vinox_common::world::chunks::positions::world_to_chunk;

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
    pub fn is_in_radius(&self, pos: IVec3, radius: i32) -> bool {
        self.chunk_pos.as_vec3().distance(pos.as_vec3()) <= radius as f32
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
