use crate::prelude::*;
use serde::{Deserialize, Serialize};
use crate::utils::id::Id;

use super::prelude::BuildingType;

pub type BuildingId = Id<BuildingType, Entity>;

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
pub enum TowerType {
    Blaster,
    Cannon,
    RocketLauncher,
    Emitter,
}

pub fn get_building_sprite_bundle(asset_server: &AssetServer, image_path: &'static str, coords: GridCoords, grid_imprint: GridImprint) -> SpriteBundle {
    SpriteBundle {
        sprite: Sprite {
            custom_size: Some(grid_imprint.world_size()),
            ..Default::default()
        },
        texture: asset_server.load(image_path),
        transform: Transform::from_translation(coords.to_world_position_centered(grid_imprint).extend(Z_BUILDING)),
        ..Default::default()
    }
}