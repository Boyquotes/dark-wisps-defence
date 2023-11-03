use bevy::prelude::*;
use crate::buildings::common_components::{Building};
use crate::grids::common::{CELL_SIZE, GridCoords, GridImprint};
use crate::grids::energy_supply::SupplierEnergy;
use crate::grids::obstacles::{Field, ObstacleGrid};
use crate::mouse::MouseInfo;
use crate::ui::interaction_state::UiInteractionState;

pub fn on_click_building_display_info_system(
    mut ui_interaction_state: ResMut<UiInteractionState>,
    mouse: Res<Input<MouseButton>>,
    mouse_info: Res<MouseInfo>,
    obstacle_grid: Res<ObstacleGrid>,
) {
    if mouse.just_pressed(MouseButton::Right) && matches!(*ui_interaction_state, UiInteractionState::DisplayBuildingInfo(_)) {
        *ui_interaction_state = UiInteractionState::Free;
        return;
    }
    if !mouse.just_pressed(MouseButton::Left)
        || !mouse_info.grid_coords.is_in_bounds(obstacle_grid.bounds())
        || !matches!(*ui_interaction_state, UiInteractionState::Free | UiInteractionState::DisplayBuildingInfo(_))
    {
        return;
    }

    match &obstacle_grid[mouse_info.grid_coords] {
        Field::Building(entity, _) => {
            *ui_interaction_state = UiInteractionState::DisplayBuildingInfo((*entity).into());
        }
        _ => {}
    }
}

pub fn display_building_info_system(
    mut gizmos: Gizmos,
    ui_interaction_state: Res<UiInteractionState>,
    buildings: Query<(&Building, &GridCoords, Option<&SupplierEnergy>)>,
) {
    let UiInteractionState::DisplayBuildingInfo(building_id) = &*ui_interaction_state else {
        return;
    };

    let Ok((building, grid_coords, energy_provider)) = buildings.get(**building_id) else { return; };
    if let Some(energy_provider) = energy_provider {
        let position = grid_coords.to_world_position() + match building.grid_imprint {
            GridImprint::Rectangle { width, height } => Vec2::new(width as f32 * CELL_SIZE / 2., height as f32 * CELL_SIZE / 2.),
        };
        gizmos.circle_2d(
            position,
            energy_provider.range as f32 * CELL_SIZE,
            Color::YELLOW,
        );

    }

}