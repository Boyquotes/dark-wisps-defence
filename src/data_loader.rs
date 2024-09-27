use std::fs::File;

use serde::{Deserialize, Serialize};
use crate::prelude::*;

pub struct DataLoaderPlugin;
impl Plugin for DataLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load_data_system);
    }
}


#[derive(Serialize, Deserialize)]
struct Data {
    buildings: Vec<Buildings>,
}
#[derive(Serialize, Deserialize)]
struct Buildings {
    #[serde(rename = "type")] 
    building_type: BuildingType,
    name: String,
    grid_imprint: GridImprint,
    cost: Vec<Cost>,
}

fn load_data_system(
    mut almanach: ResMut<Almanach>,
) {
    let data: Data = serde_yaml::from_reader(File::open(format!("assets/data.yaml")).unwrap()).unwrap();
    data.buildings.into_iter().for_each(
        |Buildings { building_type, name, grid_imprint, cost }| almanach.add_building(building_type, name, cost, grid_imprint)
    );
}