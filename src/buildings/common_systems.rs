use bevy::prelude::*;
use crate::buildings::common::{BuildingType, TowerType};
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
                _ => { return; }
            }
        }
        _ => { return; }
    };
}

