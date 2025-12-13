use crate::lib_prelude::*;

pub mod modifiers_prelude {
    pub use super::*;
}

pub struct ModifiersPlugin;
impl Plugin for ModifiersPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_message::<RecalculateFromModifierBank>()
            .add_message::<LevelUpUpgradeMessage>()
            .add_systems(PreUpdate, (
                RecalculateFromModifierBank::process.run_if(on_message::<RecalculateFromModifierBank>),
                LevelUpUpgradeMessage::process.run_if(on_message::<LevelUpUpgradeMessage>),
            ))
            .add_observer(ModifiersBank::on_insert)
            ;
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
impl ModifierType {
    /// Inserts the corresponding value-holding component for this modifier type
    /// into the given `EntityCommands`, using `value` as the component's data.
    ///
    /// This is a convenience to map a `ModifierType` to its runtime sum component
    /// (e.g., `AttackSpeed`, `AttackRange`, etc.) and attach it to an entity.
    pub fn insert_value_component(&self, entity_commands: &mut EntityCommands, value: f32) {
        match self {
            Self::AttackSpeed => { entity_commands.insert(AttackSpeed::new(value)); }
            Self::AttackRange => { entity_commands.insert(AttackRange::new(value)); }
            Self::AttackDamage => { entity_commands.insert(AttackDamage::new(value)); }
            Self::MaxHealth => { entity_commands.insert(MaxHealth::new(value)); }
            Self::MovementSpeed => { entity_commands.insert(MovementSpeed::new(value)); }
            Self::EnergySupplyRange => { entity_commands.insert(EnergySupplyRange::new(value)); }
        }
    }
}

#[derive(Component, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModifierSource {
    Baseline,
    Upgrade{ level: usize },
}

#[derive(Message)]
pub struct RecalculateFromModifierBank {
    entity: Entity,
    modifier_type: ModifierType
}
impl RecalculateFromModifierBank {
    fn process(
        mut commands: Commands,
        mut reader: MessageReader<Self>,
        objects: Query<&ModifiersBank>,
    ) {
        for message in reader.read() {
            let Ok(modifiers_bank) = objects.get(message.entity) else { continue; };
            let modifier_type = message.modifier_type;
            let sum = modifiers_bank.bank[&modifier_type].values().copied().sum();
            modifier_type.insert_value_component(&mut commands.entity(message.entity), sum);
        }
    }
}

/// Operator for modifying a `ModifiersBank` component on an entity.
///
/// This struct provides a convenient API for adding, updating, or removing modifiers
/// from an entity's `ModifiersBank`. Changes made through this operator will automatically
/// trigger recalculation messages to update the corresponding value components.
pub struct ModifierBankOperator<'o, 'w> {
    entity: Entity,
    modifier_bank: &'o mut ModifiersBank,
    writer: &'o mut MessageWriter<'w, RecalculateFromModifierBank>,
}
impl<'a, 'w> ModifierBankOperator<'a, 'w>  {
    pub fn new(entity: Entity, modifier_bank: &'a mut ModifiersBank, writer: &'a mut MessageWriter<'w, RecalculateFromModifierBank>) -> Self {
        Self { entity, modifier_bank, writer }
    }
    

    /// Adds or updates a modifier value in the bank for the given type and source.
    ///
    /// If a modifier with the same type and source already exists, its value will be overwritten.
    ///
    /// # Arguments
    /// * `modifier_type` - The type of modifier (e.g., AttackSpeed, MaxHealth)
    /// * `modifier_source` - The source of the modifier (e.g., Baseline, Upgrade)
    /// * `value` - The value to set for this modifier
    pub fn add_modifier(&mut self, modifier_type: ModifierType, modifier_source: ModifierSource, value: f32) {
        self.modifier_bank.bank.entry(modifier_type).or_default().insert(modifier_source, value);
        self.writer.write(RecalculateFromModifierBank {
            entity: self.entity,
            modifier_type,
        });
    }

    /// Triggers a full recalculation for all modifier types in the bank.
    ///
    /// This sends recalculation messages for every modifier type that has at least one entry,
    /// causing all corresponding value components to be updated.
    pub fn trigger_full_recalc(&mut self) {
        for modifier_type in self.modifier_bank.bank.keys().copied() {
            self.writer.write(RecalculateFromModifierBank {
                entity: self.entity,
                modifier_type,
            });
        }
    }

}

