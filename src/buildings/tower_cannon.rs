use lib_grid::grids::energy_supply::EnergySupplyGrid;

use crate::prelude::*;
use crate::projectiles::cannonball::BuilderCannonball;
use crate::wisps::components::Wisp;
use crate::wisps::spawning::WISP_GRID_IMPRINT;

pub struct TowerCannonPlugin;
impl Plugin for TowerCannonPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(BuilderTowerCannon::on_add).add_systems(Update, (
                shooting_system.run_if(in_state(GameState::Running)),
            ));
    }
}

pub const TOWER_CANNON_BASE_IMAGE: &str = "buildings/tower_cannon.png";

#[derive(Component)]
pub struct BuilderTowerCannon {
    grid_position: GridCoords,
}

impl BuilderTowerCannon {
    pub fn new(grid_position: GridCoords) -> Self {
        Self { grid_position }
    }

    pub fn on_add(
        trigger: Trigger<OnAdd, BuilderTowerCannon>,
        mut commands: Commands,
        builders: Query<&BuilderTowerCannon>,
        asset_server: Res<AssetServer>,
        almanach: Res<Almanach>,
        energy_supply_grid: Res<EnergySupplyGrid>,
    ) {
        let entity = trigger.target();
        let Ok(builder) = builders.get(entity) else { return; };
        
        let grid_imprint = almanach.get_building_info(BuildingType::Tower(TowerType::Cannon)).grid_imprint;
        commands.entity(entity)
            .remove::<BuilderTowerCannon>()
            .insert((
                Sprite {
                    image: asset_server.load(TOWER_CANNON_BASE_IMAGE),
                    custom_size: Some(grid_imprint.world_size()),
                    ..Default::default()
                },
                MarkerTower,
                TowerCannon,
                builder.grid_position,
                grid_imprint,
                TechnicalState{ has_energy_supply: energy_supply_grid.is_imprint_suppliable(builder.grid_position, grid_imprint), ..default() },
                related![Modifiers[
                    (ModifierAttackRange(15), ModifierSourceBaseline),
                    (ModifierAttackSpeed(0.5), ModifierSourceBaseline),
                    (ModifierAttackDamage(50), ModifierSourceBaseline),
                    (ModifierMaxHealth(100), ModifierSourceBaseline),
                ]],
            ));
    }
}

pub fn shooting_system(
    mut commands: Commands,
    mut tower_cannons: Query<(&Transform, &TechnicalState, &mut TowerShootingTimer, &mut TowerWispTarget, &AttackDamage), With<TowerCannon>>,
    wisps: Query<(&GridPath, &GridCoords), With<Wisp>>,
) {
    for (transform, technical_state, mut timer, mut target, attack_damage) in tower_cannons.iter_mut() {
        if !technical_state.has_energy_supply { continue; }
        let TowerWispTarget::Wisp(target_wisp) = *target else { continue; };
        if !timer.0.finished() { continue; }

        let Ok((wisp_grid_path, wisp_coords)) = wisps.get(target_wisp) else {
            // Target wisp does not exist anymore
            *target = TowerWispTarget::SearchForNewTarget;
            continue;
        };

        // If wisps has path, target the next path position. Otherwise, target the wisp's current position.
        let target_world_position = wisp_grid_path.next_in_path().map_or(
            wisp_coords.to_world_position_centered(WISP_GRID_IMPRINT),
            |coords| coords.to_world_position_centered(WISP_GRID_IMPRINT)
        );

        commands.spawn(BuilderCannonball::new(transform.translation.xy(), target_world_position, attack_damage.clone()));
        timer.0.reset();
    }
}
