use crate::lib_prelude::*;

pub mod almanach_prelude {
    pub use super::{Almanach, AlmanachBuildingInfo, AlmanachUpgradeInfo};
}

pub struct AlmanachPlugin;
impl Plugin for AlmanachPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<Almanach>()
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
    pub upgrades: Vec<AlmanachUpgradeInfo>,
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
            almanach.get_building_info(*building_type).upgrades.iter().for_each(|upgrade| {
                match upgrade.upgrade_type {
                    ModifierType::AttackSpeed => {
                        parent.spawn((ModifierAttackSpeed(upgrade.levels[0].value), ModifierSourceUpgrade{ level: 0, cost: upgrade.levels[0].cost.clone() }));
                    }
                    ModifierType::AttackRange => {
                        parent.spawn((ModifierAttackRange(upgrade.levels[0].value as usize), ModifierSourceUpgrade{ level: 0, cost: upgrade.levels[0].cost.clone() }));
                    }
                    ModifierType::AttackDamage => {
                        parent.spawn((ModifierAttackDamage(upgrade.levels[0].value as i32), ModifierSourceUpgrade{ level: 0, cost: upgrade.levels[0].cost.clone() }));
                    }
                    ModifierType::MaxHealth => {
                        parent.spawn((ModifierMaxHealth(upgrade.levels[0].value as i32), ModifierSourceUpgrade{ level: 0, cost: upgrade.levels[0].cost.clone() }));
                    }
                }
            })
        );
    }
}