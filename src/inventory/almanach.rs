use crate::prelude::*;

pub struct AlmanachPlugin;
impl Plugin for AlmanachPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Almanach>();
    }
}

#[derive(Resource, Default)]
pub struct Almanach {
    buildings: HashMap<BuildingType, AlmanachBuildingInfo>,
}

pub struct AlmanachBuildingInfo {
    pub name: String,
    pub cost: Vec<Cost>,
    pub grid_imprint: GridImprint,
}

impl Almanach {
    pub fn get_building_info(&self, building_type: BuildingType) -> &AlmanachBuildingInfo {
        let info = self.buildings.get(&building_type).expect(format!("Building {building_type:?} not found in almanach").as_str());
        &info
    }
    pub fn add_building(&mut self, building_type: BuildingType, name: String, cost: Vec<Cost>, grid_imprint: GridImprint) {
        self.buildings.insert(building_type, AlmanachBuildingInfo { name, cost, grid_imprint });
    }
}