use crate::prelude::*;
use bevy::sprite::Anchor;
use crate::grids::energy_supply::EnergySupplyGrid;
use crate::projectiles::rocket::BuilderRocket;
use crate::utils::math::angle_difference;
use crate::wisps::components::Wisp;

pub struct TowerRocketLauncherPlugin;
impl Plugin for TowerRocketLauncherPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<BuilderTowerRocketLauncher>()
            .add_systems(PostUpdate, (
                BuilderTowerRocketLauncher::spawn_system,
            )).add_systems(Update, (
                shooting_system,
            ));
    }
}

pub const TOWER_ROCKET_LAUNCHER_BASE_IMAGE: &str = "buildings/tower_rocket_launcher.png";

#[derive(Component)]
pub struct MarkerTowerRocketLauncher;

#[derive(Event)]
pub struct BuilderTowerRocketLauncher {
    pub entity: Entity,
    pub grid_position: GridCoords,
}
impl BuilderTowerRocketLauncher {
    pub fn new(entity: Entity, grid_position: GridCoords) -> Self { Self { entity, grid_position } }
    pub fn spawn_system(
        mut commands: Commands,
        mut events: EventReader<BuilderTowerRocketLauncher>,
        asset_server: Res<AssetServer>,
        almanach: Res<Almanach>,
        energy_supply_grid: Res<EnergySupplyGrid>,
    ) {
        for &BuilderTowerRocketLauncher{ entity, grid_position } in events.read() {
            let grid_imprint = almanach.get_building_grid_imprint(BuildingType::Tower(TowerType::RocketLauncher));
            let (tower_base, tower_top) = get_tower_rocket_launcher_sprite_bundle(&asset_server, grid_position, grid_imprint);
            let tower_base_entity = commands.entity(entity).insert((
                tower_base,
                MarkerTower,
                MarkerTowerRocketLauncher,
                grid_position,
                Health(100),
                TowerRange(30),
                Building,
                BuildingType::Tower(TowerType::RocketLauncher),
                grid_imprint,
                TowerShootingTimer::from_seconds(2.0),
                TowerWispTarget::default(),
                TechnicalState{ has_energy_supply: energy_supply_grid.is_imprint_suppliable(grid_position, grid_imprint), ..default() },
                TowerTopRotation { speed: 1.0, current_angle: 0. },
            )).id();
            commands.spawn((
                tower_top,
                MarkerTowerRotationalTop(tower_base_entity.into()),
            ));
        }
    }
}
impl Command for BuilderTowerRocketLauncher {
    fn apply(self, world: &mut World) {
        world.send_event(self);
    }
}

pub fn get_tower_rocket_launcher_sprite_bundle(asset_server: &AssetServer, coords: GridCoords, grid_imprint: GridImprint) -> (SpriteBundle, SpriteBundle) {
    let world_position = coords.to_world_position_centered(grid_imprint);
    let world_size = grid_imprint.world_size();
    let tower_base = SpriteBundle {
        sprite: Sprite {
            custom_size: Some(world_size),
            ..Default::default()
        },
        texture: asset_server.load(TOWER_ROCKET_LAUNCHER_BASE_IMAGE),
        transform: Transform::from_translation(world_position.extend(Z_BUILDING)),
        ..Default::default()
    };
    let tower_top = SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(world_size.x * 1.52 * 0.5, world_size.y * 0.5)),
            anchor: Anchor::Custom(Vec2::new(-0.20, 0.0)),
            ..Default::default()
        },
        texture: asset_server.load("buildings/tower_rocket_launcher_top.png"),
        transform: Transform::from_translation(world_position.extend(Z_TOWER_TOP)),
        ..Default::default()
    };
    (tower_base, tower_top)
}

pub fn shooting_system(
    mut commands: Commands,
    mut tower_rocket_launchers: Query<(&GridImprint, &Transform, &TechnicalState, &mut TowerShootingTimer, &mut TowerWispTarget, &TowerTopRotation), (With<MarkerTowerRocketLauncher>, Without<Wisp>)>,
    wisps: Query<&Transform, With<Wisp>>,
) {
    for (grid_imprint, transform, technical_state, mut timer, mut target, top_rotation) in tower_rocket_launchers.iter_mut() {
        if !technical_state.is_operational() { continue; }
        let TowerWispTarget::Wisp(target_wisp) = *target else { continue; };
        if !timer.0.finished() { continue; }

        let Ok(wisp_position) = wisps.get(*target_wisp).map(|target| target.translation.xy()) else {
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
        commands.add(BuilderRocket::new(spawn_position, rocket_angle, target_wisp));
        timer.0.reset();
    }
}
