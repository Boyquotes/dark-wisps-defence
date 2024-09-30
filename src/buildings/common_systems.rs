use crate::effects::explosions::BuilderExplosion;
use crate::grids::emissions::{EmissionsEnergyRecalculateAll, EmitterEnergy};
use crate::prelude::*;
use crate::grids::base::GridVersion;
use crate::grids::energy_supply::{EnergySupplyGrid, SupplierChangedEvent, SupplierEnergy};
use crate::grids::obstacles::{BelowField, Field, ObstacleGrid};
use crate::grids::wisps::WispsGrid;
use crate::mouse::MouseInfo;
use crate::search::flooding::FloodEnergySupplyMode;
use crate::search::targetfinding::target_find_closest_wisp;
use crate::ui::grid_object_placer::GridObjectPlacer;
use crate::utils::math::angle_difference;
use crate::wisps::components::Wisp;
use super::{
    energy_relay::BuilderEnergyRelay,
    exploration_center::BuilderExplorationCenter,
    main_base::{EventMoveMainBase, MarkerMainBase},
    mining_complex::BuilderMiningComplex,
    tower_blaster::BuilderTowerBlaster,
    tower_emitter::BuilderTowerEmitter,
    tower_cannon::BuilderTowerCannon,
    tower_rocket_launcher::BuilderTowerRocketLauncher,
};

pub fn onclick_building_spawn_system(
    mut commands: Commands,
    mut obstacle_grid: ResMut<ObstacleGrid>,
    mouse: Res<ButtonInput<MouseButton>>,
    mouse_info: Res<MouseInfo>,
    almanach: Res<Almanach>,
    mut stock: ResMut<Stock>,
    grid_object_placer: Query<(&GridObjectPlacer, &GridImprint)>,
    mut main_base: Query<(Entity, &GridCoords), With<MarkerMainBase>>,
) {
    let mouse_coords = mouse_info.grid_coords;
    if mouse_info.is_over_ui || !mouse.just_released(MouseButton::Left) { return; }
    let (grid_object_placer, grid_imprint) = grid_object_placer.single();
    let GridObjectPlacer::Building(building_type) = grid_object_placer else { return; };
    // Grid Placement Validation
    if !mouse_coords.is_imprint_in_bounds(grid_imprint, obstacle_grid.bounds())
        || !obstacle_grid.query_building_placement(mouse_coords, *building_type, *grid_imprint) { return; }
    // Payment
    let building_costs = almanach.get_building_cost(*building_type);
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
            let entity = commands.spawn_empty().id();
            commands.add(BuilderEnergyRelay::new(entity, mouse_coords));
            GridAction::Imprint(entity)
        }
        BuildingType::ExplorationCenter => {
            let entity = commands.spawn_empty().id();
            commands.add(BuilderExplorationCenter::new(entity, mouse_coords));
            GridAction::Imprint(entity)
        }
        BuildingType::Tower(TowerType::Blaster) => {
            let entity = commands.spawn_empty().id();
            commands.add(BuilderTowerBlaster::new(entity, mouse_coords));
            GridAction::Imprint(entity)
        },
        BuildingType::Tower(TowerType::Cannon) => {
            let entity = commands.spawn_empty().id();
            commands.add(BuilderTowerCannon::new(entity, mouse_coords));
            GridAction::Imprint(entity)
        },
        BuildingType::Tower(TowerType::RocketLauncher) => {
            let entity = commands.spawn_empty().id();
            commands.add(BuilderTowerRocketLauncher::new(entity, mouse_coords));
            GridAction::Imprint(entity)
        },
        BuildingType::Tower(TowerType::Emitter) => {
            let entity = commands.spawn_empty().id();
            commands.add(BuilderTowerEmitter::new(entity, mouse_coords));
            GridAction::Imprint(entity)
        },
        BuildingType::MainBase => {
            let (main_base_entity, main_base_coords) = main_base.single_mut();
            commands.add(EventMoveMainBase { new_grid_position: mouse_coords });
            GridAction::Reprint{entity: main_base_entity, old_coords: *main_base_coords}
        },
        BuildingType::MiningComplex => {
            let entity = commands.spawn_empty().id();
            commands.add(BuilderMiningComplex::new(entity, mouse_coords));
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
    mut towers: Query<(&GridCoords, &GridImprint, &TechnicalState, &TowerRange, &mut TowerWispTarget), (With<MarkerTower>, Without<Wisp>)>,
    wisps: Query<&GridCoords, (With<Wisp>, Without<MarkerTower>)>,
) {
    for (coords, grid_imprint, technical_state, range, mut target) in towers.iter_mut() {
        if !technical_state.has_energy_supply { continue; }
        match *target {
            TowerWispTarget::Wisp(wisp_entity) => {
                if let Ok(wisp_coords) = wisps.get(*wisp_entity) {
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
    mut emissions_energy_recalculate_all: ResMut<EmissionsEnergyRecalculateAll>,
    mut obstacle_grid: ResMut<ObstacleGrid>,
    mut supplier_created_event_writer: EventWriter<SupplierChangedEvent>,
    buildings: Query<(Entity, &Health, &GridImprint, &GridCoords, Has<EmitterEnergy>, Option<&SupplierEnergy>), With<Building>>,
) {
    for (entity, health, grid_imprint, grid_coords, has_emitter_energy, maybe_supplier_energy) in buildings.iter() {
        if health.is_dead() {
            commands.entity(entity).despawn_recursive();
            obstacle_grid.deprint_main_floor(*grid_coords, *grid_imprint);
            if has_emitter_energy {
                emissions_energy_recalculate_all.0 = true;
            }
            if let Some(suplier_energy) = maybe_supplier_energy {
                supplier_created_event_writer.send(SupplierChangedEvent {
                    supplier: entity,
                    coords: grid_imprint.covered_coords(*grid_coords),
                    range: suplier_energy.range,
                    mode: FloodEnergySupplyMode::Decrease,
                });
            }
            grid_imprint.covered_coords(*grid_coords).into_iter().for_each(|coords| {
                commands.add(BuilderExplosion::new(coords));
            });
            commands.add(BuildingDestroyedEvent(entity));
        }
    }
}

pub fn rotate_tower_top_system(
    mut tower_rotational_top: Query<(&MarkerTowerRotationalTop, &mut Transform)>,
    towers: Query<&TowerTopRotation, With<MarkerTower>>,
) {
    for (tower_rotational_top, mut tower_top_transform) in tower_rotational_top.iter_mut() {
        let parent_building = tower_rotational_top.0;
        let tower_top_rotation = towers.get(*parent_building).unwrap();

        // Offset due to image naturally pointing downwards
        tower_top_transform.rotation = Quat::from_rotation_z(tower_top_rotation.current_angle);
    }
}

pub fn rotational_aiming_system(
    time: Res<Time>,
    mut towers: Query<(&mut TowerTopRotation, &TowerWispTarget, &Transform), Without<Wisp>>,
    wisps: Query<&Transform, With<Wisp>>,
) {
    for (mut rotation, target, tower_transform) in towers.iter_mut() {
        let TowerWispTarget::Wisp(target_wisp) = target else { continue; };
        let Ok(wisp_position) = wisps.get(**target_wisp).map(|target| target.translation.xy()) else { continue; };

        let direction_to_target = wisp_position - tower_transform.translation.xy();
        let target_angle = direction_to_target.y.atan2(direction_to_target.x);

        let angle_diff = angle_difference(target_angle, rotation.current_angle);

        let rotation_delta = rotation.speed * time.delta_seconds();
        rotation.current_angle += angle_diff.clamp(-rotation_delta, rotation_delta);
    }
}