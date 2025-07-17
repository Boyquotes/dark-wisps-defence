use crate::prelude::*;

use super::components::{Wisp, WispAttackRange, WispChargeAttack, WispElectricType, WispFireType, WispLightType, WispState, WispType, WispWaterType};
use super::materials::WispMaterial;

pub const WISP_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: 1, height: 1 };

#[derive(Component)]
pub struct BuilderWisp {
    wisp_type: WispType,
    grid_coords: GridCoords,
}

impl BuilderWisp {
    pub fn new(wisp_type: WispType, grid_coords: GridCoords) -> Self {
        Self { wisp_type, grid_coords }
    }

    pub fn on_add(
        trigger: Trigger<OnAdd, BuilderWisp>,
        mut commands: Commands,
        builders: Query<&BuilderWisp>,
    ) {
        let entity = trigger.target();
        let Ok(builder) = builders.get(entity) else { return; };
        
        let mut rng = nanorand::tls_rng();
        let mut entity_commands = commands.entity(entity);
        entity_commands
            .remove::<BuilderWisp>()
            .insert((
                builder.grid_coords,
                Health::new(10),
                Speed(30.),
                Transform {
                    translation: builder.grid_coords.to_world_position_centered(WISP_GRID_IMPRINT).extend(Z_WISP),
                    rotation: Quat::from_rotation_z(rng.generate::<f32>() * 2. * std::f32::consts::PI),
                    ..default()
                },
                Wisp,
                builder.wisp_type,
                WispState::default(),
                WispChargeAttack::default(),
                WispAttackRange(1),
                GridPath::default(),
            ));
        match builder.wisp_type {
            WispType::Fire => entity_commands.insert((WispFireType, EssencesContainer::from(EssenceContainer::new(EssenceType::Fire, 1)))),
            WispType::Water => entity_commands.insert((WispWaterType, EssencesContainer::from(EssenceContainer::new(EssenceType::Water, 1)))),
            WispType::Light => entity_commands.insert((WispLightType, EssencesContainer::from(EssenceContainer::new(EssenceType::Light, 1)))),
            WispType::Electric => entity_commands.insert((WispElectricType, EssencesContainer::from(EssenceContainer::new(EssenceType::Electric, 1)))),
        };
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
    let entity = trigger.target();
    if !wisps.contains(entity) { return; }
    // TODO: Do we need to remove the meash and material if the wisp is removed?
    let wisp_world_size = WISP_GRID_IMPRINT.world_size();
    let mesh = meshes.add(Rectangle::new(wisp_world_size.x, wisp_world_size.y));
    let material = materials.add(MaterialT::make(&asset_server));
    commands.entity(entity).insert((
        Mesh2d(mesh),
        MeshMaterial2d(material),
    ));
}