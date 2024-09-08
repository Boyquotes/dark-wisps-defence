use crate::prelude::*;

pub struct StatsPlugin;
impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<StatsWispsKilled>();
    }
}

#[derive(Resource, Default)]
pub struct StatsWispsKilled(pub usize);