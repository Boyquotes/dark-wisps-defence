use lib_grid::grids::energy_supply::{EnergySupplyGrid, SupplierEnergy};
use lib_grid::grids::obstacles::{BelowField, Field, ObstacleGrid};
use lib_grid::grids::wisps::WispsGrid;
use lib_grid::search::targetfinding::target_find_closest_wisp;
use lib_core::utils::angle_difference;

use crate::effects::explosions::BuilderExplosion;
use crate::prelude::*;
use crate::ui::grid_object_placer::GridObjectPlacer;
use crate::wisps::components::Wisp;
use super::{
    energy_relay::BuilderEnergyRelay,
    exploration_center::BuilderExplorationCenter,
    mining_complex::BuilderMiningComplex,
    tower_blaster::BuilderTowerBlaster,
    tower_emitter::BuilderTowerEmitter,
    tower_cannon::BuilderTowerCannon,
    tower_rocket_launcher::BuilderTowerRocketLauncher,
};

pub struct CommonSystemsPlugin;
impl Plugin for CommonSystemsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(PreUpdate, tick_shooting_timers_system.run_if(in_state(GameState::Running)))
            .add_systems(Update,(
                onclick_building_spawn_system.run_if(in_state(UiInteraction::PlaceGridObject)),
                (
                    check_energy_supply_system,
                    targeting_system,
                    rotate_tower_top_system,
                    rotational_aiming_system,
                    damage_control_system,
                ).run_if(in_state(GameState::Running)),
            ));
    }
}

pub fn onclick_building_spawn_system(
    mut commands: Commands,
    mut obstacle_grid: ResMut<ObstacleGrid>,
    mouse: Res<ButtonInput<MouseButton>>,
    mouse_info: Res<MouseInfo>,
    almanach: Res<Almanach>,
    mut stock: ResMut<Stock>,
    grid_object_placer: Query<(&GridObjectPlacer, &GridImprint)>,
    main_base: Query<(Entity, &GridCoords), With<MainBase>>,
) {
    let mouse_coords = mouse_info.grid_coords;
    if mouse_info.is_over_ui || !mouse.just_released(MouseButton::Left) { return; }
    let Ok((grid_object_placer, grid_imprint)) = grid_object_placer.single() else { return; };
    let GridObjectPlacer::Building(building_type) = grid_object_placer else { return; };
    // Grid Placement Validation
    if !mouse_coords.is_imprint_in_bounds(grid_imprint, obstacle_grid.bounds())
        || !obstacle_grid.query_building_placement(mouse_coords, *building_type, *grid_imprint) { return; }
    // Payment
    let building_costs = &almanach.get_building_info(*building_type).cost;
    if !stock.try_pay_costs(building_costs) { println!("Not enough dark ore"); return; }
    // Creation
    // ---
    enum GridAction {
        Imprint(Entity),
        ImprintMiningComplex(Entity),
        Reprint{entity: Entity, old_coords: GridCoords},
    }
    // ---
    let grid_action = match building_type {
        BuildingType::EnergyRelay => {
            let entity = commands.spawn(BuilderEnergyRelay::new(mouse_coords)).id();
            GridAction::Imprint(entity)
        }
        BuildingType::ExplorationCenter => {
            let entity = commands.spawn(BuilderExplorationCenter::new(mouse_coords)).id();
            GridAction::Imprint(entity)
        }
        BuildingType::Tower(TowerType::Blaster) => {
            let entity = commands.spawn(BuilderTowerBlaster::new(mouse_coords)).id();
            GridAction::Imprint(entity)
        },
        BuildingType::Tower(TowerType::Cannon) => {
            let entity = commands.spawn(BuilderTowerCannon::new(mouse_coords)).id();
            GridAction::Imprint(entity)
        },
        BuildingType::Tower(TowerType::RocketLauncher) => {
            let entity = commands.spawn(BuilderTowerRocketLauncher::new(mouse_coords)).id();
            GridAction::Imprint(entity)
        },
        BuildingType::Tower(TowerType::Emitter) => {
            let entity = commands.spawn(BuilderTowerEmitter::new(mouse_coords)).id();
            GridAction::Imprint(entity)
        },
        BuildingType::MainBase => {
            let Ok((main_base_entity, main_base_coords)) = main_base.single() else { return; };
            commands.entity(main_base_entity).insert(mouse_coords);
            GridAction::Reprint{entity: main_base_entity, old_coords: *main_base_coords}
        },
        BuildingType::MiningComplex => {
            let entity = commands.spawn(BuilderMiningComplex::new(mouse_coords)).id();
            GridAction::ImprintMiningComplex(entity)
        },
    };
    match grid_action {
        GridAction::Imprint(entity) => obstacle_grid.imprint(mouse_coords, Field::Building(entity, *building_type, default()), *grid_imprint),
        GridAction::ImprintMiningComplex(entity) => obstacle_grid.imprint_custom(mouse_coords, *grid_imprint, &|field| {
            // Retain information about dark ore that will be below the mining complex
            let below_field = match field {
                Field::Empty => BelowField::Empty,
                Field::DarkOre(entity) => BelowField::DarkOre(*entity),
                _ => panic!("imprint_mining_complex() can only be used with an Empty or DarkOre Field"),
            };
            *field = Field::Building(entity, BuildingType::MiningComplex, below_field);
            
        }),
        GridAction::Reprint{entity, old_coords} => obstacle_grid.reprint(
            old_coords, mouse_coords, Field::Building(entity, *building_type, default()), *grid_imprint
        ),
    }
}

