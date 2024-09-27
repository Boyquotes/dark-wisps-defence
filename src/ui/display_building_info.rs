use bevy::color::palettes::css::YELLOW;
use bevy::render::camera::RenderTarget;
use bevy::render::render_resource::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};
use crate::prelude::*;
use crate::grids::energy_supply::SupplierEnergy;
use crate::grids::obstacles::{Field, ObstacleGrid};
use crate::mouse::MouseInfo;
use crate::ui::interaction_state::UiInteractionState;

pub struct DisplayBuildingInfoPlugin;
impl Plugin for DisplayBuildingInfoPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, (
                initialize_display_building_info_system,
            ))
            .add_systems(Update, (
                on_click_building_display_info_system,
                display_building_info_system,
            ));
    }
}

#[derive(Component)]
pub struct DisplayBuildingInfoCamera;
#[derive(Component)]
pub struct DisplayBuildingInfo;



fn initialize_display_building_info_system(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let image_handle = {
        let size = Extent3d {
            width: 128,
            height: 128,
            ..default()
        };
        let mut image = Image {
            texture_descriptor: TextureDescriptor {
                label: None,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            ..Default::default()
        };
        image.resize(size);
        images.add(image)
    };
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 1,
                hdr: true,
                target: RenderTarget::Image(image_handle.clone()),
                ..default()
            },
            projection: OrthographicProjection {
                near: -1000.,
                far: 1000.,
                scale: 2.,
                ..default()
            },
            ..default()
        },
        DisplayBuildingInfoCamera,
    ));
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(5.0),
                left: Val::Percent(25.),
                width: Val::Percent(50.0),
                height: Val::Px(140.0),
                border: UiRect::all(Val::Px(4.0)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(4.0)),
                ..default()
            },
            background_color: Color::linear_rgba(0.46, 0.62, 0.67, 1.).into(),
            border_radius: BorderRadius::all(Val::Px(7.)),
            border_color: Color::linear_rgba(0., 0.2, 1., 1.).into(),
            ..default()
        },
        DisplayBuildingInfo,
    )).with_children(|parent| {
        parent.spawn((
            NodeBundle {
                style: Style {
                    width: Val::Px(128.0),
                    height: Val::Px(128.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                border_color: YELLOW.into(),
                ..default()
            },
            UiImage::new(image_handle),
        ));
    });

}

pub fn on_click_building_display_info_system(
    mut ui_interaction_state: ResMut<UiInteractionState>,
    mouse: Res<ButtonInput<MouseButton>>,
    mouse_info: Res<MouseInfo>,
    obstacle_grid: Res<ObstacleGrid>,
    mut building_info_camera: Query<(&mut Camera, &mut Transform), With<DisplayBuildingInfoCamera>>,
    buildings: Query<(&GridImprint, &GridCoords), With<Building>>,
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
            let Ok((grid_imprint, grid_coords)) = buildings.get(*entity) else { return; };
            let (mut camera, mut transform) = building_info_camera.single_mut();
            camera.is_active = true;
            let building_world_position = grid_coords.to_world_position_centered(grid_imprint);
            transform.translation.x = building_world_position.x;
            transform.translation.y = building_world_position.y;
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