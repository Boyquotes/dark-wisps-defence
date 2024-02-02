use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use crate::buildings::common::{BuildingType, TowerType};
use crate::buildings::energy_relay::ENERGY_RELAY_BASE_IMAGE;
use crate::buildings::exploration_center::EXPLORATION_CENTER_BASE_IMAGE;
use crate::buildings::main_base::MAIN_BASE_BASE_IMAGE;
use crate::buildings::mining_complex::MINING_COMPLEX_BASE_IMAGE;
use crate::buildings::tower_blaster::TOWER_BLASTER_BASE_IMAGE;
use crate::buildings::tower_cannon::TOWER_CANNON_BASE_IMAGE;
use crate::buildings::tower_rocket_launcher::TOWER_ROCKET_LAUNCHER_BASE_IMAGE;
use crate::map_objects::dark_ore::DARK_ORE_BASE_IMAGE;
use crate::map_objects::quantum_field::QuantumField;
use crate::ui::common::AdvancedInteraction;
use crate::ui::grid_object_placer::{GridObjectPlacer, GridObjectPlacerRequest};

const NOT_HOVERED_ALPHA: f32 = 0.2;

#[derive(Component, Default)]
pub struct ConstructMenuButton {
    pub is_hovered: bool,
}
#[derive(Bundle, Default)]
pub struct ConstructButtonBundle {
    pub button: ButtonBundle,
    pub construct_menu_button: ConstructMenuButton,
}
impl ConstructButtonBundle {
    pub fn new(image: Handle<Image>) -> Self {
        Self {
            button: ButtonBundle {
                style: Style {
                    width: Val::Px(65.),
                    height: Val::Px(64.),
                    ..Default::default()
                },
                background_color: (*Color::WHITE.set_a(NOT_HOVERED_ALPHA)).into(),
                image: UiImage::new(image),
                ..Default::default()
            },
            construct_menu_button: ConstructMenuButton::default(),
        }
    }
}

#[derive(Component, Default)]
pub struct ConstructMenuListPicker {
    pub is_hovered: bool,
}
#[derive(Bundle, Default)]
pub struct ConstructMenuListPickerBundle {
    pub button: ButtonBundle,
    pub construct_menu_list_picker: ConstructMenuListPicker,
}
impl ConstructMenuListPickerBundle {
    pub fn new() -> Self {
        Self {
            button: ButtonBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    left: Val::Px(65.),
                    padding: UiRect {
                        left: Val::Px(2.5),
                        right: Val::Px(2.5),
                        top: Val::ZERO,
                        bottom: Val::ZERO,
                    },
                    ..Default::default()
                },
                background_color: Color::BLACK.into(),
                z_index: ZIndex::Global(-1),
                ..Default::default()
            },
            construct_menu_list_picker: ConstructMenuListPicker::default(),
        }
    }
}

#[derive(Component)]
pub struct ConstructObjectButton {
    pub object_type: GridObjectPlacer,
}
#[derive(Bundle)]
pub struct ConstructObjectButtonBundle {
    pub button: ButtonBundle,
    pub construct_tower_button: ConstructObjectButton,
    pub advanced_interaction: AdvancedInteraction,
}
impl ConstructObjectButtonBundle {
    pub fn new(grid_object_placer: GridObjectPlacer) -> Self {
        Self {
            button: ButtonBundle {
                style: Style {
                    width: Val::Px(48.),
                    height: Val::Px(48.),
                    margin: UiRect {
                        left: Val::Px(2.5),
                        right: Val::Px(2.5),
                        top: Val::ZERO,
                        bottom: Val::ZERO,
                    },
                    ..Default::default()
                },
                background_color: Color::TURQUOISE.into(),
                focus_policy: FocusPolicy::Pass,
                ..Default::default()
            },
            construct_tower_button: ConstructObjectButton {
                object_type: grid_object_placer,
            },
            advanced_interaction: Default::default(),
        }
    }
    pub fn spawn(builder: &mut ChildBuilder, asset_server: &AssetServer, grid_object_placer: GridObjectPlacer) {
        let image_handle = match &grid_object_placer {
            GridObjectPlacer::MiningComplex => Some(MINING_COMPLEX_BASE_IMAGE),
            GridObjectPlacer::Building(building) => match building.building_type {
                BuildingType::Tower(tower_type) => {
                    match tower_type {
                        TowerType::Blaster => Some(TOWER_BLASTER_BASE_IMAGE),
                        TowerType::Cannon => Some(TOWER_CANNON_BASE_IMAGE),
                        TowerType::RocketLauncher => Some(TOWER_ROCKET_LAUNCHER_BASE_IMAGE),
                    }
                },
                BuildingType::MainBase => Some(MAIN_BASE_BASE_IMAGE),
                BuildingType::EnergyRelay => Some(ENERGY_RELAY_BASE_IMAGE),
                BuildingType::ExplorationCenter => Some(EXPLORATION_CENTER_BASE_IMAGE),
                _ => None,
            },
            GridObjectPlacer::DarkOre => Some(DARK_ORE_BASE_IMAGE),
            _ => None,
        };
        builder.spawn(ConstructObjectButtonBundle::new(grid_object_placer)).with_children(|parent| {
            if let Some(image_handle) = image_handle {
                parent.spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Px(48.0),
                            height: Val::Px(48.0),
                            ..default()
                        },
                        background_color: Color::WHITE.into(),
                        ..default()
                    },
                    UiImage::new(asset_server.load(image_handle)),
                ));
            }
        });
    }
}

