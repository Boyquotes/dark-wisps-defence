use bevy::sprite::Mesh2dHandle;

use crate::prelude::*;

use super::components::{Wisp, WispAttackRange, WispChargeAttack, WispElectricType, WispFireType, WispLightType, WispState, WispType, WispWaterType};
use super::materials::WispMaterial;

pub const WISP_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: 1, height: 1 };

#[derive(Event)]
pub struct BuilderWisp {
    pub entity: Entity,
    pub wisp_type: WispType,
    pub grid_coords: GridCoords,
}

impl BuilderWisp {
    pub fn new(entity: Entity, wisp_type: WispType, grid_coords: GridCoords) -> Self {
        Self { entity, wisp_type, grid_coords }
    }
    pub fn spawn_system(
        mut commands: Commands,
        mut events: EventReader<BuilderWisp>,
    ) {
        let mut rng = nanorand::tls_rng();
        for &BuilderWisp { entity, wisp_type, grid_coords } in events.read() {
            let mut commands = commands.entity(entity);
            commands.insert((
                grid_coords,
                Health::new(10),
                Speed(30.),
                SpatialBundle {
                    transform: Transform {
                        translation: grid_coords.to_world_position_centered(WISP_GRID_IMPRINT).extend(Z_WISP),
                        rotation: Quat::from_rotation_z(rng.generate::<f32>() * 2. * std::f32::consts::PI),
                        ..default()
                    },
                    ..default()
                },
                Wisp,
                wisp_type,
                WispState::default(),
                WispChargeAttack::default(),
                WispAttackRange(1),
                GridPath::default(),
            ));
            match wisp_type {
                WispType::Fire => commands.insert(WispFireType),
                WispType::Water => commands.insert(WispWaterType),
                WispType::Light => commands.insert(WispLightType),
                WispType::Electric => commands.insert(WispElectricType),
            };
        }
    }
}
impl Command for BuilderWisp {
    fn apply(self, world: &mut World) {
        world.send_event(self);
    }
}

pub fn on_wisp_spawn_attach_material<WispT: Component, MaterialT: Asset + WispMaterial>(
    trigger: Trigger<OnAdd, WispT>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MaterialT>>,
    wisps: Query<(), With<WispT>>,
) {
    let entity = trigger.entity();
    if !wisps.contains(entity) { return; }
    // TODO: Do we need to remove the meash and material if the wisp is removed?
    let wisp_world_size = WISP_GRID_IMPRINT.world_size();
    let mesh = meshes.add(Rectangle::new(wisp_world_size.x, wisp_world_size.y));
    let material = materials.add(MaterialT::make(&asset_server));
    commands.entity(entity).insert((
        Mesh2dHandle(mesh),
        material,
    ));
}