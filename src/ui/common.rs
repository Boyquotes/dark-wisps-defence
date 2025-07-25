use bevy::{ecs::spawn::SpawnIter, text::LineBreak};
use lib_ui::prelude::CostIndicator;

use crate::prelude::*;

pub struct CommonPlugin;
impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(on_upgrade_button_added_trigger);
    }
}

////////////////////////////////////////////
////          Upgrade Button            ////
////////////////////////////////////////////
#[derive(Component)]
#[require(Button)]
pub struct UpgradeButton {
    pub upgrades_type: UpgradeType,
    pub costs: Vec<Cost>,
    pub current_value: f32,
    pub next_value: f32
}

pub fn on_upgrade_button_added_trigger(
    trigger: Trigger<OnAdd, UpgradeButton>,
    mut commands: Commands,
    upgrade_buttons: Query<&UpgradeButton>,
) {
    let upgrade_button_entity = trigger.target();
    let Ok(upgrade_button) = upgrade_buttons.get(upgrade_button_entity) else { return; };

    // Spawn the full upgrade button structure
    commands.entity(upgrade_button_entity).with_children(|parent| {
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
                Text::new(format!("{} {}", upgrade_button.upgrades_type, upgrade_button.current_value)),
                TextColor::from(Color::WHITE),
                TextLayout::new_with_linebreak(LineBreak::NoWrap),
                Node {
                    ..default()
                },
            ));
            
            // Right: Composite clickable button
            parent.spawn((
                Button::default(),
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
            )).with_children(|parent| {
                parent.spawn((
                    Text::new(format!("-> {:.0}", upgrade_button.next_value)),
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
                        SpawnIter(upgrade_button.costs.clone().into_iter().map(|cost| CostIndicator::from(cost))),
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