pub fn create_construct_menu(
    commands: &mut Commands,
    asset_server: &AssetServer,
) -> Entity {
    let construct_towers_button = commands.spawn(NodeBundle {
        // Main Menu node
        style: Style {
            position_type: PositionType::Absolute,
            top: Val::Percent(40.),
            left: Val::Px(5.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..Default::default()
        },
        ..Default::default()
    }).with_children(|parent| {
        // Construct towers button
        parent.spawn(
            ConstructButtonBundle::new(asset_server.load("ui/construct_towers.png")),
        ).with_children(|parent| {
            // Construct towers list picker
            parent.spawn(
                ConstructMenuListPickerBundle::new(),
            ).with_children(|mut parent| {
                // Specific tower to construct
                ConstructObjectButtonBundle::spawn(&mut parent, asset_server, BuildingType::Tower(TowerType::Blaster).into());
                ConstructObjectButtonBundle::spawn(&mut parent, asset_server, BuildingType::Tower(TowerType::Cannon).into());
                ConstructObjectButtonBundle::spawn(&mut parent, asset_server, BuildingType::Tower(TowerType::RocketLauncher).into());
            });
        });
        parent.spawn(
            ConstructButtonBundle::new(asset_server.load("ui/construct_buildings.png")),
        ).with_children(|parent| {
            // Construct buildings list picker
            parent.spawn(
                ConstructMenuListPickerBundle::new(),
            ).with_children(|mut parent| {
                // Specific building to construct
                ConstructObjectButtonBundle::spawn(&mut parent, asset_server, BuildingType::EnergyRelay.into());
                ConstructObjectButtonBundle::spawn(&mut parent, asset_server, BuildingType::MiningComplex.into());
                ConstructObjectButtonBundle::spawn(&mut parent, asset_server, BuildingType::ExplorationCenter.into());
            });
        });
    }).with_children(|parent| {
        // Construct objects(editor)
        parent.spawn(
            ConstructButtonBundle::new(asset_server.load("ui/construct_editor.png")),
        ).with_children(|parent| {
            // Construct editor list picker
            parent.spawn(
                ConstructMenuListPickerBundle::new(),
            ).with_children(|mut parent| {
                // Specific editor buildings to construct
                ConstructObjectButtonBundle::spawn(&mut parent, asset_server, BuildingType::MainBase.into());
                ConstructObjectButtonBundle::spawn(&mut parent, asset_server, GridObjectPlacer::DarkOre);
                ConstructObjectButtonBundle::spawn(&mut parent, asset_server, GridObjectPlacer::Wall);
                ConstructObjectButtonBundle::spawn(&mut parent, asset_server, GridObjectPlacer::QuantumField(QuantumField::default()));
            });
        });
    }).id();
    construct_towers_button
}

pub fn initialize_construction_menu_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    create_construct_menu(&mut commands, &asset_server);
}

pub fn menu_activation_system(
    mut menu_buttons: Query<(&Interaction, &mut BackgroundColor, &Children), With<ConstructMenuButton>>,
    mut list_pickers: Query<(&Interaction, &mut Visibility), With<ConstructMenuListPicker>>,
) {
    for (menu_interaction, mut background, children) in menu_buttons.iter_mut() {
        let list_picker_entity = children.get(0).unwrap();
        let (list_picker_interaction, mut list_picker_visibility) = list_pickers.get_mut(*list_picker_entity).unwrap();
        if !matches!(menu_interaction, Interaction::None) || !matches!(list_picker_interaction, Interaction::None) {
            background.0.set_a(1.);
            *list_picker_visibility = Visibility::Inherited;
        } else {
            background.0.set_a(NOT_HOVERED_ALPHA);
            *list_picker_visibility = Visibility::Hidden;
        }
    }
}

pub fn construct_building_on_click_system(
    mut grid_object_placer_request: ResMut<GridObjectPlacerRequest>,
    mut menu_buttons: Query<(&AdvancedInteraction, &ConstructObjectButton), Changed<AdvancedInteraction>>,
    mut list_pickers: Query<(&mut Interaction, &mut Visibility), With<ConstructMenuListPicker>>,
) {
    for (advanced_interaction, button) in menu_buttons.iter_mut() {
        if advanced_interaction.was_just_released {
            grid_object_placer_request.set(button.object_type.clone());
            list_pickers.for_each_mut(|(mut interaction, mut visibility)| { *visibility = Visibility::Hidden; *interaction = Interaction::None; });
        }
    }
}
