use lib_grid::grids::obstacles::{Field, BelowField, ObstacleGrid};
use lib_grid::grids::energy_supply::EnergySupplyGrid;

use crate::map_objects::dark_ore::DarkOre;
use crate::prelude::*;

pub struct MiningComplexPlugin;
impl Plugin for MiningComplexPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(BuilderMiningComplex::on_add).add_systems(Update, (
                mine_ore_system.run_if(in_state(GameState::Running)),
            ));
    }
}

pub const MINING_COMPLEX_BASE_IMAGE: &str = "buildings/mining_complex.png";


#[derive(Component)]
pub struct MiningComplexDeliveryTimer(pub Timer);
#[derive(Component)]
pub struct MiningRange(GridImprint);

#[derive(Component)]
pub struct BuilderMiningComplex {
    grid_position: GridCoords,
}
impl BuilderMiningComplex {
    pub fn new(grid_position: GridCoords) -> Self {
        Self { grid_position }
    }

    pub fn on_add(
        trigger: Trigger<OnAdd, BuilderMiningComplex>,
        mut commands: Commands,
        builders: Query<&BuilderMiningComplex>,
        obstacle_grid: Res<ObstacleGrid>,
        asset_server: Res<AssetServer>,
        almanach: Res<Almanach>,
        energy_supply_grid: Res<EnergySupplyGrid>,
    ) {
        let entity = trigger.target();
        let Ok(builder) = builders.get(entity) else { return; };
        
        let building_info = almanach.get_building_info(BuildingType::MiningComplex);
        let grid_imprint = building_info.grid_imprint;
        let ore_entities_in_range = obstacle_grid.imprint_query_element(builder.grid_position, grid_imprint, query_dark_ore_helper);
        commands.entity(entity)
            .remove::<BuilderMiningComplex>()
            .insert((
                Sprite {
                    image: asset_server.load(MINING_COMPLEX_BASE_IMAGE),
                    custom_size: Some(grid_imprint.world_size()),
                    ..Default::default()
                },
                TechnicalState{ 
                    has_energy_supply: energy_supply_grid.is_imprint_suppliable(builder.grid_position, grid_imprint),
                    has_ore_fields: Some(!ore_entities_in_range.is_empty()),
                },
                MiningComplex { ore_entities_in_range },
                builder.grid_position,
                grid_imprint,
                MiningRange(grid_imprint),
                MiningComplexDeliveryTimer(Timer::from_seconds(1.0, TimerMode::Repeating)),
                related![Modifiers[
                    (ModifierMaxHealth::from_baseline(building_info), ModifierSourceBaseline),
                ]],
            ));
    }
}

// Helper to execute on every obstacle grid field to gather the dark_ore entities
fn query_dark_ore_helper(field: &Field) -> Option<Entity> {
    if let Field::Building(_, BuildingType::MiningComplex, BelowField::DarkOre(dark_ore_entity)) = field { Some(*dark_ore_entity) } else { None }
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