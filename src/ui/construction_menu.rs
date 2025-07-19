use bevy::color::palettes::css::{TURQUOISE, WHITE};
use bevy::ui::FocusPolicy;

use lib_ui::prelude::AdvancedInteraction;

use crate::buildings::tower_emitter::TOWER_EMITTER_BASE_IMAGE;
use crate::prelude::*;
use crate::buildings::energy_relay::ENERGY_RELAY_BASE_IMAGE;
use crate::buildings::exploration_center::EXPLORATION_CENTER_BASE_IMAGE;
use crate::buildings::main_base::MAIN_BASE_BASE_IMAGE;
use crate::buildings::mining_complex::MINING_COMPLEX_BASE_IMAGE;
use crate::buildings::tower_blaster::TOWER_BLASTER_BASE_IMAGE;
use crate::buildings::tower_cannon::TOWER_CANNON_BASE_IMAGE;
use crate::buildings::tower_rocket_launcher::TOWER_ROCKET_LAUNCHER_BASE_IMAGE;
use crate::map_objects::dark_ore::DARK_ORE_BASE_IMAGES;
use crate::map_objects::quantum_field::QuantumFieldImprintSelector;
use crate::ui::grid_object_placer::{GridObjectPlacer, GridObjectPlacerRequest};

const NOT_HOVERED_ALPHA: f32 = 0.2;
const CONSTRUCT_MENU_BUTTON_WIDTH: f32 = 65.;
const CONSTRUCT_MENU_BUTTON_HEIGHT: f32 = 64.;

pub struct ConstructionMenuPlugin;
impl Plugin for ConstructionMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, (
                initialize_construction_menu_system,
            ))
            .add_systems(Update, (
                menu_activation_system,
                construct_building_on_click_system,
            ))
            .add_observer(ConstructObjectButton::on_add)
            .add_observer(ButtonConstructMenu::on_add)
            .add_observer(ConstructMenuListPicker::on_add);
    }
}

#[derive(Component)]
#[require(Button)]
pub struct ButtonConstructMenu(pub &'static str);
impl ButtonConstructMenu {
    pub fn on_add(
        trigger: Trigger<OnAdd, ButtonConstructMenu>,
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        buttons: Query<&ButtonConstructMenu>,
    ) {
        let entity = trigger.target();
        let image = buttons.get(entity).unwrap().0;

        commands.entity(entity).insert((
            Node {
                width: Val::Px(CONSTRUCT_MENU_BUTTON_WIDTH),
                height: Val::Px(CONSTRUCT_MENU_BUTTON_HEIGHT),
                ..default()
            },
            ImageNode::new(asset_server.load(image)).with_color(WHITE.with_alpha(NOT_HOVERED_ALPHA).into()),
        ));
    }
}

#[derive(Component, Default)]
#[require(Button)]
pub struct ConstructMenuListPicker;
impl ConstructMenuListPicker {
    pub fn on_add(
        trigger: Trigger<OnAdd, ConstructMenuListPicker>,
        mut commands: Commands,
    ) {
        let entity = trigger.target();
        commands.entity(entity).insert((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                left: Val::Px(65.),
                padding: UiRect {
                    left: Val::Px(2.5),
                    right: Val::Px(2.5),
                    top: Val::ZERO,
                    bottom: Val::ZERO,
                },
                ..default()
            },
            BackgroundColor(Color::BLACK.into()),
            GlobalZIndex(-1),
        ));
    }
}

#[derive(Component)]
#[require(Button, FocusPolicy, AdvancedInteraction)]
pub struct ConstructObjectButton {
    pub object_type: GridObjectPlacer,
}
impl ConstructObjectButton{
    pub fn new(object_type: GridObjectPlacer) -> Self {
        Self { object_type }
    }

    pub fn on_add(
        trigger: Trigger<OnAdd, ConstructObjectButton>,
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        builders: Query<&ConstructObjectButton>,
    ) {
        let entity = trigger.target();

        commands.entity(entity)
            .insert((
                Node {
                    width: Val::Px(48.),
                    height: Val::Px(48.),
                    margin: UiRect {
                        left: Val::Px(2.5),
                        right: Val::Px(2.5),
                        top: Val::ZERO,
                        bottom: Val::ZERO,
                    },
                    ..default()
                },
                BackgroundColor(TURQUOISE.into()),
            ))
            .with_children(|parent| {
                let object_type = &builders.get(entity).unwrap().object_type;
                let image_handle = match &object_type {
                    GridObjectPlacer::Building(building_type) => match building_type {
                        BuildingType::Tower(tower_type) => {
                            match tower_type {
                                TowerType::Blaster => Some(TOWER_BLASTER_BASE_IMAGE),
                                TowerType::Cannon => Some(TOWER_CANNON_BASE_IMAGE),
                                TowerType::RocketLauncher => Some(TOWER_ROCKET_LAUNCHER_BASE_IMAGE),
                                TowerType::Emitter => Some(TOWER_EMITTER_BASE_IMAGE),
                            }
                        },
                        BuildingType::MainBase => Some(MAIN_BASE_BASE_IMAGE),
                        BuildingType::EnergyRelay => Some(ENERGY_RELAY_BASE_IMAGE),
                        BuildingType::ExplorationCenter => Some(EXPLORATION_CENTER_BASE_IMAGE),
                        BuildingType::MiningComplex => Some(MINING_COMPLEX_BASE_IMAGE),
                    },
                    GridObjectPlacer::DarkOre => Some(DARK_ORE_BASE_IMAGES[0]),
                    _ => None,
                };
                if let Some(image_handle) = image_handle {
                    parent.spawn((
                        Node {
                            width: Val::Px(48.0),
                            height: Val::Px(48.0),
                            ..default()
                        },
                        ImageNode::new(asset_server.load(image_handle)),
                    ));
                }
            });
    }
}

