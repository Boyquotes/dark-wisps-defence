use lib_grid::grids::energy_supply::EnergySupplyGrid;

use crate::prelude::*;
use crate::projectiles::laser_dart::BuilderLaserDart;
use crate::utils::math::angle_difference;
use crate::wisps::components::Wisp;

pub struct TowerBlasterPlugin;
impl Plugin for TowerBlasterPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(BuilderTowerBlaster::on_add).add_systems(Update, (
                shooting_system.run_if(in_state(GameState::Running)),
            ));
    }
}

pub const TOWER_BLASTER_BASE_IMAGE: &str = "buildings/tower_blaster.png";
pub const TOWER_BLASTER_TOP_IMAGE: &str = "buildings/tower_blaster_top.png";

#[derive(Component)]
pub struct TowerBlaster;

#[derive(Component)]
pub struct BuilderTowerBlaster {
    grid_position: GridCoords,
}
impl BuilderTowerBlaster {
    pub fn new(grid_position: GridCoords) -> Self {
        Self { grid_position }
    }

    pub fn on_add(
        trigger: Trigger<OnAdd, BuilderTowerBlaster>,
        mut commands: Commands,
        builders: Query<&BuilderTowerBlaster>,
        asset_server: Res<AssetServer>,
        almanach: Res<Almanach>,
        energy_supply_grid: Res<EnergySupplyGrid>,
    ) {
        let entity = trigger.target();
        let Ok(builder) = builders.get(entity) else { return; };
        
        let grid_imprint = almanach.get_building_info(BuildingType::Tower(TowerType::Blaster)).grid_imprint;
        let tower_base_entity = commands.entity(entity)
            .remove::<BuilderTowerBlaster>()
            .insert((
                Sprite {
                    image: asset_server.load(TOWER_BLASTER_BASE_IMAGE),
                    custom_size: Some(grid_imprint.world_size()),
                    ..Default::default()
                },
                Transform::from_translation(builder.grid_position.to_world_position_centered(grid_imprint).extend(Z_BUILDING)),
                MarkerTower,
                TowerBlaster,
                builder.grid_position,
                Health::new(100),
                TowerRange(15),
                Building,
                BuildingType::Tower(TowerType::Blaster),
                grid_imprint,
                TowerShootingTimer::from_seconds(0.2),
                TowerWispTarget::default(),
                TechnicalState{ has_energy_supply: energy_supply_grid.is_imprint_suppliable(builder.grid_position, grid_imprint), ..default() },
                TowerTopRotation { speed: 10.0, current_angle: 0. },
            )).id();
        let world_size = grid_imprint.world_size();
        let tower_top = commands.spawn((
            Sprite {
                image: asset_server.load(TOWER_BLASTER_TOP_IMAGE),
                custom_size: Some(Vec2::new(world_size.x * 1.52 * 0.5, world_size.y * 0.5)),
                ..Default::default()
            },
            Transform::from_translation(Vec3::new(0., 0., Z_TOWER_TOP)),
            MarkerTowerRotationalTop(tower_base_entity.into()),
        )).id();
        commands.entity(entity).add_child(tower_top);
    }
}

pub fn shooting_system(
    mut commands: Commands,
    mut tower_blasters: Query<(&GridImprint, &Transform, &TechnicalState, &mut TowerShootingTimer, &mut TowerWispTarget, &TowerTopRotation), With<TowerBlaster>>,
    wisps: Query<&Transform, With<Wisp>>,
) {
    for (grid_imprint, transform, technical_state, mut timer, mut target, top_rotation) in tower_blasters.iter_mut() {
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
        if angle_difference(target_angle, top_rotation.current_angle).abs() > std::f32::consts::PI / 36. { continue; }

        // Calculate transform offset in the direction we are aiming
        let tower_world_width = grid_imprint.world_size().x;
        let offset = Vec2::new(
            top_rotation.current_angle.cos() * tower_world_width * 0.4,
            top_rotation.current_angle.sin() * tower_world_width * 0.4,
        );
        let spawn_position = transform.translation.xy() + offset;

        commands.spawn(BuilderLaserDart::new(spawn_position, target_wisp, (wisp_position - spawn_position).normalize()));
        timer.0.reset();
    }
}