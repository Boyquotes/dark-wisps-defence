use crate::prelude::*;
use crate::buildings::common::{BuildingType, TowerType};
use crate::buildings::common_components::{Building, MarkerTower, MarkerTowerRotationalTop, TechnicalState, TowerRange, TowerShootingTimer, TowerTopRotation, TowerWispTarget};
use crate::buildings::energy_relay::BuilderEnergyRelay;
use crate::buildings::exploration_center::BuilderExplorationCenter;
use crate::buildings::main_base::{EventMoveMainBase, MarkerMainBase};
use crate::buildings::mining_complex::{BuilderMiningComplex, MINING_COMPLEX_GRID_IMPRINT};
use crate::buildings::tower_blaster::BuilderTowerBlaster;
use crate::buildings::tower_cannon::BuilderTowerCannon;
use crate::buildings::tower_rocket_launcher::BuilderTowerRocketLauncher;
use crate::grids::base::GridVersion;
use crate::grids::energy_supply::{EnergySupplyGrid, SupplierEnergy};
use crate::grids::obstacles::{Field, ObstacleGrid};
use crate::grids::wisps::WispsGrid;
use crate::inventory::almanach::Almanach;
use crate::inventory::resources::DarkOreStock;
use crate::mouse::MouseInfo;
use crate::search::targetfinding::target_find_closest_wisp;
use crate::ui::grid_object_placer::GridObjectPlacer;
use crate::utils::math::angle_difference;
use crate::wisps::components::Wisp;

pub fn onclick_building_spawn_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut obstacle_grid: ResMut<ObstacleGrid>,
    energy_supply_grid: Res<EnergySupplyGrid>,
    mouse: Res<ButtonInput<MouseButton>>,
    mouse_info: Res<MouseInfo>,
    almanach: Res<Almanach>,
    mut dark_ore_stock: ResMut<DarkOreStock>,
    grid_object_placer: Query<&GridObjectPlacer>,
    mut main_base: Query<(Entity, &GridCoords), With<MarkerMainBase>>,
) {
    let mouse_coords = mouse_info.grid_coords;
    if mouse_info.is_over_ui || !mouse.just_released(MouseButton::Left) || !mouse_coords.is_in_bounds(obstacle_grid.bounds()) { return; }
    match &*grid_object_placer.single() {
        GridObjectPlacer::Building(building) => {
            if !obstacle_grid.imprint_query_all(mouse_coords, building.grid_imprint, |field| field.is_empty()) { return; }
            let dark_ore_price = almanach.get_building_cost(building.building_type);
            if dark_ore_stock.amount < dark_ore_price { println!("Not enough dark ore"); return; }
            dark_ore_stock.amount -= dark_ore_price;
            // ---
            enum GridAction {
                Imprint(Entity),
                Reprint{entity: Entity, old_coords: GridCoords},
            }
            // ---
            let grid_action = match building.building_type {
                BuildingType::EnergyRelay => {
                    let mut builder = BuilderEnergyRelay::new(mouse_coords);
                    let entity = builder.entity.get(&mut commands);
                    commands.add(builder);
                    GridAction::Imprint(entity)
                }
                BuildingType::ExplorationCenter => {
                    let mut builder = BuilderExplorationCenter::new(mouse_coords);
                    let entity = builder.entity.get(&mut commands);
                    commands.add(builder);
                    GridAction::Imprint(entity)
                }
                BuildingType::Tower(TowerType::Blaster) => {
                    GridAction::Imprint(BuilderTowerBlaster::new(mouse_coords, &asset_server)
                        .update_energy_supply(&energy_supply_grid)
                        .spawn(&mut commands, &mut obstacle_grid))
                },
                BuildingType::Tower(TowerType::Cannon) => {
                    GridAction::Imprint(BuilderTowerCannon::new(mouse_coords, &asset_server)
                        .update_energy_supply(&energy_supply_grid)
                        .spawn(&mut commands, &mut obstacle_grid))
                },
                BuildingType::Tower(TowerType::RocketLauncher) => {
                    GridAction::Imprint(BuilderTowerRocketLauncher::new(mouse_coords, &asset_server)
                        .update_energy_supply(&energy_supply_grid)
                        .spawn(&mut commands, &mut obstacle_grid))
                },
                BuildingType::MainBase => {
                    let (main_base_entity, main_base_coords) = main_base.single_mut();
                    commands.add(EventMoveMainBase { new_grid_position: mouse_coords });
                    GridAction::Reprint{entity: main_base_entity, old_coords: *main_base_coords}
                }
                _ => panic!("Trying to place a non-supported building") 
            };
            match grid_action {
                GridAction::Imprint(entity) => obstacle_grid.imprint(mouse_coords, Field::Building(entity, building.building_type), building.grid_imprint),
                GridAction::Reprint{entity, old_coords} => obstacle_grid.reprint(old_coords, mouse_coords, Field::Building(entity, building.building_type), building.grid_imprint),
            }
        }
        GridObjectPlacer::MiningComplex => {
            if !obstacle_grid.imprint_query_all(mouse_coords, MINING_COMPLEX_GRID_IMPRINT, |field| field.is_dark_ore()) { return; }
            let dark_ore_price = almanach.get_building_cost(BuildingType::MiningComplex);
            if dark_ore_stock.amount < dark_ore_price { println!("Not enough dark ore"); return; }
            dark_ore_stock.amount -= dark_ore_price;
            let Field::DarkOre(dark_ore) = obstacle_grid[mouse_coords] else { unreachable!() };
            BuilderMiningComplex::new(mouse_coords, &asset_server)
                .update_energy_supply(&energy_supply_grid)
                .spawn(&mut commands, &mut obstacle_grid, dark_ore);
        }
        _ => { return; }
    };
}

