use crate::lib_prelude::*;

pub struct StatsPlugin;
impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(MapLoadingStage::ResetGridsAndResources), |mut commands: Commands| { commands.insert_resource(StatsWispsKilled::default()); });
    }
}

#[derive(Resource, Default)]
pub struct StatsWispsKilled(pub usize);