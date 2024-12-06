use bevy::color::palettes::css::{AQUA, BLUE, YELLOW};
use bevy::render::camera::RenderTarget;
use bevy::render::render_resource::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};
use bevy::text::LineBreak;
use crate::map_objects::common::ExpeditionTargetMarker;
use crate::map_objects::quantum_field::QuantumField;
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
                show_on_click_system.run_if(in_state(UiInteraction::Free).or(in_state(UiInteraction::DisplayInfoPanel))),
                on_building_destroyed_system.run_if(in_state(UiInteraction::DisplayInfoPanel).and(on_event::<BuildingDestroyedEvent>)),
                update_building_info_panel_system.run_if(in_state(DisplayInfoPanelState::DisplayBuilding)),
                (
                    update_quantum_field_info_panel_system,
                    update_quantum_field_action_button_system.after(update_quantum_field_info_panel_system), // This ordering prevents button flickering
                    on_quantum_field_action_button_click_system.after(update_quantum_field_action_button_system), // This ordering prevents button flickering
                ).run_if(in_state(DisplayInfoPanelState::DisplayQuantumField)),
                on_display_panel_focus_changed_system.run_if(on_event::<UiMapObjectFocusChangedEvent>)
            ))
            .add_systems(OnEnter(UiInteraction::DisplayInfoPanel), on_display_enter_system)
            .add_systems(OnExit(UiInteraction::DisplayInfoPanel), on_display_exit_system)
            .add_systems(OnEnter(DisplayInfoPanelState::DisplayBuilding), on_display_building_enter_system)
            .add_systems(OnExit(DisplayInfoPanelState::DisplayBuilding), on_display_building_exit_system)
            .add_systems(OnEnter(DisplayInfoPanelState::DisplayQuantumField), on_display_quantum_field_enter_system)
            .add_systems(OnExit(DisplayInfoPanelState::DisplayQuantumField), on_display_quantum_field_exit_system);
        app.world_mut().add_observer(on_recreate_quantum_field_panel_trigger);
        app.world_mut().add_observer(on_recreate_building_panel_trigger);
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
/// --- Quantum Fields sub-panel ---
#[derive(Component)]
struct QuantumFieldPanel;
#[derive(Component)]
struct QuantumFieldLayerHealthbar;
#[derive(Component)]
struct QuantumFieldLayerText;
#[derive(Component)]
struct QuantumFieldLayerCostsContainer;
#[derive(Component)]
struct QuantumFieldLayerCostPanel;
#[derive(Component, Default, PartialEq)]
enum QuantumFieldActionButton {
    #[default]
    Hidden,
    SendExpeditions,
    StopExpeditions,
    PayCost,
}
#[derive(Component)]
struct QuantumFieldActionButtonText;
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
#[derive(Event)]
pub struct RecreateQuantumFieldPanelTrigger;
#[derive(Event)]
pub struct RecreateBuildingPanelTrigger;

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
    commands.queue(UiMapObjectFocusChangedEvent::Unfocus);
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
    mut commands: Commands,
    mut building_panel: Query<&mut Node, With<BuildingPanel>>,
) {
    building_panel.single_mut().display = Display::Flex;
    commands.trigger(RecreateBuildingPanelTrigger);
}

fn on_display_building_exit_system(
    mut building_panel: Query<&mut Node, With<BuildingPanel>>,
) {
    building_panel.single_mut().display = Display::None;
}

fn on_display_quantum_field_enter_system(
    mut quantum_field_panel: Query<&mut Node, With<QuantumFieldPanel>>,
) {
    quantum_field_panel.single_mut().display = Display::Flex;
}

fn on_display_quantum_field_exit_system(
    mut quantum_field_panel: Query<&mut Node, With<QuantumFieldPanel>>,
) {
    quantum_field_panel.single_mut().display = Display::None;
}