pub fn targeting_system(
    mut tower_cannons: Query<(&GridCoords, &Building, &TechnicalState, &TowerRange, &mut TowerWispTarget), With<MarkerTower>>,
    obstacle_grid: Res<ObstacleGrid>,
    wisps_grid: Res<WispsGrid>,
) {
    for (coords, building, technical_state, range, mut target) in tower_cannons.iter_mut() {
        if !technical_state.has_energy_supply { continue; }
        match *target {
            TowerWispTarget::Wisp(_) => continue,
            TowerWispTarget::NoValidTargets(grid_version) => {
                if grid_version == wisps_grid.version {
                    continue;
                }
            },
            TowerWispTarget::SearchForNewTarget => {},
        }
        if let Some((_a, target_wisp)) = target_find_closest_wisp(
            &obstacle_grid,
            &wisps_grid,
            building.grid_imprint.covered_coords(*coords),
            range.0,
            true,
        ) {
            *target = TowerWispTarget::Wisp(target_wisp);
        } else {
            *target = TowerWispTarget::NoValidTargets(wisps_grid.version);
        }
    }
}

pub fn tick_shooting_timers_system(
    mut shooting_timers: Query<&mut TowerShootingTimer>,
    time: Res<Time>,
) {
    shooting_timers.iter_mut().for_each(|mut timer| { timer.0.tick(time.delta()); });
}

pub fn check_energy_supply_system(
    mut current_energy_supply_grid_version: Local<GridVersion>,
    energy_supply_grid: Res<EnergySupplyGrid>,
    mut buildings: Query<(&Building, &GridCoords, &mut TechnicalState), Without<SupplierEnergy>>
) {
    if *current_energy_supply_grid_version == energy_supply_grid.version { return; }
    *current_energy_supply_grid_version = energy_supply_grid.version;
    for (building, grid_coords, mut technical_state) in buildings.iter_mut() {
        technical_state.has_energy_supply = energy_supply_grid.is_imprint_suppliable(*grid_coords, building.grid_imprint);
    }
}

pub fn rotate_tower_top_system(
    mut tower_rotational_top: Query<(&MarkerTowerRotationalTop, &mut Transform)>,
    towers: Query<&TowerTopRotation, With<MarkerTower>>,
) {
    for (tower_rotational_top, mut tower_top_transform) in tower_rotational_top.iter_mut() {
        let parent_building = tower_rotational_top.0;
        let tower_top_rotation = towers.get(*parent_building).unwrap();

        // Offset due to image naturally pointing downwards
        tower_top_transform.rotation = Quat::from_rotation_z(tower_top_rotation.current_angle);
    }
}

pub fn rotational_aiming_system(
    time: Res<Time>,
    mut towers: Query<(&mut TowerTopRotation, &TowerWispTarget, &Transform), Without<Wisp>>,
    wisps: Query<&Transform, With<Wisp>>,
) {
    for (mut rotation, target, tower_transform) in towers.iter_mut() {
        let TowerWispTarget::Wisp(target_wisp) = target else { continue; };
        let Ok(wisp_position) = wisps.get(**target_wisp).map(|target| target.translation.xy()) else { continue; };

        let direction_to_target = wisp_position - tower_transform.translation.xy();
        let target_angle = direction_to_target.y.atan2(direction_to_target.x);

        let angle_diff = angle_difference(target_angle, rotation.current_angle);

        let rotation_delta = rotation.speed * time.delta_seconds();
        rotation.current_angle += angle_diff.clamp(-rotation_delta, rotation_delta);
    }
}