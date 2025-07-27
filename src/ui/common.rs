use bevy::{ecs::spawn::SpawnIter, text::LineBreak};
use lib_ui::prelude::CostIndicator;

use crate::prelude::*;

pub struct CommonPlugin;
impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(UpgradeLineBuilder::on_add_attack_speed_builder)
            .add_observer(UpgradeLineBuilder::on_add_attack_range_builder)
            .add_observer(UpgradeLineBuilder::on_add_attack_damage_builder)
            .add_observer(UpgradeLineBuilder::on_add_sanity_check)
            .add_observer(UpgradeLine::on_add);
    }
}

////////////////////////////////////////////
////           Upgrade Line             ////
////////////////////////////////////////////
#[derive(Component)]
pub struct UpgradeLineBuilder {
    pub potential_upgrade_entity: Entity,
}
impl UpgradeLineBuilder {
    fn on_add_sanity_check(
        trigger: Trigger<OnAdd, UpgradeLineBuilder>,
        mut commands: Commands,
        upgrade_lines: Query<&UpgradeLineBuilder>,
        all_potential_upgrades: Query<(), With<PotentialUpgradeOf>>,
    ) {
        let upgrade_line_entity = trigger.target();
        let upgrade_line = upgrade_lines.get(upgrade_line_entity).unwrap();
        if !all_potential_upgrades.contains(upgrade_line.potential_upgrade_entity) {
            // Upgrade was removed, remove the line as well
            commands.entity(upgrade_line_entity).despawn();
            return;
        }
    }
    fn on_add_attack_speed_builder(
        trigger: Trigger<OnAdd, UpgradeLineBuilder>,
        mut commands: Commands,
        upgrade_lines: Query<&UpgradeLineBuilder>,
        potential_upgrades: Query<(&ModifierType, &ModifierAttackSpeed, &ModifierSourceUpgrade, &PotentialUpgradeOf)>,
        attack_speeds: Query<&AttackSpeed>,
    ) {
        let upgrade_line_entity = trigger.target();
        let Ok(upgrade_line) = upgrade_lines.get(upgrade_line_entity) else { return; };
        let Ok((modifier_type, modifier, modifier_source, parent)) = potential_upgrades.get(upgrade_line.potential_upgrade_entity) else { return; };
        let Ok(current_attack_speed) = attack_speeds.get(parent.0) else { return; };

        commands.entity(upgrade_line_entity)
            .remove::<UpgradeLineBuilder>()
            .insert(UpgradeLine {
                potential_upgrade_entity: upgrade_line.potential_upgrade_entity,
                upgrades_type: *modifier_type,
                costs: modifier_source.current_cost().clone(),
                current_value: current_attack_speed.0,
                next_value: modifier.0 + current_attack_speed.0,
            });
    }
    fn on_add_attack_damage_builder(
        trigger: Trigger<OnAdd, UpgradeLineBuilder>,
        mut commands: Commands,
        upgrade_lines: Query<&UpgradeLineBuilder>,
        potential_upgrades: Query<(&ModifierType, &ModifierAttackDamage, &ModifierSourceUpgrade, &PotentialUpgradeOf)>,
        attack_damages: Query<&AttackDamage>,
    ) {
        let upgrade_line_entity = trigger.target();
        let Ok(upgrade_line) = upgrade_lines.get(upgrade_line_entity) else { return; };
        let Ok((modifier_type, modifier, modifier_source, parent)) = potential_upgrades.get(upgrade_line.potential_upgrade_entity) else { return; };
        let Ok(current_attack_damage) = attack_damages.get(parent.0) else { return; };

        commands.entity(upgrade_line_entity)
            .remove::<UpgradeLineBuilder>()
            .insert(UpgradeLine {
                potential_upgrade_entity: upgrade_line.potential_upgrade_entity,
                upgrades_type: *modifier_type,
                costs: modifier_source.current_cost().clone(),
                current_value: current_attack_damage.0 as f32,
                next_value: (modifier.0 + current_attack_damage.0) as f32,
            });
    }
    fn on_add_attack_range_builder(
        trigger: Trigger<OnAdd, UpgradeLineBuilder>,
        mut commands: Commands,
        upgrade_lines: Query<&UpgradeLineBuilder>,
        potential_upgrades: Query<(&ModifierType, &ModifierAttackRange, &ModifierSourceUpgrade, &PotentialUpgradeOf)>,
        attack_ranges: Query<&AttackRange>,
    ) {
        let upgrade_line_entity = trigger.target();
        let Ok(upgrade_line) = upgrade_lines.get(upgrade_line_entity) else { return; };
        let Ok((modifier_type, modifier, modifier_source, parent)) = potential_upgrades.get(upgrade_line.potential_upgrade_entity) else { return; };
        let Ok(current_attack_range) = attack_ranges.get(parent.0) else { return; };

        commands.entity(upgrade_line_entity)
            .remove::<UpgradeLineBuilder>()
            .insert(UpgradeLine {
                potential_upgrade_entity: upgrade_line.potential_upgrade_entity,
                upgrades_type: *modifier_type,
                costs: modifier_source.current_cost().clone(),
                current_value: current_attack_range.0 as f32,
                next_value: (modifier.0 + current_attack_range.0) as f32,
            });
    }
    
}
#[derive(Component)]
#[require(Node)]
pub struct UpgradeLine {
    pub potential_upgrade_entity: Entity,
    pub upgrades_type: ModifierType,
    pub costs: Vec<Cost>,
    pub current_value: f32,
    pub next_value: f32
}
impl UpgradeLine {
    fn on_add(
        trigger: Trigger<OnInsert, UpgradeLine>,
        mut commands: Commands,
        upgrade_lines: Query<&UpgradeLine>,
    ) {
        let upgrade_line_entity = trigger.target();
        let Ok(upgrade_line) = upgrade_lines.get(upgrade_line_entity) else { return; };

        // Clear everything in case this is rebuild operation
        commands.entity(upgrade_line_entity).despawn_related::<Children>();

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
                    Text::new(format!("{:?} {}", upgrade_line.upgrades_type, upgrade_line.current_value)),
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
        mut commands: Commands,
        upgrade_buttons: Query<&UpgradeButton>,
        upgrade_lines: Query<(Entity, &UpgradeLine)>,
    ) {
        let entity = trigger.target();
        let Ok(upgrade_button) = upgrade_buttons.get(entity) else { return; };
        let Ok((upgrade_line_entity, upgrade_line)) = upgrade_lines.get(upgrade_button.0) else { return; };
        println!("Upgrade button clicked: {:?}", upgrade_line.upgrades_type);
        commands.trigger_targets(ApplyPotentialUpgrade, [upgrade_line.potential_upgrade_entity]);

        // Rebuild the button
        commands.entity(upgrade_line_entity).insert(UpgradeLineBuilder { potential_upgrade_entity: upgrade_line.potential_upgrade_entity });
    }
}


#[derive(Component)]
#[require(Button)]
pub struct UpgradeButton(Entity);