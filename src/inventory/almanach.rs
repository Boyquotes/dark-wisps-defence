use bevy::prelude::*;
use bevy::utils::HashMap;
use crate::buildings::common::{BuildingType, TowerType};

#[derive(Resource)]
pub struct Almanach {
    building_costs: HashMap<BuildingType, i32>,
}

impl Almanach {
    pub fn get_building_cost(&self, building_type: BuildingType) -> i32 {
        *self.building_costs.get(&building_type).expect(format!("Building {building_type:?} not found in almanach").as_str())
    }
}

impl Default for Almanach {
    fn default() -> Self {
        let mut building_costs = [
            (BuildingType::MainBase, 0),
            (BuildingType::MiningComplex, 100),
            (BuildingType::EnergyRelay, 300),
            (BuildingType::ExplorationCenter, 500),
            (BuildingType::Tower(TowerType::Blaster), 150),
            (BuildingType::Tower(TowerType::Cannon), 250),
            (BuildingType::Tower(TowerType::RocketLauncher), 350),
        ].into_iter().collect();

        Self { building_costs }
    }
}