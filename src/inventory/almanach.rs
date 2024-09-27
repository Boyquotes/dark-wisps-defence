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

struct AlmanachBuildingInfo {
    pub name: String,
    pub cost: Vec<Cost>,
    pub grid_imprint: GridImprint,
}

impl Almanach {
    pub fn get_building_cost(&self, building_type: BuildingType) -> &Vec<Cost> {
        let info = self.buildings.get(&building_type).expect(format!("Building {building_type:?} not found in almanach").as_str());
        &info.cost
    }
    pub fn get_building_grid_imprint(&self, building_type: BuildingType) -> GridImprint {
        let info = self.buildings.get(&building_type).expect(format!("Building {building_type:?} not found in almanach").as_str());
        info.grid_imprint
    }
    pub fn get_building_name(&self, building_type: BuildingType) -> &str {
        let info = self.buildings.get(&building_type).expect(format!("Building {building_type:?} not found in almanach").as_str());
        &info.name
    }
    pub fn add_building(&mut self, building_type: BuildingType, name: String, cost: Vec<Cost>, grid_imprint: GridImprint) {
        self.buildings.insert(building_type, AlmanachBuildingInfo { name, cost, grid_imprint });
    }
}