/// A bank that stores all modifiers for an entity, organized by modifier type and source.
/// 
/// Each modifier type (e.g., AttackSpeed, MaxHealth) can have multiple sources
/// (e.g., Baseline, Upgrade) with their respective values. The final value for
/// each modifier type is calculated as the sum of all sources.
#[derive(Component, Default)]
pub struct ModifiersBank {
    bank: HashMap<ModifierType, HashMap<ModifierSource, f32>>,
}
impl ModifiersBank {
    /// Creates a ModifiersBank populated with baseline values from AlmanachBuildingInfo.
    pub fn from_baseline(baseline: &HashMap<ModifierType, f32>) -> Self {
        let mut bank = HashMap::default();
        for (modifier_type, value) in baseline.iter() {
            let mut sources = HashMap::default();
            sources.insert(ModifierSource::Baseline, *value);
            bank.insert(*modifier_type, sources);
        }
        Self { bank }
    }

    /// Returns the current sum for a given modifier type.
    pub fn get_sum(&self, modifier_type: ModifierType) -> f32 {
        self.bank.get(&modifier_type)
            .map(|sources| sources.values().copied().sum())
            .unwrap_or(0.0)
    }

    fn on_insert(
        trigger: On<Insert, Self>,
        mut writer: MessageWriter<RecalculateFromModifierBank>,
        mut banks: Query<&mut ModifiersBank>,
    ) {
        let entity = trigger.entity;
        let Ok(mut bank) = banks.get_mut(entity) else { return; };
        let mut operator = ModifierBankOperator::new(entity, &mut bank, &mut writer);
        operator.trigger_full_recalc();
    }
}

////////////////////
////  UPGRADES  ////
////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UpgradeType {
    Modifier(ModifierType)
}

pub struct UpgradeRuntimeInfo {
    pub current_level: usize,
    pub static_info: AlmanachUpgradeInfo,
}

#[derive(Component)]
pub struct Upgrades {
    pub upgrades: HashMap<UpgradeType, UpgradeRuntimeInfo>,
}
impl Upgrades {
    /// Creates an Upgrades component from almanach upgrade info.
    /// All upgrades start at level 0 (not yet purchased).
    pub fn from_almanach(almanach_upgrades: &HashMap<UpgradeType, AlmanachUpgradeInfo>) -> Self {
        let upgrades = almanach_upgrades.iter().map(|(upgrade_type, info)| {
            (*upgrade_type, UpgradeRuntimeInfo {
                current_level: 0,
                static_info: info.clone(),
            })
        }).collect();
        Self { upgrades }
    }

    /// Returns the total number of upgrades purchased across all upgrade types.
    pub fn total_upgrades_purchased(&self) -> usize {
        self.upgrades.values().map(|info| info.current_level).sum()
    }

    /// Returns the maximum number of upgrades available across all upgrade types.
    pub fn total_upgrades_available(&self) -> usize {
        self.upgrades.values().map(|info| info.static_info.levels.len()).sum()
    }
}

#[derive(Message)]
pub struct LevelUpUpgradeMessage {
    pub entity: Entity,
    pub upgrade_type: UpgradeType,
}
impl LevelUpUpgradeMessage {
    fn process(
        mut reader: MessageReader<Self>,
        mut writer: MessageWriter<RecalculateFromModifierBank>,
        mut objects: Query<(Entity, &mut Upgrades, &mut ModifiersBank)>,
    ) {
        for message in reader.read() {
            let Ok((entity, mut upgrades, mut modifiers_bank)) = objects.get_mut(message.entity) else { continue; };
            let upgrade_type = message.upgrade_type;
            let Some(upgrade_runtime_info) = upgrades.upgrades.get_mut(&upgrade_type) else { continue; };
            if upgrade_runtime_info.current_level >= upgrade_runtime_info.static_info.levels.len() - 1 { continue; }
            upgrade_runtime_info.current_level += 1;
            match upgrade_type {
                UpgradeType::Modifier(modifier_type) => {
                    let mut operator = ModifierBankOperator::new(entity, &mut modifiers_bank, &mut writer);
                    operator.add_modifier(modifier_type, ModifierSource::Upgrade { level: upgrade_runtime_info.current_level }, upgrade_runtime_info.static_info.levels[upgrade_runtime_info.current_level].value);
                }
            }
        }

    }
}
impl Command for LevelUpUpgradeMessage {
    fn apply(self, world: &mut World) {
        let mut messages = world.resource_mut::<Messages<Self>>();
        messages.write(self);
    }
}
