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
            .add_observer(on_max_health_modifier_inserted)
            .add_observer(on_movement_speed_modifier_inserted)
            .add_observer(on_energy_supply_range_modifier_inserted)
            .add_observer(ApplyPotentialUpgrade::on_trigger);
    }
}


#[derive(Component)]
#[relationship(relationship_target = Modifiers)]
pub struct ModifierOf(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = ModifierOf, linked_spawn)]
pub struct Modifiers(Vec<Entity>);

pub trait Modifier: Property + Default {
    const MODIFIER_TYPE: ModifierType;
    fn from_baseline(info: &AlmanachBuildingInfo) -> Self {
        let mut modifier = Self::default();
        modifier.set(info.baseline[&Self::MODIFIER_TYPE]);
        modifier
    }
}

#[derive(Component, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModifierType {
    AttackSpeed,
    AttackRange,
    AttackDamage,
    MaxHealth,
    MovementSpeed,
    EnergySupplyRange,
}
#[derive(Component, Clone, Default)]#[component(immutable)]#[require(ModifierType = ModifierType::AttackSpeed)] pub struct ModifierAttackSpeed(pub f32);
impl Property for ModifierAttackSpeed {
    fn get(&self) -> f32 { self.0 }
    fn set(&mut self, value: f32) { self.0 = value }
}
impl Modifier for ModifierAttackSpeed {
    const MODIFIER_TYPE: ModifierType = ModifierType::AttackSpeed;
}
#[derive(Component, Clone, Default)]#[component(immutable)]#[require(ModifierType = ModifierType::AttackRange)] pub struct ModifierAttackRange(pub f32);
impl Property for ModifierAttackRange {
    fn get(&self) -> f32 { self.0 }
    fn set(&mut self, value: f32) { self.0 = value }
}
impl Modifier for ModifierAttackRange {
    const MODIFIER_TYPE: ModifierType = ModifierType::AttackRange;
}
#[derive(Component, Clone, Default)]#[component(immutable)]#[require(ModifierType = ModifierType::AttackDamage)] pub struct ModifierAttackDamage(pub f32);
impl Property for ModifierAttackDamage {
    fn get(&self) -> f32 { self.0 }
    fn set(&mut self, value: f32) { self.0 = value }
}
impl Modifier for ModifierAttackDamage {
    const MODIFIER_TYPE: ModifierType = ModifierType::AttackDamage;
}
#[derive(Component, Clone, Default)]#[component(immutable)]#[require(ModifierType = ModifierType::MaxHealth)] pub struct ModifierMaxHealth(pub f32);
impl Property for ModifierMaxHealth {
    fn get(&self) -> f32 { self.0 }
    fn set(&mut self, value: f32) { self.0 = value }
}
impl Modifier for ModifierMaxHealth {
    const MODIFIER_TYPE: ModifierType = ModifierType::MaxHealth;
}
#[derive(Component, Clone, Default)]#[component(immutable)]#[require(ModifierType = ModifierType::MovementSpeed)] pub struct ModifierMovementSpeed(pub f32);
impl Property for ModifierMovementSpeed {
    fn get(&self) -> f32 { self.0 }
    fn set(&mut self, value: f32) { self.0 = value }
}
#[derive(Component, Clone, Default)]#[component(immutable)]#[require(ModifierType = ModifierType::EnergySupplyRange)] pub struct ModifierEnergySupplyRange(pub f32);
impl Property for ModifierEnergySupplyRange {
    fn get(&self) -> f32 { self.0 }
    fn set(&mut self, value: f32) { self.0 = value }
}
impl Modifier for ModifierEnergySupplyRange {
    const MODIFIER_TYPE: ModifierType = ModifierType::EnergySupplyRange;
}

#[derive(Component, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModifierSource {
    Baseline,
    Upgrade,
}
#[derive(Component, Clone)]#[require(ModifierSource = ModifierSource::Baseline)] pub struct ModifierSourceBaseline;
#[derive(Component, Clone)]#[require(ModifierSource = ModifierSource::Upgrade)] pub struct ModifierSourceUpgrade{ pub current_level: usize, pub upgrade_info: AlmanachUpgradeInfo }
impl ModifierSourceUpgrade {
    pub fn current_cost(&self) -> &Vec<Cost> {
        &self.upgrade_info.levels[self.current_level].cost
    }
    pub fn current_value(&self) -> f32 {
        self.upgrade_info.levels[self.current_level].value
    }
}


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
) {
    let modifier_entity = trigger.target();
    let Ok((_, modifier_of)) = modifiers.get(modifier_entity) else { return; };
    let all_modifiers_list = modification_targets.get(modifier_of.0).unwrap();
    let new_value = all_modifiers_list.iter()
        .filter_map(|entity| modifiers.get(entity).ok())
        .map(|(attack_range, _)| attack_range.0)
        .sum();
    commands.entity(modifier_of.0).insert(AttackRange(new_value));
}
fn on_attack_speed_modifier_inserted(
    trigger: Trigger<OnInsert, ModifierAttackSpeed>,
    mut commands: Commands,
    modifiers: Query<(&ModifierAttackSpeed, &ModifierOf)>,
    modification_targets: Query<&Modifiers>,
) {
    let modifier_entity = trigger.target();
    let Ok((_, modifier_of)) = modifiers.get(modifier_entity) else { return; };
    let all_modifiers_list = modification_targets.get(modifier_of.0).unwrap();
    let new_value = all_modifiers_list.iter()
        .filter_map(|entity| modifiers.get(entity).ok())
        .map(|(attack_speed, _)| attack_speed.0)
        .sum();
    commands.entity(modifier_of.0).insert(AttackSpeed(new_value));
}
fn on_attack_damage_modifier_inserted(
    trigger: Trigger<OnInsert, ModifierAttackDamage>,
    mut commands: Commands,
    modifiers: Query<(&ModifierAttackDamage, &ModifierOf)>,
    modification_targets: Query<&Modifiers>,
) {
    let modifier_entity = trigger.target();
    let Ok((_, modifier_of)) = modifiers.get(modifier_entity) else { return; };
    let all_modifiers_list = modification_targets.get(modifier_of.0).unwrap();
    let new_value = all_modifiers_list.iter()
        .filter_map(|entity| modifiers.get(entity).ok())
        .map(|(attack_damage, _)| attack_damage.0)
        .sum();
    commands.entity(modifier_of.0).insert(AttackDamage(new_value));
}
fn on_max_health_modifier_inserted(
    trigger: Trigger<OnInsert, ModifierMaxHealth>,
    mut commands: Commands,
    modifiers: Query<(&ModifierMaxHealth, &ModifierOf)>,
    modification_targets: Query<&Modifiers>,
) {
    let modifier_entity = trigger.target();
    let Ok((_, modifier_of)) = modifiers.get(modifier_entity) else { return; };
    let all_modifiers_list = modification_targets.get(modifier_of.0).unwrap();
    let new_value = all_modifiers_list.iter()
        .filter_map(|entity| modifiers.get(entity).ok())
        .map(|(max_health, _)| max_health.0)
        .sum();
    commands.entity(modifier_of.0).insert(MaxHealth(new_value));
}

