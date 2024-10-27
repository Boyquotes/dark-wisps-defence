use bevy::color::palettes::css::{BLACK, BLUE, YELLOW};
use bevy::render::camera::RenderTarget;
use bevy::render::render_resource::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};
use bevy::text::BreakLineOn;
use crate::prelude::*;
use crate::grids::obstacles::{Field, ObstacleGrid};
use crate::mouse::MouseInfo;

pub struct DisplayInfoPanelPlugin;
impl Plugin for DisplayInfoPanelPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<DisplayInfoPanelState>()
            .add_event::<UiMapObjectFocusChangedEvent>()
            .add_systems(Startup, (
                initialize_display_info_panel_system,
            ))
            .add_systems(Update, (
                hide_system.run_if(in_state(UiInteraction::DisplayInfoPanel)),
                show_on_click_system.run_if(in_state(UiInteraction::Free).or_else(in_state(UiInteraction::DisplayInfoPanel))),
                on_building_destroyed_system.run_if(in_state(UiInteraction::DisplayInfoPanel).and_then(on_event::<BuildingDestroyedEvent>())),
                update_building_info_panel_system.run_if(in_state(DisplayInfoPanelState::DisplayBuilding)),
                on_display_panel_focus_changed_system.run_if(on_event::<UiMapObjectFocusChangedEvent>())
            ))
            .add_systems(OnEnter(UiInteraction::DisplayInfoPanel), on_display_enter_system)
            .add_systems(OnExit(UiInteraction::DisplayInfoPanel), on_display_exit_system)
            .add_systems(OnEnter(DisplayInfoPanelState::DisplayBuilding), on_display_building_enter_system)
            .add_systems(OnExit(DisplayInfoPanelState::DisplayBuilding), on_display_building_exit_system)
            .add_systems(OnEnter(DisplayInfoPanelState::DisplayQuantumField), on_display_quantum_field_enter_system)
            .add_systems(OnExit(DisplayInfoPanelState::DisplayQuantumField), on_display_quantum_field_exit_system);
    }
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum DisplayInfoPanelState {
    #[default]
    None,
    DisplayBuilding,
    DisplayQuantumField,
}

#[derive(Component)]
enum DisplayInfoPanel {
    None,
    Building(BuildingType, Entity),
    QuantumField(Entity),
}
#[derive(Component)]
struct DisplayInfoPanelCamera;
/// --- Buildings sub-panel ---
#[derive(Component)]
struct BuildingPanel;
#[derive(Component)]
struct BuildingNameText;
#[derive(Component)]
struct BuildingHealthbar;
#[derive(Component)]
struct BuildingHealthbarValue;
/// --- Quantum Fields sub-panel ---
#[derive(Component)]
struct QuantumFieldPanel;
/// ---------------------------

/// Event emitted when the user clicks on a building
#[derive(Event)]
pub enum UiMapObjectFocusChangedEvent {
    Unfocus,
    Focus(Entity),
}
impl Command for UiMapObjectFocusChangedEvent {
    fn apply(self, world: &mut World) {
        world.send_event(self);
    }
}

fn on_display_enter_system(
    mut display_info_panel: Query<&mut Visibility, With<DisplayInfoPanel>>,
    mut info_panel_camera: Query<&mut Camera, With<DisplayInfoPanelCamera>>,
) {
    *display_info_panel.single_mut() = Visibility::Inherited;
    info_panel_camera.single_mut().is_active = true;
}

fn on_display_exit_system(
    mut commands: Commands,
    mut next_display_info_panel_state: ResMut<NextState<DisplayInfoPanelState>>,
    mut display_info_panel: Query<&mut Visibility, With<DisplayInfoPanel>>,
    mut info_panel_camera: Query<&mut Camera, With<DisplayInfoPanelCamera>>,
) {
    *display_info_panel.single_mut() = Visibility::Hidden;
    info_panel_camera.single_mut().is_active = false;
    next_display_info_panel_state.set(DisplayInfoPanelState::None);
    commands.add(UiMapObjectFocusChangedEvent::Unfocus);
}

fn hide_system(
    mouse: Res<ButtonInput<MouseButton>>,
    mut ui_interaction_state: ResMut<NextState<UiInteraction>>,
) {
    if mouse.just_pressed(MouseButton::Right) {
        ui_interaction_state.set(UiInteraction::Free);
    }
}

