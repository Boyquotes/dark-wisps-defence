use std::fs::File;
    
use crate::prelude::*;

pub struct DataLoaderPlugin;
impl Plugin for DataLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MapLoadingStage::ApplyMap), load_data_system);
    }
}


#[derive(Serialize, Deserialize)]
struct Data {
    buildings: Vec<AlmanachBuildingInfo>,
}

fn load_data_system(
    mut almanach: ResMut<Almanach>,
) {
    let data: Data = serde_yaml::from_reader(File::open(format!("assets/data.yaml")).unwrap()).unwrap();
    data.buildings.into_iter().for_each(
        |building_info| almanach.add_building_info(building_info)
    );
}