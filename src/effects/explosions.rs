use std::sync::OnceLock;
use bevy::prelude::*;
use crate::common::Z_GROUND_EFFECT;
use crate::effects::common::{AnimationController};
use crate::grids::common::{GridCoords, GridImprint};

pub static EXPLOSION_ATLAS: OnceLock<Handle<TextureAtlas>> = OnceLock::new();

#[derive(Component)]
pub struct MarkerExplosion;

pub fn create_explosion(
    commands: &mut Commands,
    grid_position: GridCoords,
) -> Entity {
    let explosion_entity = commands.spawn((
        SpriteSheetBundle {
            texture_atlas: EXPLOSION_ATLAS.get().unwrap().clone(),
            sprite: TextureAtlasSprite::new(0),
            transform: Transform {
                translation: grid_position.to_world_position_centered(GridImprint::Rectangle { width: 1, height: 1 }).extend(Z_GROUND_EFFECT),
                ..Default::default()
            },
            ..Default::default()
        },
        AnimationController::new(0, 3, 0.1, false),
        MarkerExplosion,
    )).id();
    explosion_entity
}

pub fn load_assets_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("effects/explosion.png");
    let texture_atlas = TextureAtlas::from_grid(
        texture_handle,
        Vec2::new(16.0, 18.0),
        4,
        1,
        None,
        None,
    );
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    EXPLOSION_ATLAS.set(texture_atlas_handle).unwrap();
    create_explosion(&mut commands, GridCoords { x: 0, y: 0 });
}

pub fn remove_explosions_system(
    mut commands: Commands,
    explosions: Query<(Entity, &AnimationController), With<MarkerExplosion>>,
) {
    for (explosion_entity, animation_controller) in &explosions {
        if animation_controller.has_finished {
            commands.entity(explosion_entity).despawn();
        }
    }
}