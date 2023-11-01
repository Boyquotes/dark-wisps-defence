use bevy::prelude::*;
use crate::buildings::common::{BuildingType, TowerType};
use crate::buildings::common_components::Building;
use crate::buildings::energy_relay::get_energy_relay_grid_imprint;
use crate::grids::common::GridImprint;
use crate::buildings::tower_blaster::get_tower_blaster_grid_imprint;
use crate::buildings::tower_cannon::get_tower_cannon_grid_imprint;
use crate::grids::common::CELL_SIZE;
use crate::grids::obstacles::{ObstacleGrid};
use crate::mouse::MouseInfo;
use crate::ui::interaction_state::UiInteractionState;

#[derive(Component, Default)]
pub enum GridObjectPlacer {
    #[default]
    None,
    Building(Building),
    Wall,
}

pub fn create_grid_object_placer_system(mut commands: Commands) {
    commands.spawn(
        GridObjectPlacer::default()
    ).insert(
        SpriteBundle::default()
    );
}

pub fn update_grid_object_placer_system(
    grid: Res<ObstacleGrid>,
    ui_interaction_state: Res<UiInteractionState>,
    mouse_info: Res<MouseInfo>,
    mut placer: Query<(&mut Transform, &mut Sprite, &mut Visibility, &mut GridObjectPlacer)>,
) {
    let (mut transform, mut sprite, mut visibility, mut grid_object_placer) = placer.single_mut();
    match *ui_interaction_state {
        UiInteractionState::PlaceGridObject => {}
        _ => {
            *visibility = Visibility::Hidden;
            *grid_object_placer = GridObjectPlacer::None;
            return;
        }
    };
    transform.translation = mouse_info.grid_coords.to_world_position().extend(10.);
    let is_imprint_placable = match &*grid_object_placer {
        GridObjectPlacer::Wall => {
            transform.translation += Vec3::new(8., 8., 0.);
            mouse_info.grid_coords.is_in_bounds(grid.bounds()) && grid[mouse_info.grid_coords].is_empty()
        },
        GridObjectPlacer::Building(building) => {
            match building.grid_imprint {
                GridImprint::Rectangle { width, height } => {
                    transform.translation.x += width as f32 * CELL_SIZE / 2.;
                    transform.translation.y += height as f32 * CELL_SIZE / 2.;
                }
            }
            grid.is_imprint_placable(mouse_info.grid_coords, building.grid_imprint)
        }
        _ => false
    };

    if is_imprint_placable {
        sprite.color = Color::rgba(0.0, 1.0, 0.0, 0.2);
    } else {
        sprite.color = Color::rgba(1.0, 0.0, 0.0, 0.2);
    }
}

pub fn on_click_initiate_grid_object_placer_system(
    mut ui_interaction_state: ResMut<UiInteractionState>,
    mut placer: Query<(&mut Sprite, &mut Visibility, &mut GridObjectPlacer)>,
    keys: Res<Input<KeyCode>>,
) {
    if !matches!(*ui_interaction_state, UiInteractionState::Free | UiInteractionState::PlaceGridObject) {
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        *ui_interaction_state = UiInteractionState::Free;
        return;
    }

    let placer_request = {
        if keys.just_pressed(KeyCode::W) {
            GridObjectPlacer::Wall
        } else if keys.just_pressed(KeyCode::E) {
            // Energy Relay
            let grid_imprint = get_energy_relay_grid_imprint();
            GridObjectPlacer::Building(
                Building{grid_imprint, building_type: BuildingType::EnergyRelay}
            )
        } else if keys.just_pressed(KeyCode::Key1) {
            // Tower Blaster
            let grid_imprint = get_tower_blaster_grid_imprint();
            GridObjectPlacer::Building(
                Building{grid_imprint, building_type: BuildingType::Tower(TowerType::Blaster)}
            )
        } else if keys.just_pressed(KeyCode::Key2) {
            // Tower Cannon
            let grid_imprint = get_tower_cannon_grid_imprint();
            GridObjectPlacer::Building(
                Building{grid_imprint, building_type: BuildingType::Tower(TowerType::Cannon)}
            )
        } else {
            return
        }
    };

    let (mut sprite, mut visibility, mut grid_object_placer) = placer.single_mut();
    *ui_interaction_state = UiInteractionState::PlaceGridObject;
    *visibility = Visibility::Visible;
    *grid_object_placer = placer_request;
    sprite.custom_size = match &*grid_object_placer {
        GridObjectPlacer::Building(building) => match building.grid_imprint {
            GridImprint::Rectangle { width, height } => Some(Vec2::new(width as f32 * CELL_SIZE, height as f32 * CELL_SIZE)),
        },
        GridObjectPlacer::Wall => Some(Vec2::new(CELL_SIZE, CELL_SIZE)),
        GridObjectPlacer::None => None,
    }
}