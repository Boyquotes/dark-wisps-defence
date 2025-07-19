use bevy::text::LineBreak;

use crate::prelude::*;

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
#[derive(Event)]
pub struct UpgradeButtonRebuildTrigger;

pub fn on_upgrade_button_added_trigger(
    trigger: Trigger<OnAdd, UpgradeButton>,
    mut commands: Commands,
    upgrade_buttons: Query<&UpgradeButton>,
) {
    let upgrade_button_entity = trigger.target();
    let Ok(upgrade_button) = upgrade_buttons.get(upgrade_button_entity) else { return; };

    // Spawn the full upgrade button structure
    commands.entity(upgrade_button_entity).with_children(|parent| {
        parent.spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
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
            ));
        });
    });
}

fn on_upgrade_button_rebuild_trigger(
    trigger: Trigger<UpgradeButtonRebuildTrigger>,
    mut commands: Commands,
    upgrade_buttons: Query<Entity, With<UpgradeButton>>,
) {
    let upgrade_button_entity = trigger.target();
}
