use crate::lib_prelude::*;

pub mod modifiers_prelude {
    pub use super::*;
}

pub struct ModifiersPlugin;
impl Plugin for ModifiersPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(recalculate_on_modifier_inserted::<ModifierAttackRange, AttackRange>)
            .add_observer(recalculate_on_modifier_inserted::<ModifierAttackSpeed, AttackSpeed>)
            .add_observer(recalculate_on_modifier_inserted::<ModifierAttackDamage, AttackDamage>)
            .add_observer(recalculate_on_modifier_inserted::<ModifierMaxHealth, MaxHealth>)
            .add_observer(recalculate_on_modifier_inserted::<ModifierMovementSpeed, MovementSpeed>)
            .add_observer(recalculate_on_modifier_inserted::<ModifierEnergySupplyRange, EnergySupplyRange>)
            .add_observer(ApplyPotentialUpgrade::on_trigger);
    }
}


#[derive(Component)]
#[relationship(relationship_target = Modifiers)]
pub struct ModifierOf(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = ModifierOf, linked_spawn)]
pub struct Modifiers(Vec<Entity>);

pub trait Modifier: Property + Sized {
    const MODIFIER_TYPE: ModifierType;
    fn from_baseline(info: &AlmanachBuildingInfo) -> Self {
        Self::new(info.baseline[&Self::MODIFIER_TYPE])
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
#[derive(Component, Clone, Default, Property, Modifier)]#[component(immutable)]#[require(ModifierType = ModifierType::AttackSpeed)] pub struct ModifierAttackSpeed(pub f32);
#[derive(Component, Clone, Default, Property, Modifier)]#[component(immutable)]#[require(ModifierType = ModifierType::AttackRange)] pub struct ModifierAttackRange(pub f32);
#[derive(Component, Clone, Default, Property, Modifier)]#[component(immutable)]#[require(ModifierType = ModifierType::AttackDamage)] pub struct ModifierAttackDamage(pub f32);
#[derive(Component, Clone, Default, Property, Modifier)]#[component(immutable)]#[require(ModifierType = ModifierType::MaxHealth)] pub struct ModifierMaxHealth(pub f32);
#[derive(Component, Clone, Default, Property, Modifier)]#[component(immutable)]#[require(ModifierType = ModifierType::MovementSpeed)] pub struct ModifierMovementSpeed(pub f32);
#[derive(Component, Clone, Default, Property, Modifier)]#[component(immutable)]#[require(ModifierType = ModifierType::EnergySupplyRange)] pub struct ModifierEnergySupplyRange(pub f32);

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
/// 
/// T - Modifier component (for example ModifierAttackRange)
/// U - Property component (for example AttackRange)
fn recalculate_on_modifier_inserted<T: Component + Modifier, U: Component + Property>(
    trigger: Trigger<OnInsert, T>,
    mut commands: Commands,
    modifiers: Query<(&T, &ModifierOf)>,
    modification_targets: Query<&Modifiers>,
) {
    let modifier_entity = trigger.target();
    let Ok((_, modifier_of)) = modifiers.get(modifier_entity) else { return; };
    let all_modifiers_list = modification_targets.get(modifier_of.0).unwrap();
    let new_value = all_modifiers_list.iter()
        .filter_map(|entity| modifiers.get(entity).ok())
        .map(|(modifier, _)| modifier.get())
        .sum();
    commands.entity(modifier_of.0).insert(U::new(new_value));
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
