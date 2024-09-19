use bevy::color::palettes::css::YELLOW;
use crate::prelude::*;
use crate::grids::energy_supply::SupplierEnergy;
use crate::grids::obstacles::{Field, ObstacleGrid};
use crate::mouse::MouseInfo;
use crate::ui::interaction_state::UiInteractionState;

pub fn on_click_building_display_info_system(
    mut ui_interaction_state: ResMut<UiInteractionState>,
    mouse: Res<ButtonInput<MouseButton>>,
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
        Field::Building(entity, _, _) => {
            *ui_interaction_state = UiInteractionState::DisplayBuildingInfo((*entity).into());
        }
        _ => {}
    }
}

pub fn display_building_info_system(
    mut gizmos: Gizmos,
    ui_interaction_state: Res<UiInteractionState>,
    buildings: Query<(&GridImprint, &GridCoords, Option<&SupplierEnergy>), With<Building>>,
) {
    let UiInteractionState::DisplayBuildingInfo(building_id) = &*ui_interaction_state else {
        return;
    };

    let Ok((grid_imprint, grid_coords, energy_provider)) = buildings.get(**building_id) else { return; };
    if let Some(energy_provider) = energy_provider {
        let position = grid_coords.to_world_position() + match *grid_imprint {
            GridImprint::Rectangle { width, height } => Vec2::new(width as f32 * CELL_SIZE / 2., height as f32 * CELL_SIZE / 2.),
        };
        gizmos.circle_2d(
            position,
            energy_provider.range as f32 * CELL_SIZE,
            YELLOW,
        );

    }

}