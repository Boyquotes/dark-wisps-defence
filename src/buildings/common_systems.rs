use bevy::prelude::*;
use crate::buildings::common::{BuildingType, TowerType};
use crate::buildings::common_components::TowerShootingTimer;
use crate::buildings::tower_blaster::create_tower_blaster;
use crate::grids::obstacles::ObstacleGrid;
use crate::mouse::MouseInfo;
use crate::ui::grid_object_placer::GridObjectPlacer;

pub fn onclick_building_spawn_system(
    mut commands: Commands,
    mut grid: ResMut<ObstacleGrid>,
    mouse: Res<Input<MouseButton>>,
    mouse_info: Res<MouseInfo>,
    grid_object_placer: Query<&GridObjectPlacer>,
) {
    let mouse_coords = mouse_info.grid_coords;
    if !mouse.pressed(MouseButton::Left) || !mouse_coords.is_in_bounds(grid.bounds()) { return; }
    match &*grid_object_placer.single() {
        GridObjectPlacer::Building(building) => {
            if !grid.is_imprint_placable(mouse_coords, building.grid_imprint) { return; }
            match building.building_type {
                BuildingType::Tower(TowerType::Blaster) => {
                    create_tower_blaster(&mut commands, &mut grid, mouse_coords);
                },
                BuildingType::Tower(TowerType::Cannon) => {
                    super::tower_cannon::create_tower_cannon(&mut commands, &mut grid, mouse_coords);
                },
                _ => panic!("Trying to place a non-supported building")            }
        }
        _ => { return; }
    };
}

pub fn tick_shooting_timers_system(
    mut shooting_timers: Query<&mut TowerShootingTimer>,
    time: Res<Time>,
) {
    shooting_timers.iter_mut().for_each(|mut timer| { timer.0.tick(time.delta()); });
}