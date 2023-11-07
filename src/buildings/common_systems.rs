use bevy::prelude::*;
use crate::buildings::common::{BuildingType, TowerType};
use crate::buildings::common_components::{Building, MarkerTower, TechnicalState, TowerRange, TowerShootingTimer, TowerWispTarget};
use crate::grids::base::GridVersion;
use crate::grids::common::GridCoords;
use crate::grids::emissions::EmitterCreatedEvent;
use crate::grids::energy_supply::{EnergySupplyGrid, SupplierCreatedEvent, SupplierEnergy};
use crate::grids::obstacles::ObstacleGrid;
use crate::grids::wisps::WispsGrid;
use crate::mouse::MouseInfo;
use crate::search::targetfinding::target_find_closest_wisp;
use crate::ui::grid_object_placer::GridObjectPlacer;

pub fn onclick_building_spawn_system(
    mut commands: Commands,
    mut emitter_created_event_writer: EventWriter<EmitterCreatedEvent>,
    mut supplier_created_event_writer: EventWriter<SupplierCreatedEvent>,
    mut obstacle_grid: ResMut<ObstacleGrid>,
    energy_supply_grid: Res<EnergySupplyGrid>,
    mouse: Res<Input<MouseButton>>,
    mouse_info: Res<MouseInfo>,
    grid_object_placer: Query<&GridObjectPlacer>,
) {
    let mouse_coords = mouse_info.grid_coords;
    if !mouse.pressed(MouseButton::Left) || !mouse_coords.is_in_bounds(obstacle_grid.bounds()) { return; }
    match &*grid_object_placer.single() {
        GridObjectPlacer::Building(building) => {
            if !obstacle_grid.is_imprint_placable(mouse_coords, building.grid_imprint) { return; }
            match building.building_type {
                BuildingType::EnergyRelay => {
                    super::energy_relay::create_energy_relay(
                        &mut commands, &mut emitter_created_event_writer, &mut supplier_created_event_writer, &mut obstacle_grid, mouse_coords
                    );
                }
                BuildingType::Tower(TowerType::Blaster) => {
                    super::tower_blaster::create_tower_blaster(&mut commands, &mut obstacle_grid, &energy_supply_grid, mouse_coords);
                },
                BuildingType::Tower(TowerType::Cannon) => {
                    super::tower_cannon::create_tower_cannon(&mut commands, &mut obstacle_grid, &energy_supply_grid, mouse_coords);
                },
                _ => panic!("Trying to place a non-supported building")            }
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