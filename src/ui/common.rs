use bevy::{ecs::spawn::SpawnIter, text::LineBreak};
use lib_ui::prelude::CostIndicator;

use crate::prelude::*;

pub struct CommonPlugin;
impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(UpgradeLine::on_add);
    }
}

////////////////////////////////////////////
////          Upgrade Button            ////
////////////////////////////////////////////
#[derive(Component)]
#[require(Node)]
pub struct UpgradeLine {
    pub upgrade_entity: Entity,
    pub upgrades_type: UpgradeType,
    pub costs: Vec<Cost>,
    pub current_value: f32,
    pub next_value: f32
}
impl UpgradeLine {
    fn on_add(
        trigger: Trigger<OnAdd, UpgradeLine>,
        mut commands: Commands,
        upgrade_lines: Query<&UpgradeLine>,
    ) {
        let upgrade_line_entity = trigger.target();
        let Ok(upgrade_line) = upgrade_lines.get(upgrade_line_entity) else { return; };

        // Spawn the full upgrade line structure
        commands.entity(upgrade_line_entity).with_children(|parent| {
            // Main container for the entire upgrade line
            parent.spawn((
                Node {
                    width: Val::Percent(100.),
                    height: Val::Px(36.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(Val::Px(8.)),
                    ..default()
                },
            )).with_children(|parent| {
                // Left: Upgrade name + current level
                parent.spawn((
                    Text::new(format!("{} {}", upgrade_line.upgrades_type, upgrade_line.current_value)),
                    TextColor::from(Color::WHITE),
                    TextLayout::new_with_linebreak(LineBreak::NoWrap),
                    Node {
                        ..default()
                    },
                ));
                
                // Right: Composite clickable button
                parent.spawn((
                    UpgradeButton(upgrade_line_entity),
                    Node {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::horizontal(Val::Px(6.)),
                        border: UiRect::all(Val::Px(1.)),
                        ..default()
                    },
                    BorderColor(Color::WHITE),
                    BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
                ))
                    .observe(Self::on_click)
                    .with_children(|parent| {
                    parent.spawn((
                        Text::new(format!("-> {:.0}", upgrade_line.next_value)),
                        TextColor::from(Color::WHITE),
                        TextLayout::new_with_linebreak(LineBreak::NoWrap),
                        Node {
                            margin: UiRect::right(Val::Px(8.)),
                            ..default()
                        },
                    ));
                    
                    // Cost indicators
                    parent.spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::right(Val::Px(4.)),
                            ..default()
                        },
                        Children::spawn(
                            SpawnIter(upgrade_line.costs.clone().into_iter().map(|cost| CostIndicator::from(cost))),
                        ),
                    ));
                    
                    // Upgrade text
                    parent.spawn((
                        Text::new("Upgrade"),
                        TextColor::from(Color::WHITE),
                        TextLayout::new_with_linebreak(LineBreak::NoWrap),
                    ));
                });
            });
        });
    }

    fn on_click(
        trigger: Trigger<Pointer<Click>>,
        upgrade_buttons: Query<&UpgradeButton>,
        upgrade_lines: Query<&UpgradeLine>,
    ) {
        let entity = trigger.target();
        let Ok(upgrade_button) = upgrade_buttons.get(entity) else { return; };
        let Ok(upgrade_line) = upgrade_lines.get(upgrade_button.0) else { return; };
        println!("Upgrade button clicked: {}", upgrade_line.upgrades_type);
    }
}


#[derive(Component)]
#[require(Button)]
pub struct UpgradeButton(Entity);