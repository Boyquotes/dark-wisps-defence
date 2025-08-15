use crate::map_objects::dark_ore::{
    DarkOre,
    dark_ore_area_scanner::{DarkOreAreaScanner, DarkOreInRange},
};
use crate::prelude::*;
use crate::ui::indicators::{IndicatorDisplay, IndicatorType, Indicators};

pub struct MiningComplexPlugin;
impl Plugin for MiningComplexPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                mine_ore_system.run_if(in_state(GameState::Running)),
            ))
            .add_observer(BuilderMiningComplex::on_add);
    }
}

pub const MINING_COMPLEX_BASE_IMAGE: &str = "buildings/mining_complex.png";


#[derive(Component)]
pub struct MiningComplexDeliveryTimer(pub Timer);

#[derive(Component)]
pub struct BuilderMiningComplex {
    grid_position: GridCoords,
}
impl BuilderMiningComplex {
    pub fn new(grid_position: GridCoords) -> Self {
        Self { grid_position }
    }

    pub fn on_add(
        trigger: Trigger<OnAdd, BuilderMiningComplex>,
        mut commands: Commands,
        builders: Query<&BuilderMiningComplex>,
        asset_server: Res<AssetServer>,
        almanach: Res<Almanach>,
    ) {
        let entity = trigger.target();
        let Ok(builder) = builders.get(entity) else { return; };
        
        let building_info = almanach.get_building_info(BuildingType::MiningComplex);
        let grid_imprint = building_info.grid_imprint;
        commands.entity(entity)
            .remove::<BuilderMiningComplex>()
            .insert((
                MiningComplex,
                Sprite {
                    image: asset_server.load(MINING_COMPLEX_BASE_IMAGE),
                    custom_size: Some(grid_imprint.world_size()),
                    ..Default::default()
                },
                builder.grid_position,
                grid_imprint,
                NeedsPower::default(),
                DarkOreAreaScanner{range_imprint: grid_imprint},
                MiningComplexDeliveryTimer(Timer::from_seconds(1.0, TimerMode::Repeating)),
                related![Modifiers[
                    (ModifierMaxHealth::from_baseline(building_info), ModifierSourceBaseline),
                ]],
                related![Indicators[
                    IndicatorType::NoPower,
                ]],
                children![
                    IndicatorDisplay::default(),
                ],
            ));
    }
}

fn mine_ore_system(
    mut stock: ResMut<Stock>,
    mut mining_complexes: Query<(&mut MiningComplexDeliveryTimer, &DarkOreInRange), (With<MiningComplex>, With<HasPower>)>,
    mut dark_ores: Query<&mut DarkOre>,
    time: Res<Time>,
) {
    let mut rng = nanorand::tls_rng();
    for (mut timer, ore_in_range) in mining_complexes.iter_mut() {
        let ore_in_range = &ore_in_range.0;
        if ore_in_range.is_empty() { continue; }
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            let ore_index = rng.generate_range(0..ore_in_range.len());
            let ore_entity = ore_in_range[ore_index];
            if let Ok(mut dark_ore) = dark_ores.get_mut(ore_entity) {
                let mined_amount = std::cmp::min(dark_ore.amount, 100);
                stock.add(ResourceType::DarkOre, mined_amount);
                dark_ore.amount -= mined_amount;
            }
        }
    }
}