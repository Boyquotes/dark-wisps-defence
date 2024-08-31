use crate::prelude::*;
use crate::buildings::common::BuildingType;
use crate::buildings::common_components::{Building, TechnicalState};
use crate::grids::energy_supply::EnergySupplyGrid;
use crate::inventory::resources::DarkOreStock;

pub struct MiningComplexPlugin;
impl Plugin for MiningComplexPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<BuilderMiningComplex>()
            .add_systems(PostUpdate, (
                BuilderMiningComplex::spawn_system,
            )).add_systems(Update, (
                mine_ore_system,
            ));
    }
}

pub const MINING_COMPLEX_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: 3, height: 3 };
pub const MINING_COMPLEX_BASE_IMAGE: &str = "buildings/mining_complex.png";


#[derive(Component)]
pub struct MarkerMiningComplex;

#[derive(Component)]
pub struct MiningComplexDeliveryTimer(pub Timer);

#[derive(Event)]
pub struct BuilderMiningComplex {
    pub entity: LazyEntity,
    pub grid_position: GridCoords,
}
impl BuilderMiningComplex {
    pub fn new(grid_position: GridCoords) -> Self {
        Self { entity: LazyEntity::default(), grid_position }
    }
    pub fn spawn_system(
        mut commands: Commands,
        mut events: EventReader<BuilderMiningComplex>,
        asset_server: Res<AssetServer>,
        energy_supply_grid: Res<EnergySupplyGrid>,
    ) {
        for &BuilderMiningComplex{ mut entity, grid_position } in events.read() {
            let entity = entity.get(&mut commands);
            commands.entity(entity).insert((
                get_mining_complex_sprite_bundle(&asset_server, grid_position),
                MarkerMiningComplex,
                grid_position,
                Health(10000),
                Building::from(BuildingType::MiningComplex),
                TechnicalState{ has_energy_supply: energy_supply_grid.is_imprint_suppliable(grid_position, MINING_COMPLEX_GRID_IMPRINT) },
                MiningComplexDeliveryTimer(Timer::from_seconds(1.0, TimerMode::Repeating)),
            ));
        }
    }
}
impl Command for BuilderMiningComplex {
    fn apply(self, world: &mut World) {
        world.send_event(self);
    }
}

pub fn get_mining_complex_sprite_bundle(asset_server: &AssetServer, coords: GridCoords) -> SpriteBundle {
    SpriteBundle {
        sprite: Sprite {
            custom_size: Some(MINING_COMPLEX_GRID_IMPRINT.world_size()),
            ..Default::default()
        },
        texture: asset_server.load(MINING_COMPLEX_BASE_IMAGE),
        transform: Transform::from_translation(coords.to_world_position_centered(MINING_COMPLEX_GRID_IMPRINT).extend(Z_BUILDING)),
        ..Default::default()
    }
}

pub fn mine_ore_system(
    mut dark_ore_stock: ResMut<DarkOreStock>,
    mut mining_complexes: Query<(&mut MiningComplexDeliveryTimer, &TechnicalState), With<MarkerMiningComplex>>,
    time: Res<Time>,
) {
    for (mut timer, technical_state) in mining_complexes.iter_mut() {
        if !technical_state.is_operational() { continue; }
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            dark_ore_stock.add(10);
        }
    }
}
