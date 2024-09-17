use bevy::math::Vec3Swizzles;
use crate::prelude::*;
use crate::buildings::common::{BuildingType, TowerType};
use crate::buildings::common_components::{Building, MarkerTower, TowerWispTarget, TowerShootingTimer, TechnicalState, TowerRange, TowerTopRotation, MarkerTowerRotationalTop};
use crate::grids::energy_supply::EnergySupplyGrid;
use crate::projectiles::laser_dart::BuilderLaserDart;
use crate::utils::math::angle_difference;
use crate::wisps::components::Wisp;

pub struct TowerBlasterPlugin;
impl Plugin for TowerBlasterPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<BuilderTowerBlaster>()
            .add_systems(PostUpdate, (
                BuilderTowerBlaster::spawn_system,
            )).add_systems(Update, (
                shooting_system,
            ));
    }
}

pub const TOWER_BLASTER_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: 2, height: 2 };
pub const TOWER_BLASTER_BASE_IMAGE: &str = "buildings/tower_blaster.png";
pub const TOWER_BLASTER_TOP_IMAGE: &str = "buildings/tower_blaster_top.png";

#[derive(Component)]
pub struct MarkerTowerBlaster;

#[derive(Event)]
pub struct BuilderTowerBlaster {
    pub entity: Entity,
    pub grid_position: GridCoords,
}
impl BuilderTowerBlaster {
    pub fn new(entity: Entity, grid_position: GridCoords) -> Self {
        Self { entity, grid_position }
    }
    pub fn spawn_system(
        mut commands: Commands,
        mut events: EventReader<BuilderTowerBlaster>,
        asset_server: Res<AssetServer>,
        energy_supply_grid: Res<EnergySupplyGrid>,
    ) {
        for &BuilderTowerBlaster{ entity, grid_position } in events.read() {
            let (tower_base, tower_top) = get_tower_blaster_sprite_bundle(&asset_server, grid_position);
            let tower_base_entity = commands.entity(entity).insert((
                tower_base,
                MarkerTower,
                MarkerTowerBlaster,
                grid_position,
                Health(100),
                TowerRange(15),
                Building::from(BuildingType::Tower(TowerType::Blaster)),
                TowerShootingTimer::from_seconds(0.2),
                TowerWispTarget::default(),
                TechnicalState{ has_energy_supply: energy_supply_grid.is_imprint_suppliable(grid_position, TOWER_BLASTER_GRID_IMPRINT) },
                TowerTopRotation { speed: 10.0, current_angle: 0. },
            )).id();
            commands.spawn((
                tower_top,
                MarkerTowerRotationalTop(tower_base_entity.into()),
            ));
        }
    }
}
impl Command for BuilderTowerBlaster {
    fn apply(self, world: &mut World) {
        world.send_event(self);
    }
}

/// Returns (tower base sprite bundle, tower top sprite bundle)
pub fn get_tower_blaster_sprite_bundle(asset_server: &AssetServer, grid_position: GridCoords) -> (SpriteBundle, SpriteBundle) {
    let world_position = grid_position.to_world_position_centered(TOWER_BLASTER_GRID_IMPRINT);
    let world_size = TOWER_BLASTER_GRID_IMPRINT.world_size();
    let tower_base = SpriteBundle {
        sprite: Sprite {
            custom_size: Some(world_size),
            ..Default::default()
        },
        texture: asset_server.load(TOWER_BLASTER_BASE_IMAGE),
        transform: Transform::from_translation(world_position.extend(Z_BUILDING)),
        ..Default::default()
    };

    let tower_top = SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(world_size.x * 1.52 * 0.5, world_size.y * 0.5)),
            ..Default::default()
        },
        texture: asset_server.load(TOWER_BLASTER_TOP_IMAGE),
        transform: Transform::from_translation(world_position.extend(Z_TOWER_TOP)),
        ..Default::default()
    };
    (tower_base, tower_top)
}

pub fn shooting_system(
    mut commands: Commands,
    mut tower_blasters: Query<(&Transform, &TechnicalState, &mut TowerShootingTimer, &mut TowerWispTarget, &TowerTopRotation), With<MarkerTowerBlaster>>,
    wisps: Query<&Transform, With<Wisp>>,
) {
    for (transform, technical_state, mut timer, mut target, top_rotation) in tower_blasters.iter_mut() {
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
        if angle_difference(target_angle, top_rotation.current_angle).abs() > std::f32::consts::PI / 36. { continue; }

        // Calculate transform offset in the direction we are aiming
        let tower_world_width = TOWER_BLASTER_GRID_IMPRINT.world_size().x;
        let offset = Vec2::new(
            top_rotation.current_angle.cos() * tower_world_width * 0.4,
            top_rotation.current_angle.sin() * tower_world_width * 0.4,
        );
        let spawn_position = transform.translation.xy() + offset;

        commands.add(BuilderLaserDart::new(spawn_position, target_wisp, (wisp_position - spawn_position).normalize()));
        timer.0.reset();
    }
}