pub fn targeting_system(
    obstacle_grid: Res<ObstacleGrid>,
    wisps_grid: Res<WispsGrid>,
    mut towers: Query<(&GridCoords, &GridImprint, &TechnicalState, &AttackRange, &mut TowerWispTarget), (With<MarkerTower>, Without<Wisp>)>,
    wisps: Query<&GridCoords, (With<Wisp>, Without<MarkerTower>)>,
) {
    for (coords, grid_imprint, technical_state, range, mut target) in towers.iter_mut() {
        if !technical_state.has_energy_supply { continue; }
        match *target {
            TowerWispTarget::Wisp(wisp_entity) => {
                if let Ok(wisp_coords) = wisps.get(wisp_entity) {
                    // Check if wisp is still in range. For now we use Manhattan distance to check. This may not be correct for all tower types.
                    if coords.manhattan_distance(wisp_coords) as usize <= range.0 { continue; }
                }
            },
            TowerWispTarget::NoValidTargets(grid_version) => {
                if grid_version == wisps_grid.version {
                    continue;
                }
            },
            TowerWispTarget::SearchForNewTarget => {},
        }
        if let Some((_a, target_wisp)) = target_find_closest_wisp(
            &obstacle_grid,
            &wisps_grid,
            grid_imprint.covered_coords(*coords),
            range.0,
            true,
        ) {
            *target = TowerWispTarget::Wisp(target_wisp);
        } else {
            *target = TowerWispTarget::NoValidTargets(wisps_grid.version);
        }
    }
}

pub fn tick_shooting_timers_system(
    mut shooting_timers: Query<&mut TowerShootingTimer>,
    time: Res<Time>,
) {
    shooting_timers.iter_mut().for_each(|mut timer| { timer.0.tick(time.delta()); });
}

pub fn check_energy_supply_system(
    mut current_energy_supply_grid_version: Local<GridVersion>,
    energy_supply_grid: Res<EnergySupplyGrid>,
    mut buildings: Query<(&GridImprint, &GridCoords, &mut TechnicalState), (With<Building>, Without<SupplierEnergy>)>
) {
    if *current_energy_supply_grid_version == energy_supply_grid.version { return; }
    *current_energy_supply_grid_version = energy_supply_grid.version;
    for (grid_imprint, grid_coords, mut technical_state) in buildings.iter_mut() {
        technical_state.has_energy_supply = energy_supply_grid.is_imprint_suppliable(*grid_coords, *grid_imprint);
    }
}

pub fn damage_control_system(
    mut commands: Commands,
    mut obstacle_grid: ResMut<ObstacleGrid>,
    buildings: Query<(Entity, &Health, &GridImprint, &GridCoords), With<Building>>,
) {
    for (entity, health, grid_imprint, grid_coords) in buildings.iter() {
        if health.is_dead() {
            commands.entity(entity).despawn();
            obstacle_grid.deprint_main_floor(*grid_coords, *grid_imprint);
            grid_imprint.covered_coords(*grid_coords).into_iter().for_each(|coords| {
                commands.spawn(BuilderExplosion(coords));
            });
            commands.queue(BuildingDestroyedEvent(entity));
        }
    }
}

pub fn rotate_tower_top_system(
    mut tower_rotational_top: Query<(&MarkerTowerRotationalTop, &mut Transform)>,
    towers: Query<&TowerTopRotation, With<MarkerTower>>,
) {
    for (tower_rotational_top, mut tower_top_transform) in tower_rotational_top.iter_mut() {
        let parent_building = tower_rotational_top.0;
        let tower_top_rotation = towers.get(parent_building).unwrap();

        // Offset due to image naturally pointing downwards
        tower_top_transform.rotation = Quat::from_rotation_z(tower_top_rotation.current_angle);
    }
}

pub fn rotational_aiming_system(
    time: Res<Time>,
    mut towers: Query<(&mut TowerTopRotation, &TowerWispTarget, &Transform, &TechnicalState), Without<Wisp>>,
    wisps: Query<&Transform, With<Wisp>>,
) {
    for (mut rotation, target, tower_transform, technical_state) in towers.iter_mut() {
        if !technical_state.has_energy_supply { continue; }
        let TowerWispTarget::Wisp(target_wisp) = target else { continue; };
        let Ok(wisp_position) = wisps.get(*target_wisp).map(|target| target.translation.xy()) else { continue; };

        let direction_to_target = wisp_position - tower_transform.translation.xy();
        let target_angle = direction_to_target.y.atan2(direction_to_target.x);

        let angle_diff = angle_difference(target_angle, rotation.current_angle);

        let rotation_delta = rotation.speed * time.delta_secs();
        rotation.current_angle += angle_diff.clamp(-rotation_delta, rotation_delta);
    }
}
