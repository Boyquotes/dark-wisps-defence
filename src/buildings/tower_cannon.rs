use crate::prelude::*;
use crate::ui::indicators::{IndicatorDisplay, IndicatorType, Indicators};
use crate::projectiles::cannonball::BuilderCannonball;
use crate::wisps::components::Wisp;
use crate::wisps::spawning::WISP_GRID_IMPRINT;

pub struct TowerCannonPlugin;
impl Plugin for TowerCannonPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(BuilderTowerCannon::on_add).add_systems(Update, (
                shooting_system.run_if(in_state(GameState::Running)),
            ));
    }
}

pub const TOWER_CANNON_BASE_IMAGE: &str = "buildings/tower_cannon.png";

#[derive(Component)]
pub struct BuilderTowerCannon {
    grid_position: GridCoords,
}

impl BuilderTowerCannon {
    pub fn new(grid_position: GridCoords) -> Self {
        Self { grid_position }
    }

    pub fn on_add(
        trigger: Trigger<OnAdd, BuilderTowerCannon>,
        mut commands: Commands,
        builders: Query<&BuilderTowerCannon>,
        asset_server: Res<AssetServer>,
        almanach: Res<Almanach>,
    ) {
        let entity = trigger.target();
        let Ok(builder) = builders.get(entity) else { return; };
        
        let building_info = almanach.get_building_info(BuildingType::Tower(TowerType::Cannon));
        let grid_imprint = building_info.grid_imprint;
        commands.entity(entity)
            .remove::<BuilderTowerCannon>()
            .insert((
                TowerCannon,
                Sprite {
                    image: asset_server.load(TOWER_CANNON_BASE_IMAGE),
                    custom_size: Some(grid_imprint.world_size()),
                    ..Default::default()
                },
                Tower,
                builder.grid_position,
                grid_imprint,
                NeedsPower::default(),
                related![Modifiers[
                    (ModifierAttackRange::from_baseline(building_info), ModifierSourceBaseline),
                    (ModifierAttackSpeed::from_baseline(building_info), ModifierSourceBaseline),
                    (ModifierAttackDamage::from_baseline(building_info), ModifierSourceBaseline),
                    (ModifierMaxHealth::from_baseline(building_info), ModifierSourceBaseline),
                ]],
                related![Indicators[
                    IndicatorType::NoPower,
                    IndicatorType::DisabledByPlayer,
                ]],
                children![
                    IndicatorDisplay::default(),
                ],
            ));
        commands.trigger_targets(lib_inventory::almanach::AlmanachRequestPotentialUpgradesInsertion, entity);
    }
}

pub fn shooting_system(
    mut commands: Commands,
    mut tower_cannons: Query<(&Transform, &mut TowerShootingTimer, &mut TowerWispTarget, &AttackDamage), (With<TowerCannon>, With<HasPower>, Without<DisabledByPlayer>)>,
    wisps: Query<(&GridPath, &GridCoords), With<Wisp>>,
) {
    for (transform, mut timer, mut target, attack_damage) in tower_cannons.iter_mut() {
        let TowerWispTarget::Wisp(target_wisp) = *target else { continue; };
        if !timer.0.finished() { continue; }

        let Ok((wisp_grid_path, wisp_coords)) = wisps.get(target_wisp) else {
            // Target wisp does not exist anymore
            *target = TowerWispTarget::SearchForNewTarget;
            continue;
        };

        // If wisps has path, target the next path position. Otherwise, target the wisp's current position.
        let target_world_position = wisp_grid_path.next_in_path().map_or(
            wisp_coords.to_world_position_centered(WISP_GRID_IMPRINT),
            |coords| coords.to_world_position_centered(WISP_GRID_IMPRINT)
        );

        commands.spawn(BuilderCannonball::new(transform.translation.xy(), target_world_position, attack_damage.clone()));
        timer.0.reset();
    }
}
