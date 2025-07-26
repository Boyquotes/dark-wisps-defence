use bevy::math::i32;

use crate::lib_prelude::*;

pub mod modifiers_prelude {
    pub use super::*;
}

pub struct ModifiersPlugin;
impl Plugin for ModifiersPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(on_attack_range_modifier_inserted)
            .add_observer(on_attack_speed_modifier_inserted)
            .add_observer(on_attack_damage_modifier_inserted)
            .add_observer(GrantUpgradeModifier::on_trigger);
    }
}


#[derive(Component)]
#[relationship(relationship_target = Modifiers)]
pub struct ModifierOf(Entity);

#[derive(Component)]
#[relationship_target(relationship = ModifierOf, linked_spawn)]
pub struct Modifiers(Vec<Entity>);

#[derive(Component, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModifierType {
    AttackSpeed,
    AttackRange,
    AttackDamage,
    MaxHealth
}
#[derive(Component)]#[component(immutable)]#[require(ModifierType = ModifierType::AttackSpeed)] pub struct ModifierAttackSpeed(pub f32);
#[derive(Component)]#[component(immutable)]#[require(ModifierType = ModifierType::AttackRange)] pub struct ModifierAttackRange(pub usize);
#[derive(Component)]#[component(immutable)]#[require(ModifierType = ModifierType::AttackDamage)] pub struct ModifierAttackDamage(pub i32);
#[derive(Component)]#[component(immutable)]#[require(ModifierType = ModifierType::MaxHealth)] pub struct ModifierMaxHealth(pub i32);


#[derive(Component, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModifierSource {
    Baseline,
    Upgrade,
}
#[derive(Component)]#[require(ModifierSource = ModifierSource::Baseline)] pub struct ModifierSourceBaseline;
#[derive(Component)]#[require(ModifierSource = ModifierSource::Upgrade)] pub struct ModifierSourceUpgrade{ pub level: usize }


/// On Modifier inserted:
/// - Get the modifier's target(parent)
/// - Get all modifiers of the target
/// - Calculate the new value
/// - Insert the new value
fn on_attack_range_modifier_inserted(
    trigger: Trigger<OnInsert, ModifierAttackRange>,
    mut commands: Commands,
    modifiers: Query<(&ModifierAttackRange, &ModifierOf)>,
    modification_targets: Query<&Modifiers>,
) -> Result<()> {
    let modifier_entity = trigger.target();
    let (_, modifier_of) = modifiers.get(modifier_entity)?;
    let all_modifiers_list = modification_targets.get(modifier_of.0)?;
    let new_value = all_modifiers_list.iter()
        .filter_map(|entity| modifiers.get(entity).ok())
        .map(|(attack_range, _)| attack_range.0)
        .sum();
    commands.entity(modifier_of.0).insert(AttackRange(new_value));
    Ok(())
}
fn on_attack_speed_modifier_inserted(
    trigger: Trigger<OnInsert, ModifierAttackSpeed>,
    mut commands: Commands,
    modifiers: Query<(&ModifierAttackSpeed, &ModifierOf)>,
    modification_targets: Query<&Modifiers>,
) -> Result<()> {
    let modifier_entity = trigger.target();
    let (_, modifier_of) = modifiers.get(modifier_entity)?;
    let all_modifiers_list = modification_targets.get(modifier_of.0)?;
    let new_value = all_modifiers_list.iter()
        .filter_map(|entity| modifiers.get(entity).ok())
        .map(|(attack_speed, _)| attack_speed.0)
        .sum();
    commands.entity(modifier_of.0).insert(AttackSpeed(new_value));
    Ok(())
}
fn on_attack_damage_modifier_inserted(
    trigger: Trigger<OnInsert, ModifierAttackDamage>,
    mut commands: Commands,
    modifiers: Query<(&ModifierAttackDamage, &ModifierOf)>,
    modification_targets: Query<&Modifiers>,
) -> Result<()> {
    let modifier_entity = trigger.target();
    let (_, modifier_of) = modifiers.get(modifier_entity)?;
    let all_modifiers_list = modification_targets.get(modifier_of.0)?;
    let new_value = all_modifiers_list.iter()
        .filter_map(|entity| modifiers.get(entity).ok())
        .map(|(attack_damage, _)| attack_damage.0)
        .sum();
    commands.entity(modifier_of.0).insert(AttackDamage(new_value));
    Ok(())
}
// fn on_max_health_modifier_inserted(
//     trigger: Trigger<OnInsert, ModifierMaxHealth>,
//     mut commands: Commands,
//     modifiers: Query<(&ModifierMaxHealth, &ModifierOf)>,
//     modification_targets: Query<&Modifiers>,
// ) -> Result<()> {
//     let modifier_entity = trigger.target();
//     let (_, modifier_of) = modifiers.get(modifier_entity)?;
//     let all_modifiers_list = modification_targets.get(modifier_of.0)?;
//     let new_value = all_modifiers_list.iter()
//         .filter_map(|entity| modifiers.get(entity).ok())
//         .map(|(max_health, _)| max_health.0)
//         .sum();
//     commands.entity(modifier_of.0).insert(MaxHealth(new_value));
//     Ok(())
// }


#[derive(Event)]
pub struct GrantUpgradeModifier {
    pub modifier_type: ModifierType,
    pub upgrade_level: usize,
    pub value: f32,
}
impl GrantUpgradeModifier {
    fn on_trigger(
        trigger: Trigger<GrantUpgradeModifier>,
        mut commands: Commands,
    ) {
        let entity = trigger.target();
        let mut entity_commands = commands.entity(entity);
        let trigger = trigger.event();
        match trigger.modifier_type {
            ModifierType::AttackSpeed => entity_commands.with_related::<ModifierOf>((ModifierOf(entity), ModifierSourceUpgrade{ level: trigger.upgrade_level }, ModifierAttackSpeed(trigger.value))),
            ModifierType::AttackRange => entity_commands.with_related::<ModifierOf>((ModifierOf(entity), ModifierSourceUpgrade{ level: trigger.upgrade_level }, ModifierAttackRange(trigger.value as usize))),
            ModifierType::AttackDamage => entity_commands.with_related::<ModifierOf>((ModifierOf(entity), ModifierSourceUpgrade{ level: trigger.upgrade_level }, ModifierAttackDamage(trigger.value as i32))),
            ModifierType::MaxHealth => entity_commands.with_related::<ModifierOf>((ModifierOf(entity), ModifierSourceUpgrade{ level: trigger.upgrade_level }, ModifierMaxHealth(trigger.value as i32))),
        };
    }
}