fn on_movement_speed_modifier_inserted(
    trigger: Trigger<OnInsert, ModifierMovementSpeed>,
    mut commands: Commands,
    modifiers: Query<(&ModifierMovementSpeed, &ModifierOf)>,
    modification_targets: Query<&Modifiers>,
) {
    let modifier_entity = trigger.target();
    let Ok((_, modifier_of)) = modifiers.get(modifier_entity) else { return; };
    let all_modifiers_list = modification_targets.get(modifier_of.0).unwrap();
    let new_value = all_modifiers_list.iter()
        .filter_map(|entity| modifiers.get(entity).ok())
        .map(|(movement_speed, _)| movement_speed.0)
        .sum();
    commands.entity(modifier_of.0).insert(MovementSpeed(new_value));
}

fn on_energy_supply_range_modifier_inserted(
    trigger: Trigger<OnInsert, ModifierEnergySupplyRange>,
    mut commands: Commands,
    modifiers: Query<(&ModifierEnergySupplyRange, &ModifierOf)>,
    modification_targets: Query<&Modifiers>,
) {
    let modifier_entity = trigger.target();
    let Ok((_, modifier_of)) = modifiers.get(modifier_entity) else { return; };
    let all_modifiers_list = modification_targets.get(modifier_of.0).unwrap();
    let new_value = all_modifiers_list.iter()
        .filter_map(|entity| modifiers.get(entity).ok())
        .map(|(energy_supply_range, _)| energy_supply_range.0)
        .sum();
    commands.entity(modifier_of.0).insert(EnergySupplyRange(new_value));
}



////////////////////
////  UPGRADES  ////
////////////////////


#[derive(Component)]
//#[require(ApplyPotentialUpgradeObserver)]
#[relationship(relationship_target = PotentialUpgrades)]
pub struct PotentialUpgradeOf(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = PotentialUpgradeOf, linked_spawn)]
pub struct PotentialUpgrades(Vec<Entity>);

#[derive(Event)]
pub struct ApplyPotentialUpgrade;
impl ApplyPotentialUpgrade {
    fn on_trigger(
        trigger: Trigger<ApplyPotentialUpgrade>,
        mut commands: Commands,
        mut potential_upgrades: Query<(&ModifierType, &mut ModifierSourceUpgrade, &PotentialUpgradeOf)>,
        // Modifier value components
    ) {
        let entity = trigger.target();
        let Ok((modifier_type, mut modifier_source_upgrade, parent)) = potential_upgrades.get_mut(entity) else { return; };

        // First turn the potential upgrade into full fledged modifier. We first need to insert ModifierOf and then clone as otherwise on_add observers won't trigger properly(they expect ModifierOf to be present)
        let modifier_entity = commands.spawn(ModifierOf(parent.0)).id();
        let mut commands_entity = commands.entity(entity);
        commands_entity.clone_with(modifier_entity, |_| {});
        
        // Then level up the potential upgrade
        if modifier_source_upgrade.current_level + 1 >= modifier_source_upgrade.upgrade_info.levels.len() { 
            commands_entity.despawn();
            return;
         }
        modifier_source_upgrade.current_level += 1;

        // And add matching value component
        let new_value = modifier_source_upgrade.current_value();
        match modifier_type {
            ModifierType::AttackDamage => { commands_entity.insert(ModifierAttackDamage(new_value)); }
            ModifierType::AttackRange => { commands_entity.insert(ModifierAttackRange(new_value)); }
            ModifierType::AttackSpeed => { commands_entity.insert(ModifierAttackSpeed(new_value)); }
            ModifierType::MaxHealth => { commands_entity.insert(ModifierMaxHealth(new_value)); }
            ModifierType::MovementSpeed => { commands_entity.insert(ModifierMovementSpeed(new_value)); }
            ModifierType::EnergySupplyRange => { commands_entity.insert(ModifierEnergySupplyRange(new_value)); }
        }
    }
}
