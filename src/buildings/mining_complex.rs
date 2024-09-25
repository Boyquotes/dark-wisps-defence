use crate::grids::obstacles::{self, Field, ObstacleGrid};
use crate::map_objects::dark_ore::DarkOre;
use crate::prelude::*;
use crate::grids::energy_supply::EnergySupplyGrid;

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

pub const MINING_COMPLEX_BASE_IMAGE: &str = "buildings/mining_complex.png";


#[derive(Component)]
pub struct MiningComplex {
    ore_entities_in_range: Vec<Entity>,
}
#[derive(Component)]
pub struct MiningComplexDeliveryTimer(pub Timer);
#[derive(Component)]
pub struct MiningRange(GridImprint);

#[derive(Event)]
pub struct BuilderMiningComplex {
    pub entity: Entity,
    pub grid_position: GridCoords,
}
impl BuilderMiningComplex {
    pub fn new(entity: Entity, grid_position: GridCoords) -> Self {
        Self { entity, grid_position }
    }
    pub fn spawn_system(
        mut commands: Commands,
        obstacle_grid: Res<ObstacleGrid>,
        mut events: EventReader<BuilderMiningComplex>,
        asset_server: Res<AssetServer>,
        almanach: Res<Almanach>,
        energy_supply_grid: Res<EnergySupplyGrid>,
    ) {
        for &BuilderMiningComplex{ entity, grid_position } in events.read() {
            let grid_imprint = almanach.get_building_grid_imprint(BuildingType::MiningComplex);
            let ore_entities_in_range = obstacle_grid.imprint_query_element(grid_position, grid_imprint, query_dark_ore_helper);
            commands.entity(entity).insert((
                get_mining_complex_sprite_bundle(&asset_server, grid_position, grid_imprint),
                TechnicalState{ 
                    has_energy_supply: energy_supply_grid.is_imprint_suppliable(grid_position, grid_imprint),
                    has_ore_fields: Some(!ore_entities_in_range.is_empty()),
                },
                MiningComplex { ore_entities_in_range },
                grid_position,
                Health(100),
                Building,
                BuildingType::MiningComplex,
                grid_imprint,
                MiningRange(grid_imprint),
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

// Helper to execute on every obstacle grid field to gather the dark_ore entities
fn query_dark_ore_helper(field: &Field) -> Option<Entity> {
    if let obstacles::Field::Building(_, BuildingType::MiningComplex, obstacles::BelowField::DarkOre(dark_ore_entity)) = field { Some(*dark_ore_entity) } else { None }
}

pub fn get_mining_complex_sprite_bundle(asset_server: &AssetServer, coords: GridCoords, grid_imprint: GridImprint) -> SpriteBundle {
    SpriteBundle {
        sprite: Sprite {
            custom_size: Some(grid_imprint.world_size()),
            ..Default::default()
        },
        texture: asset_server.load(MINING_COMPLEX_BASE_IMAGE),
        transform: Transform::from_translation(coords.to_world_position_centered(grid_imprint).extend(Z_BUILDING)),
        ..Default::default()
    }
}

fn mine_ore_system(
    mut stock: ResMut<Stock>,
    mut mining_complexes: Query<(&mut MiningComplex, &mut MiningComplexDeliveryTimer, &mut TechnicalState)>,
    mut dark_ores: Query<&mut DarkOre>,
    time: Res<Time>,
) {
    let mut rng = nanorand::tls_rng();
    for (mut mining_complex, mut timer, mut technical_state) in mining_complexes.iter_mut() {
        if !technical_state.is_operational() { continue; }
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            let ore_index = rng.generate_range(0..mining_complex.ore_entities_in_range.len());
            let ore_entity = mining_complex.ore_entities_in_range[ore_index];
            if let Ok(mut dark_ore) = dark_ores.get_mut(ore_entity) {
                let mined_amount = std::cmp::min(dark_ore.amount, 100);
                stock.add(ResourceType::DarkOre, mined_amount);
                dark_ore.amount -= mined_amount;
                if dark_ore.amount == 0 {
                    // Ore is already fully exhausted
                    mining_complex.ore_entities_in_range.swap_remove(ore_index);
                }
            } else {
                // This ore is gone
                mining_complex.ore_entities_in_range.swap_remove(ore_index);
            };
            if mining_complex.ore_entities_in_range.is_empty() {
                technical_state.has_ore_fields = Some(false);
            }
        }
    }
}