fn on_display_building_enter_system(
    almanach: Res<Almanach>,
    display_info_panel: Query<&DisplayInfoPanel>,
    mut building_panel: Query<&mut Style, With<BuildingPanel>>,
    mut building_name_text: Query<&mut Text, With<BuildingNameText>>,
) {
    let DisplayInfoPanel::Building(building_type, _) = display_info_panel.single() else { unreachable!() };
    // Update the building name
    building_name_text.single_mut().sections[0].value = almanach.get_building_name(*building_type).to_string();

    building_panel.single_mut().display = Display::Flex;
}

fn on_display_building_exit_system(
    mut building_panel: Query<&mut Style, With<BuildingPanel>>,
) {
    building_panel.single_mut().display = Display::None;
}

fn on_display_quantum_field_enter_system(
    mut quantum_field_panel: Query<&mut Style, With<QuantumFieldPanel>>,
) {
    quantum_field_panel.single_mut().display = Display::Flex;
}

fn on_display_quantum_field_exit_system(
    mut quantum_field_panel: Query<&mut Style, With<QuantumFieldPanel>>,
) {
    quantum_field_panel.single_mut().display = Display::None;
}

fn on_display_panel_focus_changed_system(
    mut events: EventReader<UiMapObjectFocusChangedEvent>,
    mut info_panel_camera: Query<&mut Transform, With<DisplayInfoPanelCamera>>,
    grid_positions: Query<(&GridCoords, &GridImprint)>,
) {
    let Some(event) = events.read().last() else { unreachable!() };
    let UiMapObjectFocusChangedEvent::Focus(entity) = event else { return; };

    // Center the camera on the focused structure
    let Ok((grid_coords, grid_imprint)) = grid_positions.get(*entity) else { return; };
    let world_position = grid_coords.to_world_position_centered(*grid_imprint);
    let mut camera_transform = info_panel_camera.single_mut();
    camera_transform.translation.x = world_position.x;
    camera_transform.translation.y = world_position.y;
}

fn show_on_click_system(
    mut ui_interaction_state: ResMut<NextState<UiInteraction>>,
    mut next_display_info_panel_state: ResMut<NextState<DisplayInfoPanelState>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mouse_info: Res<MouseInfo>,
    obstacle_grid: Res<ObstacleGrid>,
    mut ui_map_object_focus_changed_events: EventWriter<UiMapObjectFocusChangedEvent>,
    mut display_info_panel: Query<&mut DisplayInfoPanel>,
) {
    if !mouse.just_pressed(MouseButton::Left) || !mouse_info.grid_coords.is_in_bounds(obstacle_grid.bounds()) { return; }

    let mut display_building_info = display_info_panel.single_mut();
    let focused_structure = match &obstacle_grid[mouse_info.grid_coords] {
        Field::Building(building_entity, building_type, _ ) => {
            *display_building_info = DisplayInfoPanel::Building(*building_type, *building_entity);
            next_display_info_panel_state.set(DisplayInfoPanelState::DisplayBuilding);
            *building_entity
        },
        Field::QuantumField(entity) => {
            *display_building_info = DisplayInfoPanel::QuantumField(*entity);
            next_display_info_panel_state.set(DisplayInfoPanelState::DisplayQuantumField);
            *entity
        },
        _ => return,
    };

    ui_map_object_focus_changed_events.send(UiMapObjectFocusChangedEvent::Focus((focused_structure).into()));
    ui_interaction_state.set(UiInteraction::DisplayInfoPanel);
}

fn on_building_destroyed_system(
    mut ui_interaction_state: ResMut<NextState<UiInteraction>>,
    mut events: EventReader<BuildingDestroyedEvent>,
    display_info_panel: Query<&DisplayInfoPanel>,
) {
    let DisplayInfoPanel::Building(_, building_entity) = display_info_panel.single() else { unreachable!() };
    for event in events.read() {
        if event.0 == *building_entity {
            ui_interaction_state.set(UiInteraction::Free);
        }
    }
}

