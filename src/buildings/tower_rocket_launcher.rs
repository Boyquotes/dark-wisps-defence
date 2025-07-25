use lib_grid::grids::energy_supply::EnergySupplyGrid;
use lib_core::utils::angle_difference;

use crate::prelude::*;
use bevy::sprite::Anchor;
use crate::projectiles::rocket::BuilderRocket;
use crate::wisps::components::Wisp;

pub struct TowerRocketLauncherPlugin;
impl Plugin for TowerRocketLauncherPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(BuilderTowerRocketLauncher::on_add).add_systems(Update, (
                shooting_system.run_if(in_state(GameState::Running)),
            ));
    }
}

pub const TOWER_ROCKET_LAUNCHER_BASE_IMAGE: &str = "buildings/tower_rocket_launcher.png";

#[derive(Component)]
pub struct BuilderTowerRocketLauncher {
    grid_position: GridCoords,
}
impl BuilderTowerRocketLauncher {
    pub fn new(grid_position: GridCoords) -> Self { Self { grid_position } }

    pub fn on_add(
        trigger: Trigger<OnAdd, BuilderTowerRocketLauncher>,
        mut commands: Commands,
        builders: Query<&BuilderTowerRocketLauncher>,
        asset_server: Res<AssetServer>,
        almanach: Res<Almanach>,
        energy_supply_grid: Res<EnergySupplyGrid>,
    ) {
        let entity = trigger.target();
        let Ok(builder) = builders.get(entity) else { return; };
        
        let grid_imprint = almanach.get_building_info(BuildingType::Tower(TowerType::RocketLauncher)).grid_imprint;
        let tower_base_entity = commands.entity(entity)
            .remove::<BuilderTowerRocketLauncher>()
            .insert((
                Sprite {
                    image: asset_server.load(TOWER_ROCKET_LAUNCHER_BASE_IMAGE),
                    custom_size: Some(grid_imprint.world_size()),
                    ..Default::default()
                },
                MarkerTower,
                TowerRocketLauncher,
                builder.grid_position,
                Health::new(100),
                AttackRange(30),
                grid_imprint,
                TowerShootingTimer::from_seconds(2.0),
                TowerWispTarget::default(),
                TechnicalState{ has_energy_supply: energy_supply_grid.is_imprint_suppliable(builder.grid_position, grid_imprint), ..default() },
                TowerTopRotation { speed: 1.0, current_angle: 0. },
            )).id();
        let world_size = grid_imprint.world_size();
        let tower_top = commands.spawn((
            Sprite {
                image: asset_server.load("buildings/tower_rocket_launcher_top.png"),
                custom_size: Some(Vec2::new(world_size.x * 1.52 * 0.5, world_size.y * 0.5)),
                anchor: Anchor::Custom(Vec2::new(-0.20, 0.0)),
                ..Default::default()
            },
            ZDepth(Z_TOWER_TOP),
            MarkerTowerRotationalTop(tower_base_entity),
        )).id();
        commands.entity(entity).add_child(tower_top);
    }
}

pub fn shooting_system(
    mut commands: Commands,
    mut tower_rocket_launchers: Query<(&GridImprint, &Transform, &TechnicalState, &mut TowerShootingTimer, &mut TowerWispTarget, &TowerTopRotation), (With<TowerRocketLauncher>, Without<Wisp>)>,
    wisps: Query<&Transform, With<Wisp>>,
) {
    for (grid_imprint, transform, technical_state, mut timer, mut target, top_rotation) in tower_rocket_launchers.iter_mut() {
        if !technical_state.is_operational() { continue; }
        let TowerWispTarget::Wisp(target_wisp) = *target else { continue; };
        if !timer.0.finished() { continue; }

        let Ok(wisp_position) = wisps.get(target_wisp).map(|target| target.translation.xy()) else {
            // Target wisp does not exist anymore
            *target = TowerWispTarget::SearchForNewTarget;
            continue;
        };

        // Check if the tower top is facing the target
        let direction_to_target = wisp_position - transform.translation.xy();
        let target_angle = direction_to_target.y.atan2(direction_to_target.x);
        if angle_difference(target_angle, top_rotation.current_angle).abs() > std::f32::consts::PI / 72. { continue; }

        // Calculate transform offset in the direction we are aiming
        let tower_world_width = grid_imprint.world_size().x;
        let offset = Vec2::new(
            top_rotation.current_angle.cos() * tower_world_width * 0.4,
            top_rotation.current_angle.sin() * tower_world_width * 0.4,
        );
        let spawn_position = transform.translation.xy() + offset;

        let rocket_angle = Quat::from_rotation_z(top_rotation.current_angle);
        commands.spawn(BuilderRocket::new(spawn_position, rocket_angle, target_wisp));
        timer.0.reset();
    }
}
