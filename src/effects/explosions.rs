use std::sync::OnceLock;
use bevy::prelude::*;
use crate::common::Z_GROUND_EFFECT;
use crate::effects::common::AnimationController;
use crate::grids::common::{GridCoords, GridImprint};

pub static EXPLOSION_ATLAS: OnceLock<ExplosionAtlas> = OnceLock::new();
// TODO: Get rid of OnceLock

#[derive(Debug)]
pub struct ExplosionAtlas {
    pub atlas_handle: Handle<TextureAtlasLayout>,
    pub texture_handle: Handle<Image>,
}

#[derive(Component)]
pub struct MarkerExplosion;

#[derive(Bundle)]
pub struct BuilderExplosion {
    pub sprite_bundle: SpriteBundle,
    pub texture_atlas: TextureAtlas,
    pub animation_controller: AnimationController,
    pub marker_explosion: MarkerExplosion,
}

impl BuilderExplosion {
    pub fn new(grid_position: GridCoords) -> Self {
        let explosion_atlas = EXPLOSION_ATLAS.get().unwrap();
        Self {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: grid_position.to_world_position_centered(GridImprint::Rectangle { width: 1, height: 1 }).extend(Z_GROUND_EFFECT),
                    ..Default::default()
                },
                texture: explosion_atlas.texture_handle.clone(),
                ..default()
            },
            texture_atlas: TextureAtlas {
                layout: explosion_atlas.atlas_handle.clone(),
                index: 0,
                ..Default::default()
            },
            animation_controller: AnimationController::new(0, 3, 0.1, false),
            marker_explosion: MarkerExplosion,
        }
    }
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        commands.spawn(self).id()
    }
}

pub fn load_assets_system(
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture_handle = asset_server.load("effects/explosion.png");
    let texture_atlas = TextureAtlasLayout::from_grid(
        UVec2::new(16, 18),
        4,
        1,
        None,
        None,
    );
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    let expolsion_atlas = ExplosionAtlas {
        atlas_handle: texture_atlas_handle,
        texture_handle,
    };
    EXPLOSION_ATLAS.set(expolsion_atlas).unwrap();
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