fn update_building_info_panel_system(
    buildings: Query<&Health, With<Building>>,
    display_info_panel: Query<&DisplayInfoPanel>,
    mut healthbar: Query<(&mut Style, &mut BackgroundColor), With<BuildingHealthbar>>,
    mut health_text: Query<&mut Text, With<BuildingHealthbarValue>>,
) {
    let DisplayInfoPanel::Building(_, building_entity) = display_info_panel.single() else { return; };
    let Ok(health) = buildings.get(*building_entity) else { return; };
    // Update the healthbar
    let (mut style, mut background_color) = healthbar.single_mut();
    let health_percentage = health.get_percent();
    style.width = Val::Percent(health_percentage * 100.);
    background_color.0 = Color::linear_rgba(1. - health_percentage, health_percentage, 0., 1.);

    // Update the health text
    health_text.single_mut().sections[0].value = format!("{} / {}", health.get_current(), health.get_max());
}


fn initialize_display_info_panel_system(
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
        DisplayInfoPanelCamera,
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
        DisplayInfoPanel::None,
    )).with_children(|parent| {
        // Camera image (Left side)
        parent.spawn((
            NodeBundle {
                style: Style {
                    min_width: Val::Px(128.0),
                    min_height: Val::Px(128.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                border_color: YELLOW.into(),
                ..default()
            },
            UiImage::new(camera_image_handle),
        ));
        // Right panels
        make_building_panel(parent);
        make_quantum_field_panel(parent);
    });
}

fn make_building_panel(parent: &mut ChildBuilder) {
    parent.spawn((
        NodeBundle {
            style: Style {
                display: Display::None,
                height: Val::Percent(100.),
                width: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Start,
                padding: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            ..default()
        },
        BuildingPanel,
    )).with_children(|parent| {
        // Top line of the panel
        parent.spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Start,
                    ..default()
                },
                ..default()
            },
        )).with_children(|parent| {
            // Building name
            parent.spawn((
                TextBundle {
                    text: Text {
                        sections: vec![TextSection::new("### Building Name ###", TextStyle{ color: BLUE.into(), ..default() })],
                        linebreak_behavior: BreakLineOn::NoWrap,
                        ..default() 
                    },
                    style: Style {
                        margin: UiRect{ left: Val::Px(4.), right: Val::Px(4.), ..default() },
                        ..default()
                    },
                    ..default()
                },
                BuildingNameText,
            ));
            // Health Bar
            parent.spawn((
                // Bottom rectangle(background)
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.),
                        height: Val::Percent(100.),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    background_color: Color::linear_rgba(0., 0., 0., 0.).into(),
                    border_color: Color::linear_rgba(0., 0.2, 1., 1.).into(),
                    ..default()
                },
            )).with_children(|parent| {
                // Top rectangle(health)
                parent.spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.),
                            height: Val::Percent(100.),
                            ..default()
                        },
                        background_color: Color::linear_rgba(0., 1., 0., 1.).into(),
                        ..default()
                    },
                    BuildingHealthbar,
                ));
                // Current hp text
                parent.spawn(NodeBundle {
                    // This additional container is needed to center the text as no combination of flex_direction, justify_content and align_items work
                    style: Style {
                        position_type: PositionType::Absolute,
                        width: Val::Percent(100.),
                        height: Val::Percent(100.),
                        padding: UiRect { top: Val::Px(2.0), ..default() },
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default() 
                }).with_children(|parent| {
                    parent.spawn((
                        TextBundle {
                            text: Text {
                                sections: vec![TextSection::new("### Current Health / Max Health ###", TextStyle{ color: BLACK.into(), font_size: 16.0, ..default() })],
                                linebreak_behavior: BreakLineOn::NoWrap,
                                ..default() 
                            },
                            ..default()
                        },
                        BuildingHealthbarValue,
                    ));
                });
            });
        });
    });

}

fn make_quantum_field_panel(parent: &mut ChildBuilder) {
    parent.spawn((
        NodeBundle {
            style: Style {
                display: Display::None,
                height: Val::Percent(100.),
                width: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Start,
                padding: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            ..default()
        },
        QuantumFieldPanel,
    )).with_children(|parent| {
        // Top line of the panel
        parent.spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Start,
                    ..default()
                },
                ..default()
            },
        )).with_children(|parent| {
            // Building name
            parent.spawn((
                TextBundle {
                    text: Text {
                        sections: vec![TextSection::new("Quantum Field", TextStyle{ color: BLUE.into(), ..default() })],
                        linebreak_behavior: BreakLineOn::NoWrap,
                        ..default() 
                    },
                    style: Style {
                        margin: UiRect{ left: Val::Px(4.), right: Val::Px(4.), ..default() },
                        ..default()
                    },
                    ..default()
                },
            ));
        });
    });
}