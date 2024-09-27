use bevy::color::palettes::css::{BLUE, YELLOW};
use bevy::render::camera::RenderTarget;
use bevy::render::render_resource::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};
use crate::inventory::almanach;
use crate::prelude::*;
use crate::grids::energy_supply::SupplierEnergy;
use crate::grids::obstacles::{Field, ObstacleGrid};
use crate::mouse::MouseInfo;

pub struct DisplayBuildingInfoPlugin;
impl Plugin for DisplayBuildingInfoPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<BuildingUiFocusChangedEvent>()
            .add_systems(Startup, (
                initialize_display_building_info_system,
            ))
            .add_systems(Update, (
                hide_system.run_if(in_state(UiInteraction::DisplayBuildingInfo)),
                show_on_click_system.run_if(in_state(UiInteraction::Free).or_else(in_state(UiInteraction::DisplayBuildingInfo))),
                on_building_destroyed_system.run_if(in_state(UiInteraction::DisplayBuildingInfo).and_then(on_event::<BuildingDestroyedEvent>())),
                display_building_info_system.run_if(in_state(UiInteraction::DisplayBuildingInfo)),
            ))
            .add_systems(OnEnter(UiInteraction::DisplayBuildingInfo), on_display_enter_system)
            .add_systems(OnExit(UiInteraction::DisplayBuildingInfo), on_display_exit_system);
    }
}

#[derive(Component)]
pub struct DisplayBuildingInfo {
    pub building_entity: Entity,
}
#[derive(Component)]
struct DisplayBuildingInfoCamera;
#[derive(Component)]
struct BuildingNameText;

/// Event emitted when the user clicks on a building
#[derive(Event)]
pub struct BuildingUiFocusChangedEvent {
    pub building_entity: Entity,
}



fn initialize_display_building_info_system(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let camera_image_handle = {
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
                target: RenderTarget::Image(camera_image_handle.clone()),
                is_active: false,
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
            visibility: Visibility::Hidden,
            ..default()
        },
        DisplayBuildingInfo { building_entity: Entity::PLACEHOLDER },
    )).with_children(|parent| {
        // Camera image (Left side)
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
            UiImage::new(camera_image_handle),
        ));
        // Right panel
        parent.spawn((
            NodeBundle {
                style: Style {
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Start,
                    align_items: AlignItems::Start,
                    padding: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                ..default()
            },
        )).with_children(|parent| {
            // Building name
            parent.spawn((
                TextBundle {
                    text: Text::from_section("### Building Name ###", TextStyle{ color: BLUE.into(), ..default() }),
                    style: Style {
                        margin: UiRect{ left: Val::Px(4.), ..default() },
                        ..default()
                    },
                    ..default()
                },
                BuildingNameText,
            ));
        });
    });

}

fn on_display_enter_system(
    mut display_building_info: Query<&mut Visibility, With<DisplayBuildingInfo>>,
    mut building_info_camera: Query<&mut Camera, With<DisplayBuildingInfoCamera>>,
) {
    *display_building_info.single_mut() = Visibility::Inherited;
    building_info_camera.single_mut().is_active = true;
}

fn on_display_exit_system(
    mut display_building_info: Query<&mut Visibility, With<DisplayBuildingInfo>>,
    mut building_info_camera: Query<&mut Camera, With<DisplayBuildingInfoCamera>>,
) {
    *display_building_info.single_mut() = Visibility::Hidden;
    building_info_camera.single_mut().is_active = false;
}

fn hide_system(
    mouse: Res<ButtonInput<MouseButton>>,
    mut ui_interaction_state: ResMut<NextState<UiInteraction>>,
) {
    if mouse.just_pressed(MouseButton::Right) {
        ui_interaction_state.set(UiInteraction::Free);
    }
}

fn show_on_click_system(
    almanach: Res<Almanach>,
    mut ui_interaction_state: ResMut<NextState<UiInteraction>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mouse_info: Res<MouseInfo>,
    obstacle_grid: Res<ObstacleGrid>,
    mut building_ui_focus_changed_events: EventWriter<BuildingUiFocusChangedEvent>,
    mut display_building_info: Query<&mut DisplayBuildingInfo>,
    mut building_info_camera: Query<&mut Transform, With<DisplayBuildingInfoCamera>>,
    mut building_name_text: Query<&mut Text, With<BuildingNameText>>,
    buildings: Query<(&BuildingType, &GridImprint, &GridCoords), With<Building>>,
) {
    if !mouse.just_pressed(MouseButton::Left) || !mouse_info.grid_coords.is_in_bounds(obstacle_grid.bounds()) { return; }

    let Field::Building(entity, _, _) = &obstacle_grid[mouse_info.grid_coords] else { return; };
    let Ok((building_type, grid_imprint, grid_coords)) = buildings.get(*entity) else { return; };
    // Center the camera on the building
    let mut camera_transform = building_info_camera.single_mut();
    let building_world_position = grid_coords.to_world_position_centered(*grid_imprint);
    camera_transform.translation.x = building_world_position.x;
    camera_transform.translation.y = building_world_position.y;
    // Update the building name
    building_name_text.single_mut().sections[0].value = almanach.get_building_name(*building_type).to_string();

    let mut display_building_info = display_building_info.single_mut();
    display_building_info.building_entity = *entity;

    building_ui_focus_changed_events.send(BuildingUiFocusChangedEvent { building_entity: *entity });
    ui_interaction_state.set(UiInteraction::DisplayBuildingInfo);
}

fn on_building_destroyed_system(
    mut ui_interaction_state: ResMut<NextState<UiInteraction>>,
    mut events: EventReader<BuildingDestroyedEvent>,
    display_building_info: Query<&DisplayBuildingInfo>,
) {
    let display_building_info = display_building_info.single();
    for event in events.read() {
        if event.0 == display_building_info.building_entity {
            ui_interaction_state.set(UiInteraction::Free);
        }
    }
}

fn display_building_info_system(
    mut gizmos: Gizmos,
    buildings: Query<(&GridImprint, &GridCoords, Option<&SupplierEnergy>), With<Building>>,
    display_building_info: Query<&DisplayBuildingInfo>,
) {
    let building_entity = display_building_info.single().building_entity;
    let Ok((grid_imprint, grid_coords, energy_provider)) = buildings.get(building_entity) else { return; };
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