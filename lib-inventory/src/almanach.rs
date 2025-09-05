use crate::lib_prelude::*;

pub mod almanach_prelude {
    pub use super::{Almanach, AlmanachBuildingInfo, AlmanachUpgradeInfo};
}

pub struct AlmanachPlugin;
impl Plugin for AlmanachPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(MapLoadingStage::ResetGridsAndResources), |mut commands: Commands| { commands.insert_resource(Almanach::default()); })
            .add_observer(AlmanachRequestPotentialUpgradesInsertion::on_trigger);
    }
}

#[derive(Resource, Default)]
pub struct Almanach {
    buildings: HashMap<BuildingType, AlmanachBuildingInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct AlmanachBuildingInfo {
    pub building_type: BuildingType,
    pub name: String,
    pub cost: Vec<Cost>,
    pub grid_imprint: GridImprint,
    pub upgrades: HashMap<ModifierType, AlmanachUpgradeInfo>,
    pub baseline: HashMap<ModifierType, f32>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AlmanachUpgradeInfo {
    pub upgrade_type: ModifierType,
    pub levels: Vec<AlmanachUpgradeLevelInfo>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AlmanachUpgradeLevelInfo {
    pub cost: Vec<Cost>,
    pub value: f32,
}

impl Almanach {
    pub fn get_building_info(&self, building_type: BuildingType) -> &AlmanachBuildingInfo {
        let info = self.buildings.get(&building_type).expect(format!("Building {building_type:?} not found in almanach").as_str());
        &info
    }
    pub fn add_building_info(&mut self, building_info: AlmanachBuildingInfo) {
        self.buildings.insert(building_info.building_type, building_info);
    }
}

#[derive(Event)]
pub struct AlmanachRequestPotentialUpgradesInsertion;
impl AlmanachRequestPotentialUpgradesInsertion {
    fn on_trigger(
        trigger: Trigger<Self>,
        mut commands: Commands,
        almanach: Res<Almanach>,
        buildings: Query<&BuildingType>,
    ) {
        let entity = trigger.target();
        let Ok(building_type) = buildings.get(entity) else { return; };

        commands.entity(entity).with_related_entities::<PotentialUpgradeOf>(|parent|
            almanach.get_building_info(*building_type).upgrades.values().for_each(|upgrade| {
                match upgrade.upgrade_type {
                    ModifierType::AttackSpeed => {
                        parent.spawn((ModifierAttackSpeed(upgrade.levels[0].value), ModifierSourceUpgrade{ current_level: 0, upgrade_info: upgrade.clone() }));
                    }
                    ModifierType::AttackRange => {
                        parent.spawn((ModifierAttackRange(upgrade.levels[0].value), ModifierSourceUpgrade{ current_level: 0, upgrade_info: upgrade.clone() }));
                    }
                    ModifierType::AttackDamage => {
                        parent.spawn((ModifierAttackDamage(upgrade.levels[0].value), ModifierSourceUpgrade{ current_level: 0, upgrade_info: upgrade.clone() }));
                    }
                    ModifierType::MaxHealth => {
                        parent.spawn((ModifierMaxHealth(upgrade.levels[0].value), ModifierSourceUpgrade{ current_level: 0, upgrade_info: upgrade.clone() }));
                    }
                    ModifierType::EnergySupplyRange => {
                        parent.spawn((ModifierEnergySupplyRange(upgrade.levels[0].value), ModifierSourceUpgrade{ current_level: 0, upgrade_info: upgrade.clone() }));
                    }
                    ModifierType::MovementSpeed => {
                        panic!("Building is trying to run away!");
                    }
                }
            })
        );
    }
}