use bevy::color::palettes::css::{AQUA, BLUE, INDIGO};

use lib_grid::grids::obstacles::{Field, ObstacleGrid};

use crate::prelude::*;
use crate::map_objects::common::ExpeditionZone;
use crate::ui::common::AdvancedInteraction;
use crate::ui::display_info_panel::{DisplayInfoPanel, DisplayPanelMainContentRoot, UiMapObjectFocusedTrigger};
use crate::ui::grid_object_placer::{GridObjectPlacer, GridObjectPlacerRequest};

use super::common::ExpeditionTargetMarker;

pub struct QuantumFieldPlugin;
impl Plugin for QuantumFieldPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<BuilderQuantumField>()
            .add_systems(PostUpdate, (
                BuilderQuantumField::spawn_system,
            ))
            .add_systems(Startup, (
                create_grid_placer_ui_for_quantum_field_system,
            ))
            .add_systems(PostStartup, (
                initialize_quantum_field_panel_content_system,
            ))
            .add_systems(Update, (
                onclick_spawn_system,
                operate_arrows_for_grid_placer_ui_for_quantum_field_system,
                process_expeditions_system.run_if(in_state(GameState::Running)),
                (
                    update_quantum_field_info_panel_system,
                    update_quantum_field_action_button_system.after(update_quantum_field_info_panel_system), // This ordering prevents button flickering
                    on_quantum_field_action_button_click_system.after(update_quantum_field_action_button_system), // This ordering prevents button flickering
                ).run_if(in_state(UiInteraction::DisplayInfoPanel)),
            ));
        app.add_observer(on_ui_map_object_focus_changed_trigger);
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct QuantumFieldImprintSelector(i32);
impl QuantumFieldImprintSelector {
    pub const MIN_IMPRINT_SIZE: i32 = 3;
    pub const MAX_IMPRINT_SIZE: i32 = 6;
    pub fn get_size(&self) -> i32 {
        self.0
    }
    pub fn get(&self) -> GridImprint {
        GridImprint::Rectangle { width: self.0, height: self.0 }
    }
    pub fn set(&mut self, new_size: i32) -> Result<(), String> {
        if new_size >= Self::MIN_IMPRINT_SIZE && new_size <= Self::MAX_IMPRINT_SIZE {
            self.0 = new_size;
            Ok(())
        } else {
            Err(format!("Quantum field imprint size must be between {} and {}", Self::MIN_IMPRINT_SIZE, Self::MAX_IMPRINT_SIZE))
        }
    }
    pub fn increase(&mut self) -> Result<(), String> {
        self.set(self.0 + 1)
    }
    pub fn decrease(&mut self) -> Result<(), String> {
        self.set(self.0 - 1)
    }
}
impl Default for QuantumFieldImprintSelector {
    fn default() -> Self {
        Self(Self::MIN_IMPRINT_SIZE)
    }
}

// Marks QuantumField with all its layers solved
#[derive(Component)]
struct Solved;

#[derive(Component)]
pub struct QuantumField {
    pub layers: Vec<QuantumFieldLayer>,
    pub current_layer: usize,
    pub current_layer_progress: i32,
}
impl QuantumField {
    pub fn progress_layer(&mut self, amount: i32) {
        if self.is_solved() { return; }
        self.current_layer_progress = std::cmp::min(self.current_layer_progress + amount, self.layers[self.current_layer].value);
    }
    pub fn move_to_next_layer(&mut self) {
        if self.is_solved() { return; }
        self.current_layer += 1;
        self.current_layer_progress = 0;
    }
    /// Returns (current_layer_progress, current_layer_target)
    pub fn get_progress_details(&self) -> (i32, i32) {
        if self.is_solved() { return (0, 0); }
        (self.current_layer_progress, self.layers[self.current_layer].value)
    }
    pub fn is_solved(&self) -> bool {
        self.current_layer == self.layers.len()
    }
    pub fn is_current_layer_solved(&self) -> bool {
        self.current_layer_progress >= self.layers[self.current_layer].value
    }
    pub fn get_current_layer_costs(&self) -> &[Cost] {
        if self.is_solved() { return &[]; }
        &self.layers[self.current_layer].costs
    }
}