pub fn create_construct_menu(
    commands: &mut Commands,
) -> Entity {
    commands.spawn((
        Node { // Main Menu node
            position_type: PositionType::Absolute,
            top: Val::Percent(40.),
            left: Val::Px(5.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        },
        children![
            // Construct towers button
            (
                ButtonConstructMenu("ui/construct_towers.png"),
                // Construct towers list picker
                children![(
                    ConstructMenuListPicker,
                    children![
                        // Specific tower to construct
                        ConstructObjectButton::new(BuildingType::Tower(TowerType::Blaster).into()),
                        ConstructObjectButton::new(BuildingType::Tower(TowerType::Cannon).into()),
                        ConstructObjectButton::new(BuildingType::Tower(TowerType::RocketLauncher).into()),
                        ConstructObjectButton::new(BuildingType::Tower(TowerType::Emitter).into()),
                    ]
                )]
            ),
            // Construct buildings button
            (
                ButtonConstructMenu("ui/construct_buildings.png"),
                // Construct buildings list picker
                children![(
                    ConstructMenuListPicker,
                    children![
                        // Specific building to construct
                        ConstructObjectButton::new(BuildingType::EnergyRelay.into()),
                        ConstructObjectButton::new(BuildingType::MiningComplex.into()),
                        ConstructObjectButton::new(BuildingType::ExplorationCenter.into()),
                    ]
                )]
            ),
            // Construct objects(editor) button
            (
                ButtonConstructMenu("ui/construct_editor.png"),
                // Construct objects(editor) list picker
                children![(
                    ConstructMenuListPicker,
                    children![
                        // Specific editor building to construct
                        ConstructObjectButton::new(BuildingType::MainBase.into()),
                        ConstructObjectButton::new(GridObjectPlacer::DarkOre),
                        ConstructObjectButton::new(GridObjectPlacer::Wall),
                        ConstructObjectButton::new(GridObjectPlacer::QuantumField(QuantumFieldImprintSelector::default())),
                    ]
                )]
            ),
        ]
    )).id()
}

fn initialize_construction_menu_system(
    mut commands: Commands,
) {
    create_construct_menu(&mut commands);
}

fn menu_activation_system(
    mouse_info: Res<MouseInfo>,
    mut menu_buttons: Query<(&Interaction, &mut ImageNode, &Children, &GlobalTransform), With<ButtonConstructMenu>>,
    mut list_pickers: Query<(&Interaction, &mut Visibility, &ViewVisibility), With<ConstructMenuListPicker>>,
) {
    for (menu_interaction, mut ui_image, children, button_transform) in menu_buttons.iter_mut() {
        let list_picker_entity = children.get(0).unwrap();
        let (list_picker_interaction, mut list_picker_visibility, list_picker_is_visible) = list_pickers.get_mut(*list_picker_entity).unwrap();
        if !matches!(menu_interaction, Interaction::None)
            || !matches!(list_picker_interaction, Interaction::None)
            // If the list picker is already visible, give it some leeway so it does not disappear when player moves mouse from the button to the list picker
            || (list_picker_is_visible.get() && get_extended_construct_button_world_rect(button_transform.translation()).contains(mouse_info.screen_position)) 
        {
            ui_image.color.set_alpha(1.);
            *list_picker_visibility = Visibility::Inherited;
        } else {
            ui_image.color.set_alpha(NOT_HOVERED_ALPHA);
            *list_picker_visibility = Visibility::Hidden;
        }
    }
}

fn construct_building_on_click_system(
    mut grid_object_placer_request: ResMut<GridObjectPlacerRequest>,
    mut menu_buttons: Query<(&AdvancedInteraction, &ConstructObjectButton), Changed<AdvancedInteraction>>,
    mut list_pickers: Query<(&mut Interaction, &mut Visibility), With<ConstructMenuListPicker>>,
) {
    for (advanced_interaction, button) in menu_buttons.iter_mut() {
        if advanced_interaction.was_just_released {
            grid_object_placer_request.set(button.object_type.clone());
            list_pickers.iter_mut().for_each(|(mut interaction, mut visibility)| { *visibility = Visibility::Hidden; *interaction = Interaction::None; });
        }
    }
}

// Helper function to get the construct button rect that is elongated to the right so when player hovers from it to the list picker it stays visible
fn get_extended_construct_button_world_rect(translation: Vec3) -> Rect {
    Rect::new(
        translation.x - CONSTRUCT_MENU_BUTTON_WIDTH / 2.,
        translation.y - CONSTRUCT_MENU_BUTTON_HEIGHT / 2.,
        translation.x + CONSTRUCT_MENU_BUTTON_WIDTH / 2. + 20., 
        translation.y + CONSTRUCT_MENU_BUTTON_HEIGHT / 2.,
    )
}