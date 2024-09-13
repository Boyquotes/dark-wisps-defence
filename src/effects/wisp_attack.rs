use bevy::log::tracing_subscriber::fmt::time;
use bevy::transform::commands;

use crate::mouse::MouseInfo;
use crate::prelude::*;
use crate::effects::common::AnimationController;

pub struct WispAttackEffectPlugin;
impl Plugin for WispAttackEffectPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<BuilderWispAttackEffect>()
            .init_resource::<WispAttackEffectAtlas>()
            .add_systems(PostUpdate, (
                BuilderWispAttackEffect::spawn_system,
            ))
            .add_systems(Update, (
                remove_explosions_system,
                spawn_random_wisps_effect_system,
            ));
    }
}

#[derive(Resource)]
pub struct WispAttackEffectAtlas {
    pub atlas_handle: Handle<TextureAtlasLayout>,
    pub texture_handle: Handle<Image>,
}
impl FromWorld for WispAttackEffectAtlas {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture_handle = asset_server.load("effects/wisp_attack.png");

        let texture_atlas = TextureAtlasLayout::from_grid(
            UVec2::new(192, 192),
            5,
            2,
            None,
            None,
        );
        let mut texture_atlases = world.resource_mut::<Assets<TextureAtlasLayout>>();
        let texture_atlas_handle = texture_atlases.add(texture_atlas);
        WispAttackEffectAtlas {
            atlas_handle: texture_atlas_handle,
            texture_handle,
        }
    }
}

#[derive(Component)]
pub struct MarkerWispAttackEffect;

#[derive(Event)]
pub struct BuilderWispAttackEffect {
    pub world_position: Vec2,
}

impl BuilderWispAttackEffect {
    pub fn new(world_position: Vec2) -> Self {
        Self { world_position }
    }
    pub fn spawn_system(
        mut commands: Commands,
        mut events: EventReader<BuilderWispAttackEffect>,
        explosion_atlas: Res<WispAttackEffectAtlas>,
    ) {
        for &BuilderWispAttackEffect { world_position } in events.read() {
            commands.spawn((
                SpriteBundle {
                    transform: Transform {
                        translation: world_position.extend(Z_GROUND_EFFECT),
                        scale: Vec3::new(0.25, 0.25, 1.0),
                        ..Default::default()
                    },
                    texture: explosion_atlas.texture_handle.clone(),
                    ..default()
                },
                TextureAtlas {
                    layout: explosion_atlas.atlas_handle.clone(),
                    index: 0,
                    ..Default::default()
                },
                AnimationController::new(0, 9, 0.025, false),
                MarkerWispAttackEffect,
            ));
        }
    }
} 
impl Command for BuilderWispAttackEffect {
    fn apply(self, world: &mut World) {
        world.send_event(self);
    }
}

fn remove_explosions_system(
    mut commands: Commands,
    explosions: Query<(Entity, &AnimationController), With<MarkerWispAttackEffect>>,
) {
    for (explosion_entity, animation_controller) in &explosions {
        if animation_controller.has_finished {
            commands.entity(explosion_entity).despawn();
        }
    }
}

fn spawn_random_wisps_effect_system(
    mut commands: Commands,
    button_input: Res<ButtonInput<MouseButton>>,
    mouse_info: Res<MouseInfo>,
) {
    if button_input.just_released(MouseButton::Left){
        println!("Spawning");
        commands.add(BuilderWispAttackEffect::new(mouse_info.world_position));
    }
}