// Describes a layer of QuantumField to solve
// `value` - amount of research needed to solve the layer
// `costs` - costs needed to pay after solving the layer to finalize it
pub struct QuantumFieldLayer {
    pub value: i32,
    pub costs: Vec<Cost>,
}

#[derive(Event)]
pub struct BuilderQuantumField {
    pub entity: Entity,
    pub grid_position: GridCoords,
    pub grid_imprint: GridImprint,
}
impl BuilderQuantumField {
    pub fn new(entity: Entity, grid_position: GridCoords, grid_imprint: GridImprint) -> Self {
        Self { entity, grid_position, grid_imprint }
    }
    pub fn spawn_system(
        mut commands: Commands,
        mut events: EventReader<BuilderQuantumField>,
    ) {
        for &BuilderQuantumField{ entity, grid_position, grid_imprint } in events.read() {
            commands.entity(entity).insert((
                Sprite {
                    custom_size: Some(grid_imprint.world_size()),
                    color: INDIGO.into(),
                    ..Default::default()
                },
                Transform::from_translation(grid_position.to_world_position_centered(grid_imprint).extend(Z_OBSTACLE)),
                grid_position,
                grid_imprint,
                QuantumField {
                    current_layer: 0,
                    current_layer_progress: 0,
                    layers: vec![
                        QuantumFieldLayer {
                            value: 15000,
                            costs: vec![Cost{ resource_type: ResourceType::DarkOre, amount: 100}, Cost{ resource_type: ResourceType::DarkOre, amount: 100}, Cost{ resource_type: ResourceType::DarkOre, amount: 100}],
                        },
                        QuantumFieldLayer {
                            value: 30000,
                            costs: vec![Cost{ resource_type: ResourceType::DarkOre, amount: 200}],
                        },
                        QuantumFieldLayer {
                            value: 45000,
                            costs: vec![Cost{ resource_type: ResourceType::DarkOre, amount: 300}],
                        },
                    ],
                },
                ExpeditionZone::default(),
            ));
        }
    }
}
impl Command for BuilderQuantumField {
    fn apply(self, world: &mut World) {
        world.send_event(self);
    }
}

fn remove_quantum_field(
    commands: &mut Commands,
    obstacle_grid: &mut ResMut<ObstacleGrid>,
    entity: Entity,
    grid_position: GridCoords,
    grid_imprint: GridImprint,
) {
    commands.entity(entity).despawn();
    obstacle_grid.deprint_all(grid_position, grid_imprint);
}

fn onclick_spawn_system(
    mut commands: Commands,
    mut obstacles_grid: ResMut<ObstacleGrid>,
    mouse: Res<ButtonInput<MouseButton>>,
    mouse_info: Res<MouseInfo>,
    grid_object_placer: Query<&GridObjectPlacer>,
    quantum_fields_query: Query<&GridCoords, With<QuantumField>>,
) {
    let GridObjectPlacer::QuantumField(imprint_selector) = grid_object_placer.single().unwrap() else { return; };
    let grid_imprint = imprint_selector.get();
    let mouse_coords = mouse_info.grid_coords;
    if mouse_info.is_over_ui || !mouse_coords.is_in_bounds(obstacles_grid.bounds()) { return; }
    if mouse.pressed(MouseButton::Left) {
        // Place a quantum_field
        if obstacles_grid.imprint_query_all(mouse_coords, grid_imprint, |field| field.is_empty()) {
            let quantum_field_entity = commands.spawn_empty().id();
            commands.queue(BuilderQuantumField::new(quantum_field_entity, mouse_coords, grid_imprint));
            obstacles_grid.imprint(mouse_coords, Field::QuantumField(quantum_field_entity), grid_imprint);
        }
    } else if mouse.pressed(MouseButton::Right) {
        // Remove a quantum_field
        match obstacles_grid[mouse_coords] {
            Field::QuantumField(entity) => {
                if let Ok(quantum_field_coords) = quantum_fields_query.get(entity) {
                    remove_quantum_field(&mut commands, &mut obstacles_grid, entity, *quantum_field_coords, grid_imprint);
                }
            },
            _ => {}
        }
    }
}

