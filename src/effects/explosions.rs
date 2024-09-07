use crate::prelude::*;
use crate::effects::common::AnimationController;

pub struct ExplosionPlugin;
impl Plugin for ExplosionPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<BuilderExplosion>()
            .init_resource::<ExplosionAtlas>()
            .add_systems(PostUpdate, (
                BuilderExplosion::spawn_system,
            ))
            .add_systems(Update, (
                remove_explosions_system,
            ));
    }
}

#[derive(Resource)]
pub struct ExplosionAtlas {
    pub atlas_handle: Handle<TextureAtlasLayout>,
    pub texture_handle: Handle<Image>,
}
impl FromWorld for ExplosionAtlas {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let texture_handle = asset_server.load("effects/explosion.png");
        
        let texture_atlas = TextureAtlasLayout::from_grid(
            UVec2::new(16, 18),
            4,
            1,
            None,
            None,
        );
        let mut texture_atlases = world.resource_mut::<Assets<TextureAtlasLayout>>();
        let texture_atlas_handle = texture_atlases.add(texture_atlas);
        ExplosionAtlas {
            atlas_handle: texture_atlas_handle,
            texture_handle,
        }
    }
}

#[derive(Component)]
pub struct MarkerExplosion;

#[derive(Event)]
pub struct BuilderExplosion {
    pub entity: LazyEntity,
    pub grid_position: GridCoords,
}

impl BuilderExplosion {
    pub fn new(grid_position: GridCoords) -> Self {
        Self { entity: LazyEntity::default(), grid_position }
    }
    pub fn spawn_system(
        mut commands: Commands,
        mut events: EventReader<BuilderExplosion>,
        explosion_atlas: Res<ExplosionAtlas>,
    ) {
        for &BuilderExplosion { mut entity, grid_position } in events.read() {
            let entity = entity.get(&mut commands);
            commands.entity(entity).insert((
                SpriteBundle {
                    transform: Transform {
                        translation: grid_position.to_world_position_centered(GridImprint::Rectangle { width: 1, height: 1 }).extend(Z_GROUND_EFFECT),
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
                AnimationController::new(0, 3, 0.1, false),
                MarkerExplosion,
            ));
        }
    }
} 
impl Command for BuilderExplosion {
    fn apply(self, world: &mut World) {
        world.send_event(self);
    }
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