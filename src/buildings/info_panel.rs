use bevy::color::palettes::css::BLUE;
use lib_ui::healthbar::Healthbar;

use crate::prelude::*;
use crate::ui::common::UpgradeLineBuilder;
use crate::ui::display_info_panel::{DisplayInfoPanel, DisplayPanelMainContentRoot, UiMapObjectFocusedTrigger};


pub struct InfoPanelPlugin;
impl Plugin for InfoPanelPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(PostStartup, initialize_building_panel_content_system)
            .add_systems(Update, update_building_info_panel_system.run_if(in_state(UiInteraction::DisplayInfoPanel)))
            .add_observer(on_ui_map_object_focus_changed_trigger)
            .add_observer(on_building_info_panel_enabled_for_towers_trigger);
    }
}

/// Common
#[derive(Component)]
pub struct BuildingInfoPanel;
#[derive(Component)]
pub struct BuildingInfoPanelNameText;
#[derive(Component)]
pub struct BuildingInfoPanelHealthbar;
#[derive(Event)]
pub struct BuildingInfoPanelEnabledTrigger;

// Tower Subpanel
#[derive(Component)]
struct BuildingInfoPanelTowerRoot;
#[derive(Component)]
struct BuildingInfoPanelTowerUpgradeCountText;
#[derive(Component)]
struct BuildingInfoPanelTowerUpgradesContainer;

fn update_building_info_panel_system(
    buildings: Query<&Health, With<Building>>,
    display_info_panel: Query<&DisplayInfoPanel>,
    mut healthbars: Query<&mut Healthbar, With<BuildingInfoPanelHealthbar>>,
) {
    let focused_entity = display_info_panel.single().unwrap().current_focus;
    let Ok(health) = buildings.get(focused_entity) else { return; };
    // Update the healthbar
    let Ok(mut healthbar) = healthbars.single_mut() else { return; };
    healthbar.value = health.get_current() as f32;
    healthbar.max_value = health.get_max() as f32;
    let health_percentage = health.get_percent();
    healthbar.color = Color::linear_rgba(1. - health_percentage, health_percentage, 0., 1.);
}

fn on_ui_map_object_focus_changed_trigger(
    trigger: Trigger<UiMapObjectFocusedTrigger>,
    mut commands: Commands,
    almanach: Res<Almanach>,
    mut building_name_text: Query<&mut Text, With<BuildingInfoPanelNameText>>,
    mut building_panel: Query<&mut Node, With<BuildingInfoPanel>>,
    buildings: Query<&BuildingType>,
) {
    let focused_entity = trigger.target();
    let Ok(building_type) = buildings.get(focused_entity) else { 
        building_panel.single_mut().unwrap().display = Display::None;
        return; 
    };
    building_panel.single_mut().unwrap().display = Display::Flex;
    commands.trigger_targets(BuildingInfoPanelEnabledTrigger, [focused_entity]);

    // Update the building name
    building_name_text.single_mut().unwrap().0 = almanach.get_building_info(*building_type).name.to_string();
}

fn initialize_building_panel_content_system(
    mut commands: Commands,
    display_info_panel_main_content_root: Query<Entity, With<DisplayPanelMainContentRoot>>,
) {
    let display_info_panel_main_content_root = display_info_panel_main_content_root.single().unwrap();
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
            BuildingInfoPanel,
            children![
                // Top line of the panel
                (
                    Node {
                        width: Val::Percent(100.),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Start,
                        ..default()
                    },
                    children![
                        // Building name
                        (
                            Text::new("### Building Name ###"),
                            TextColor::from(BLUE),
                            TextLayout::new_with_linebreak(LineBreak::NoWrap),
                            Node {
                                margin: UiRect{ left: Val::Px(4.), right: Val::Px(4.), ..default() },
                                ..default()
                            },
                            BuildingInfoPanelNameText,
                        ),
                        // Building Healthbar
                        (
                            Node {
                                width: Val::Percent(100.),
                                height: Val::Percent(100.),
                                ..default()
                            },
                            Healthbar::default(),
                            BuildingInfoPanelHealthbar,
                        ),
                    ],
                ),
                // Specialized panels depending on the building type
                tower_subpanel_content_bundle(),
            ],
        ));
    });
}

// Tower subpanel section
fn on_building_info_panel_enabled_for_towers_trigger(
    trigger: Trigger<BuildingInfoPanelEnabledTrigger>,
    mut commands: Commands,
    mut tower_subpanel_root: Query<&mut Node, With<BuildingInfoPanelTowerRoot>>,
    towers: Query<&PotentialUpgrades, With<MarkerTower>>,
    upgrades_containers: Query<Entity, With<BuildingInfoPanelTowerUpgradesContainer>>,
){
    let focused_entity = trigger.target();
    let Ok(potential_upgrades) = towers.get(focused_entity) else {
        tower_subpanel_root.single_mut().unwrap().display = Display::None;
        return;
    };
    tower_subpanel_root.single_mut().unwrap().display = Display::Flex;

    // Rebuild the upgrades container
    let upgrades_container = upgrades_containers.single().unwrap();
    
    // Clear all existing children
    commands.entity(upgrades_container).despawn_related::<Children>();

    // Create upgrade buttons for each available upgrade
    commands.entity(upgrades_container).with_children(|parent| {
        potential_upgrades.iter().for_each(|potential_upgrade_entity| {
            parent.spawn(UpgradeLineBuilder { potential_upgrade_entity });
        });
    });
}

fn tower_subpanel_content_bundle() -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Start,
            ..default()
        },
        BuildingInfoPanelTowerRoot,
        children![
            (
                Text::new("--- Upgrades ##/## ---"),
                TextColor::from(BLUE),
                TextLayout::new_with_linebreak(LineBreak::NoWrap),
                Node {
                    margin: UiRect{ left: Val::Px(4.), right: Val::Px(4.), ..default() },
                    ..default()
                },
                BuildingInfoPanelTowerUpgradeCountText,
            ),
            (
                Node {
                    width: Val::Percent(100.),
                    justify_items: JustifyItems::Center,
                    ..default()
                },
                BuildingInfoPanelTowerUpgradesContainer,
            ),
        ],
    )
}