fn process_expeditions_system(
    mut commands: Commands,
    mut quantum_fields: Query<(Entity, &mut QuantumField, &mut ExpeditionZone), (Changed<ExpeditionZone>, Without<Solved>)>,
) {
    for (entity, mut quantum_field, mut expedition_zone) in quantum_fields.iter_mut() {
        while expedition_zone.expeditions_arrived > 0 {
            expedition_zone.expeditions_arrived -= 1;
            quantum_field.progress_layer(1500); // TODO: It should come from Almanach
            if quantum_field.is_solved() {
                commands.entity(entity).insert(Solved);
            }
        }
    }
}

/// Widget for selecting QuantumField grid imprint size during construction
/// The widget consists of one horizontal layer containing left arrow button, text label specifying the imprint size and right arrow button
#[derive(Component, Default)]
pub struct GridPlacerUiForQuantumField {
    pub imprint_selector: QuantumFieldImprintSelector,
}
impl GridPlacerUiForQuantumField {
    pub fn imprint_str(&self) -> String {
        let size = self.imprint_selector.get_size();
        format!("Quantum Field {}x{}", size, size)
    }
}
#[derive(Component)]
pub enum ArrowButton {
    Decrease,
    Increase,
}
impl ArrowButton {
    fn text(&self) -> &str {
        match self {
            ArrowButton::Decrease => "<",
            ArrowButton::Increase => ">",
        }
    }
}

pub fn create_grid_placer_ui_for_quantum_field_system(
    mut commands: Commands,
) {
    struct ArrowButtonBundle {
        button: Button,
        node: Node,
        background_color: BackgroundColor,
        z_index: GlobalZIndex,
        text: Text,
        arrow_button: ArrowButton,
        advanced_interaction: AdvancedInteraction,
    }
    impl ArrowButtonBundle {
        fn new(arrow_button: ArrowButton) -> Self {
            Self {
                button: Button::default(),
                node: Node {
                    width: Val::Px(16.),
                    height: Val::Px(16.),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                background_color: Color::BLACK.into(),
                z_index: GlobalZIndex(-1),
                text: Text::new(arrow_button.text()),
                arrow_button,
                advanced_interaction: Default::default(),
            }
        }
        pub fn spawn(self, spawner: &mut ChildSpawnerCommands) {
            spawner.spawn((self.button, self.node, self.background_color, self.z_index, self.arrow_button, self.advanced_interaction)).with_children(|parent| {
                parent.spawn((self.text, TextFont::default().with_font_size(12.)));
            });
        }
    }

    let grid_placer_ui_for_quantum_field = GridPlacerUiForQuantumField::default();
    let ui_text = grid_placer_ui_for_quantum_field.imprint_str();
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Percent(50.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(2.5),
            ..default()
        },
        grid_placer_ui_for_quantum_field,
    )).with_children(|parent| {
        ArrowButtonBundle::new(ArrowButton::Decrease).spawn(parent);
        parent.spawn(Text::new(ui_text));
        ArrowButtonBundle::new(ArrowButton::Increase).spawn(parent);
    });
}

pub fn operate_arrows_for_grid_placer_ui_for_quantum_field_system(
    mut ui: Query<(&mut Visibility, &Children, &mut GridPlacerUiForQuantumField)>,
    mut arrows: Query<(&ArrowButton, &AdvancedInteraction)>,
    mut texts: Query<&mut Text>,
    grid_object_placer: Query<&GridObjectPlacer>,
    mut placer_request: ResMut<GridObjectPlacerRequest>,
) {
    let Ok((mut visibility, ui_children, mut grid_placer_ui)) = ui.single_mut() else { return; };
    let GridObjectPlacer::QuantumField(_) = grid_object_placer.single().unwrap() else {
        *visibility = Visibility::Hidden;
        return;
    };
    *visibility = Visibility::Inherited;

    for (arrow_button, advanced_interaction) in arrows.iter_mut() {
        if advanced_interaction.was_just_released {
            match arrow_button {
                ArrowButton::Decrease => { let _ = grid_placer_ui.imprint_selector.decrease();},
                ArrowButton::Increase => { let _ = grid_placer_ui.imprint_selector.increase();},
            }

            let ui_text = grid_placer_ui.imprint_str();
            texts.get_mut(ui_children[1]).unwrap().0 = ui_text;
            placer_request.set(GridObjectPlacer::QuantumField(grid_placer_ui.imprint_selector));
        }
    }
}

