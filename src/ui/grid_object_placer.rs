use bevy::prelude::*;
use crate::grid::GridImprint;
use crate::buildings::main_base::get_main_base_grid_imprint;
use crate::grid::{CELL_SIZE, ObstacleGrid};
use crate::mouse::MouseInfo;
use crate::ui::interaction_state::UiInteractionState;

#[derive(Component, Default, PartialEq)]
pub enum GridObjectPlacer {
    #[default]
    None,
    Building(GridImprint),
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
    transform.translation = mouse_info.grid_coords.to_world_coords().extend(10.);
    let is_imprint_placable = match *grid_object_placer {
        GridObjectPlacer::Wall => {
            transform.translation += Vec3::new(8., 8., 0.);
            mouse_info.grid_coords.is_in_bounds(grid.bounds()) && grid[mouse_info.grid_coords].is_empty()
        },
        GridObjectPlacer::Building(imprint) => {
            match imprint {
                GridImprint::Rectangle { width, height } => {
                    transform.translation.x += width as f32 * CELL_SIZE / 2.;
                    transform.translation.y += height as f32 * CELL_SIZE / 2.;
                }
            }
            grid.is_imprint_placable(mouse_info.grid_coords, imprint)
        }
        _ => false
    };

    if is_imprint_placable {
        sprite.color = Color::rgba(0.0, 1.0, 0.0, 0.2);
    } else {
        sprite.color = Color::rgba(1.0, 0.0, 0.0, 0.2);
    }
}

pub fn placer_test() {
    let placer = 3;

}

pub fn on_click_initiate_grid_object_placer_system(
    mut ui_interaction_state: ResMut<UiInteractionState>,
    mut placer: Query<(&mut Sprite, &mut Visibility, &mut GridObjectPlacer)>,
    keys: Res<Input<KeyCode>>,
) {
    match *ui_interaction_state {
        UiInteractionState::Free | UiInteractionState::PlaceGridObject => {
            if keys.just_pressed(KeyCode::Escape) {
                *ui_interaction_state = UiInteractionState::Free;
            } else if keys.just_pressed(KeyCode::W) {
                *ui_interaction_state = UiInteractionState::PlaceGridObject;
                let (mut sprite, mut visibility, mut grid_object_placer) = placer.single_mut();
                *grid_object_placer = GridObjectPlacer::Wall;
                sprite.custom_size = Some(Vec2::new(CELL_SIZE, CELL_SIZE));
                *visibility = Visibility::Visible;
            } else if keys.just_pressed(KeyCode::Key1) {
                // Main Base
                *ui_interaction_state = UiInteractionState::PlaceGridObject;
                let (mut sprite, mut visibility, mut grid_object_placer) = placer.single_mut();
                let imprint = get_main_base_grid_imprint();
                let (sprite_width, sprite_height) = match imprint {
                    GridImprint::Rectangle { width, height } => (width as f32 * CELL_SIZE, height as f32 * CELL_SIZE),
                };
                *grid_object_placer = GridObjectPlacer::Building(imprint);

                sprite.custom_size = Some(Vec2::new(sprite_width, sprite_height));
                *visibility = Visibility::Visible;
            }
        }
    }
}