fn on_display_panel_focus_changed_system(
    mut commands: Commands,
    mut events: EventReader<UiMapObjectFocusChangedEvent>,
    display_info_panel: Query<&DisplayInfoPanel>,
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

    match display_info_panel.single() {
        DisplayInfoPanel::QuantumField(_) => commands.trigger(RecreateQuantumFieldPanelTrigger),
        DisplayInfoPanel::Building(_, _) => commands.trigger(RecreateBuildingPanelTrigger),
        _ => (),
    }
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

fn on_recreate_building_panel_trigger(
    _trigger: Trigger<RecreateBuildingPanelTrigger>,
    almanach: Res<Almanach>,
    display_info_panel: Query<&DisplayInfoPanel>,
    mut building_name_text: Query<&mut Text, With<BuildingNameText>>,
) {
    println!("Recreating building panel");
    let DisplayInfoPanel::Building(building_type, _) = display_info_panel.single() else { return; };
    // Update the building name
    building_name_text.single_mut().0 = almanach.get_building_name(*building_type).to_string();
}

fn on_building_destroyed_system(
    mut ui_interaction_state: ResMut<NextState<UiInteraction>>,
    mut events: EventReader<BuildingDestroyedEvent>,
    display_info_panel: Query<&DisplayInfoPanel>,
) {
    let DisplayInfoPanel::Building(_, building_entity) = display_info_panel.single() else { return; };
    for event in events.read() {
        if event.0 == *building_entity {
            ui_interaction_state.set(UiInteraction::Free);
        }
    }
}

fn update_building_info_panel_system(
    buildings: Query<&Health, With<Building>>,
    display_info_panel: Query<&DisplayInfoPanel>,
    mut healthbars: Query<&mut Healthbar, With<BuildingHealthbar>>,
) {
    let DisplayInfoPanel::Building(_, building_entity) = display_info_panel.single() else { return; };
    let Ok(health) = buildings.get(*building_entity) else { return; };
    // Update the healthbar
    let mut healthbar = healthbars.single_mut();
    healthbar.value = health.get_current() as f32;
    healthbar.max_value = health.get_max() as f32;
    let health_percentage = health.get_percent();
    healthbar.color = Color::linear_rgba(1. - health_percentage, health_percentage, 0., 1.);
}

fn update_quantum_field_info_panel_system(
    quantum_fields: Query<(&QuantumField, Has<ExpeditionTargetMarker>)>,
    display_info_panel: Query<&DisplayInfoPanel>,
    mut action_button: Query<&mut QuantumFieldActionButton>,
    mut healthbars: Query<&mut Healthbar, With<QuantumFieldLayerHealthbar>>,
    mut texts: Query<&mut Text, With<QuantumFieldLayerText>>,
) {
    let DisplayInfoPanel::QuantumField(entity) = display_info_panel.single() else { return; };
    let Ok((quantum_field, has_expedition_target_marker)) = quantum_fields.get(*entity) else { return; };
    let mut healthbar = healthbars.single_mut();
    // Update the layer text
    let mut text = texts.single_mut();
    text.0 = if quantum_field.is_solved() {
        "All Quantum Layers Solved".to_string()
    } else {
        format!("Quantum Layer {}/{}", quantum_field.current_layer + 1, quantum_field.layers.len())
    };
    // Update the layer progress
    let (current_layer_progress, current_layer_target) = quantum_field.get_progress_details();
    healthbar.value = current_layer_progress as f32;
    healthbar.max_value = current_layer_target as f32;
    // Update the action button
    *action_button.single_mut() = {
        if quantum_field.is_solved() {
            QuantumFieldActionButton::Hidden
        } else if quantum_field.is_current_layer_solved() {
            QuantumFieldActionButton::PayCost
        } else if  has_expedition_target_marker {
            QuantumFieldActionButton::StopExpeditions
        } else {
            QuantumFieldActionButton::SendExpeditions
        }
    };
}

fn on_recreate_quantum_field_panel_trigger(
    _trigger: Trigger<RecreateQuantumFieldPanelTrigger>,
    mut commands: Commands,
    display_info_panel: Query<&DisplayInfoPanel>,
    quantum_fields: Query<&QuantumField>,
    costs_container: Query<Entity, With<QuantumFieldLayerCostsContainer>>,
    costs_panels: Query<Entity, With<QuantumFieldLayerCostPanel>>,
) {
    let DisplayInfoPanel::QuantumField(quantum_field_entity) = display_info_panel.single() else { return; };
    // Remove the old panels
    costs_panels.iter().for_each(|entity| commands.entity(entity).despawn_recursive());

    // Create the new panels
    let costs_container_entity = costs_container.single();
    let Ok(quantum_field) = quantum_fields.get(*quantum_field_entity) else { return; };
    for cost in quantum_field.get_current_layer_costs() {
        commands.entity(costs_container_entity).with_children(|parent| {
            parent.spawn((
                Node {
                    margin: UiRect{ top: Val::Px(4.), bottom: Val::Px(4.), ..default() },
                    ..default()
                },
                CostIndicator {
                    cost: *cost,
                    ..default()
                },
                QuantumFieldLayerCostPanel,
            ));
        });
    }
}

fn update_quantum_field_action_button_system(
    mut action_button: Query<(&QuantumFieldActionButton, &mut Node)>,
    mut action_button_text: Query<&mut Text, With<QuantumFieldActionButtonText>>,
) {
    let (action_button, mut style) = action_button.single_mut();
    let mut text = action_button_text.single_mut();
    match action_button {
        QuantumFieldActionButton::SendExpeditions => {
            text.0 = "Send Expeditions".to_string();
            style.display = Display::Flex;
        },
        QuantumFieldActionButton::StopExpeditions => {
            text.0 = "Stop Expeditions".to_string();
            style.display = Display::Flex;
        },
        QuantumFieldActionButton::PayCost => {
            text.0 = "Pay Cost".to_string();
            style.display = Display::Flex;
        },
        QuantumFieldActionButton::Hidden => {
            style.display = Display::None;
        },
    }
}

fn on_quantum_field_action_button_click_system(
    mut commands: Commands,
    mut stock: ResMut<Stock>,
    display_info_panel: Query<&DisplayInfoPanel>,
    mut action_button: Query<(&mut QuantumFieldActionButton, &AdvancedInteraction)>,
    mut quantum_fields: Query<&mut QuantumField>,
) {
    let DisplayInfoPanel::QuantumField(entity) = display_info_panel.single() else { return; };
    let (mut action_button, interaction) = action_button.single_mut();
    if interaction.was_just_released {
        match *action_button {
            QuantumFieldActionButton::SendExpeditions => {
                commands.entity(*entity).insert(ExpeditionTargetMarker);
            },
            QuantumFieldActionButton::StopExpeditions => {
                commands.entity(*entity).remove::<ExpeditionTargetMarker>();
            },
            QuantumFieldActionButton::PayCost => {
                let Ok(mut quantum_field) = quantum_fields.get_mut(*entity) else { return; };
                if stock.try_pay_costs(quantum_field.get_current_layer_costs()) {
                    quantum_field.move_to_next_layer();
                    commands.trigger(RecreateQuantumFieldPanelTrigger);
                }
            },
            QuantumFieldActionButton::Hidden => {},
        }
        *action_button = QuantumFieldActionButton::Hidden; // To make sure no multi-trigger occurs
    }
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
        Camera2d::default(),
        Camera {
            order: 1,
            hdr: true,
            target: RenderTarget::Image(camera_image_handle.clone()),
            is_active: false,
            ..default()
        },
        OrthographicProjection {
            near: -1000.,
            far: 1000.,
            scale: 2., // TODO, check new scaling_mode
            ..OrthographicProjection::default_2d()
        },
        DisplayInfoPanelCamera,
    ));
    commands.spawn((
        Node {
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
        BackgroundColor::from(Color::linear_rgba(0.46, 0.62, 0.67, 1.)),
        BorderColor::from(Color::linear_rgba(0., 0.2, 1., 1.)),
        BorderRadius::all(Val::Px(7.)),
        Visibility::Hidden,
        DisplayInfoPanel::None,
    )).with_children(|parent| {
        // Camera image (Left side)
        parent.spawn((
            Node {
                min_width: Val::Px(128.0),
                min_height: Val::Px(128.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor::from(YELLOW),
            ImageNode::new(camera_image_handle),
        ));
        // Right panels
        make_building_panel(parent);
        make_quantum_field_panel(parent);
    });
}

fn make_building_panel(parent: &mut ChildBuilder) {
    parent.spawn((
        Node {
            display: Display::None,
            height: Val::Percent(100.),
            width: Val::Percent(100.),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Start,
            padding: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BuildingPanel,
    )).with_children(|parent| {
        // Top line of the panel
        parent.spawn((
            Node {
                width: Val::Percent(100.),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Start,
                ..default()
            },
        )).with_children(|parent| {
            // Building name
            parent.spawn((
                Text::new("### Building Name ###"),
                TextColor::from(BLUE),
                TextLayout::new_with_linebreak(LineBreak::NoWrap),
                Node {
                    margin: UiRect{ left: Val::Px(4.), right: Val::Px(4.), ..default() },
                    ..default()
                },
                BuildingNameText,
            ));
            // Healthbar
            parent.spawn((
                Node {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    ..default()
                },
                Healthbar::default(),
                BuildingHealthbar,
            ));
        });
    });

}

fn make_quantum_field_panel(parent: &mut ChildBuilder) {
    parent.spawn((
        Node {
            display: Display::None,
            height: Val::Percent(100.),
            width: Val::Percent(100.),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Start,
            padding: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        QuantumFieldPanel,
    )).with_children(|parent| {
        // Top line of the panel
        parent.spawn((
            Node {
                width: Val::Percent(100.),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Start,
                ..default()
            },
        )).with_children(|parent| {
            // Structure name
            parent.spawn((
                Text::new("Quantum Field"),
                TextColor::from(BLUE),
                TextLayout::new_with_linebreak(LineBreak::NoWrap),
                Node {
                    margin: UiRect{ left: Val::Px(4.), right: Val::Px(4.), ..default() },
                    ..default()
                },
            ));
        });
        // Panel Body
        parent.spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            //BackgroundColor::from(Color::linear_rgba(0., 0., 0., 0.)),
            //BorderColor::from(Color::linear_rgba(0., 0.2, 1., 1.)),
        )).with_children(|parent| {
            parent.spawn(Node {
                width: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                ..default()
            }).with_children(|parent| {
                parent.spawn((
                    Text::new("Quantum Layer #/#"),
                    TextColor::from(BLUE),
                    TextFont::default().with_font_size(16.0),
                    QuantumFieldLayerText,
                ));
            });
            parent.spawn((
                Node {
                    top: Val::Px(2.0),
                    width: Val::Percent(60.),
                    height: Val::Px(20.),
                    ..default()
                },
                Healthbar {
                    color: AQUA.into(),
                    ..default()
                },
                QuantumFieldLayerHealthbar,
            ));
            // Costs Panel - content is dynamic and managed from a dedicated system
            parent.spawn((
                Node {
                    width: Val::Percent(100.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                QuantumFieldLayerCostsContainer,
            ));
            // [Send Expeditions / Stop Expeditions / Pay Cost] Button.
            parent.spawn((
                Button::default(),
                Node {
                    width: Val::Percent(50.),
                    height: Val::Px(20.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor::from(Color::linear_rgba(0., 0., 0.2, 0.2)),
                BorderColor::from(Color::linear_rgba(0., 0.2, 1., 1.)),
                AdvancedInteraction::default(),
                QuantumFieldActionButton::default(),
            )).with_children(|parent| {
                parent.spawn((
                    Text::new("Send Expeditions / Stop Expeditions / Pay cost"),
                    TextColor::from(BLUE),
                    TextFont::default().with_font_size(12.0),
                    QuantumFieldActionButtonText,
                ));
            });
        });
    });
}