////////////////////////////////////////////
////        Display Info Panel          ////
////////////////////////////////////////////
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


fn update_quantum_field_info_panel_system(
    quantum_fields: Query<(&QuantumField, Has<ExpeditionTargetMarker>)>,
    display_info_panel: Query<&DisplayInfoPanel>,
    mut action_button: Query<&mut QuantumFieldActionButton>,
    mut healthbars: Query<&mut Healthbar, With<QuantumFieldLayerHealthbar>>,
    mut texts: Query<&mut Text, With<QuantumFieldLayerText>>,
) {
    let focused_entity = display_info_panel.single().unwrap().current_focus;
    let Ok((quantum_field, has_expedition_target_marker)) = quantum_fields.get(focused_entity) else { return; };
    let Ok(mut healthbar) = healthbars.single_mut() else { return; };
    // Update the layer text
    let Ok(mut text) = texts.single_mut() else { return; };
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
    *action_button.single_mut().unwrap() = {
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

fn on_ui_map_object_focus_changed_trigger(
    trigger: Trigger<UiMapObjectFocusedTrigger>,
    mut commands: Commands,
    quantum_fields: Query<&QuantumField>,
    mut quantum_field_panel: Query<&mut Node, With<QuantumFieldPanel>>,
    costs_container: Query<Entity, With<QuantumFieldLayerCostsContainer>>,
    costs_panels: Query<Entity, With<QuantumFieldLayerCostPanel>>,
) {
    let focused_entity = trigger.target();
    let Ok(quantum_field) = quantum_fields.get(focused_entity) else { 
        quantum_field_panel.single_mut().unwrap().display = Display::None;
        return;
     };
    quantum_field_panel.single_mut().unwrap().display = Display::Flex;

    // Remove the old panels
    costs_panels.iter().for_each(|entity| commands.entity(entity).despawn());

    // Create the new panels
    let Ok(costs_container_entity) = costs_container.single() else { return; };
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
    let Ok((action_button, mut style)) = action_button.single_mut() else { return; };
    let Ok(mut text) = action_button_text.single_mut() else { return; };
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
    let focused_entity = display_info_panel.single().unwrap().current_focus;
    let Ok((mut action_button, interaction)) = action_button.single_mut() else { return; };
    if interaction.was_just_released {
        match *action_button {
            QuantumFieldActionButton::SendExpeditions => {
                commands.entity(focused_entity).insert(ExpeditionTargetMarker);
            },
            QuantumFieldActionButton::StopExpeditions => {
                commands.entity(focused_entity).remove::<ExpeditionTargetMarker>();
            },
            QuantumFieldActionButton::PayCost => {
                let Ok(mut quantum_field) = quantum_fields.get_mut(focused_entity) else { return; };
                if stock.try_pay_costs(quantum_field.get_current_layer_costs()) {
                    quantum_field.move_to_next_layer();
                    commands.trigger_targets(UiMapObjectFocusedTrigger, [focused_entity]);
                }
            },
            QuantumFieldActionButton::Hidden => {},
        }
        *action_button = QuantumFieldActionButton::Hidden; // To make sure no multi-trigger occurs
    }
}


fn initialize_quantum_field_panel_content_system(
    mut commands: Commands,
    display_info_panel_main_content_root: Query<Entity, With<DisplayPanelMainContentRoot>>,
) {
    let Ok(display_info_panel_main_content_root) = display_info_panel_main_content_root.single() else { return; };
    commands.entity(display_info_panel_main_content_root).with_children(|parent| {
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
    });
}