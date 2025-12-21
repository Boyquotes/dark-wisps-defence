use crate::lib_prelude::*;

pub mod almanach_prelude {
    pub use super::{Almanach, AlmanachBuildingInfo, AlmanachUpgradeInfo};
}

pub struct AlmanachPlugin;
impl Plugin for AlmanachPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(MapLoadingStage::Init), |mut commands: Commands| { commands.insert_resource(Almanach::default()); })
;
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
    pub upgrades: HashMap<UpgradeType, AlmanachUpgradeInfo>,
    pub baseline: HashMap<ModifierType, f32>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AlmanachUpgradeInfo {
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
