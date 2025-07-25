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
    pub upgrades_info: AlmanachUpgradeInfo,
    pub current_level: usize,
}
#[derive(Component)]
struct UpgradeButtonChildren {
    // TODO;
}

pub fn on_upgrade_button_added_trigger(
    trigger: Trigger<OnAdd, UpgradeButton>,
    mut commands: Commands,
    upgrade_buttons: Query<&UpgradeButton>,
) {
    let upgrade_button_entity = trigger.target();
    let Ok(upgrade_button) = upgrade_buttons.get(upgrade_button_entity) else { return; };
    let current_upgrade_level = upgrade_button.current_level;

    // Spawn the full upgrade button structure
    commands.entity(upgrade_button_entity).with_children(|parent| {
        parent.spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        )).with_children(|parent| {
            parent.spawn((
                Text::new(format!("{} lvl {}", upgrade_button.upgrades_info.upgrade_type, upgrade_button.current_level)),
                TextColor::from(Color::WHITE),
                TextLayout::new_with_linebreak(LineBreak::NoWrap),
                Node {
                    margin: UiRect{ left: Val::Px(4.), right: Val::Px(4.), ..default() },
                    ..default()
                },
            ));
            // Costs container
            parent.spawn((
                Node {
                    width: Val::Percent(100.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                Children::spawn(
                    SpawnIter(upgrade_button.upgrades_info.levels[current_upgrade_level].cost.clone().into_iter().map(|cost| CostIndicator { cost, ..default() })),
                ),
            ));
        });
    });
}