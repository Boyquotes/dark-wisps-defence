use bevy::color::palettes::css::GRAY;

use lib_grid::grids::emissions::EmissionsEnergyRecalculateAll;
use lib_grid::grids::obstacles::{Field, ObstacleGrid};

use crate::prelude::*;
use crate::ui::grid_object_placer::GridObjectPlacer;

pub struct WallPlugin;
impl Plugin for WallPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                onclick_spawn_system.run_if(in_state(UiInteraction::PlaceGridObject)),
                color_rotation_system,
            ))
            .add_systems(PostUpdate, (
                BuilderWall::on_game_save.run_if(on_message::<SaveGameSignal>),
            ))
            .add_systems(Update, lib_core::common::load_batch_system::<BuilderWall>)
            .add_observer(BuilderWall::on_add);
    }
}

pub const WALL_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: 1, height: 1 };

#[derive(Component)]
#[require(MapBound)]
pub struct Wall;

#[derive(Component)]
pub struct BuilderWall {
    pub grid_position: GridCoords,
    pub entity: Option<Entity>,
}
impl SSS for BuilderWall {}
impl Saveable for BuilderWall {
    fn save(self, tx: &rusqlite::Transaction) -> rusqlite::Result<()> {
        let entity_index = self.entity.expect("BuilderWall for saving purpose must have an entity").index() as i64;
        
        // 0. Ensure entity exists in entities table
        tx.execute(
            "INSERT OR IGNORE INTO entities (id) VALUES (?1)",
            [entity_index],
        )?;

        // 1. Insert into walls table
        tx.execute(
            "INSERT INTO walls (id) VALUES (?1)",
            [entity_index],
        )?;

        // 2. Insert into grid_positions table
        tx.execute(
            "INSERT INTO grid_positions (wall_id, x, y) VALUES (?1, ?2, ?3)",
            (entity_index, self.grid_position.x, self.grid_position.y),
        )?;
        Ok(())
    }
}
impl Loadable for BuilderWall {
    fn load_batch(
        conn: &rusqlite::Connection, 
        limit: usize, 
        offset: usize,
        commands: &mut Commands,
        entity_map: &lib_core::common::EntityMap,
    ) -> rusqlite::Result<usize> {
        let mut stmt = conn.prepare_cached(
            "SELECT w.id, gp.x, gp.y FROM walls w 
             JOIN grid_positions gp ON w.id = gp.wall_id 
             LIMIT ?1 OFFSET ?2"
        )?;

        let rows = stmt.query_map([limit, offset], |row| {
            let entity_index: i64 = row.get(0)?;
            let x: i32 = row.get(1)?;
            let y: i32 = row.get(2)?;
            Ok((entity_index, GridCoords { x, y }))
        })?;

        let mut count = 0;
        for row in rows {
            let (old_id, grid_position) = row?;
            if let Some(&new_entity) = entity_map.map.get(&(old_id as u64)) {
                commands.entity(new_entity).insert(BuilderWall::new_for_saving(grid_position, new_entity));
                count += 1;
            } else {
                eprintln!("Warning: Wall with old ID {} has no corresponding new entity", old_id);
            }
        }
        Ok(count)
    }
}
impl BuilderWall {
    pub fn new(grid_position: GridCoords) -> Self { 
        Self { grid_position, entity: None }
    }
    pub fn new_for_saving(grid_position: GridCoords, entity: Entity) -> Self { 
        Self { grid_position, entity: Some(entity) }
    }
    
    fn on_add(
        trigger: On<Add, BuilderWall>,
        mut commands: Commands,
        mut emissions_energy_recalculate_all: ResMut<EmissionsEnergyRecalculateAll>,
        builders: Query<&BuilderWall>,
    ) {
        let entity = trigger.entity;
        let Ok(builder) = builders.get(entity) else { return; };
        
        commands.entity(entity)
            .remove::<BuilderWall>()
            .insert((
                Sprite {
                    color: GRAY.into(), // Color::hsla(0., 0.5, 1.3, 0.8); for hdr
                    custom_size: Some(WALL_GRID_IMPRINT.world_size()),
                    ..default()
                },
                Transform::from_translation(builder.grid_position.to_world_position_centered(WALL_GRID_IMPRINT).extend(Z_OBSTACLE)),
                builder.grid_position,
                WALL_GRID_IMPRINT,
                Wall,
            ));
        emissions_energy_recalculate_all.0 = true;
    }

    fn on_game_save(
        mut commands: Commands,
        walls: Query<(Entity, &GridCoords), With<Wall>>,
    ) {
        println!("Creating batch of BuilderWalls for saving. {} walls", walls.iter().count());
        let batch = walls
            .iter()
            .map(|(entity, grid_coords)| BuilderWall::new_for_saving(*grid_coords, entity))
            .collect::<SaveableBatchCommand<_>>();
        commands.queue(batch);
    }
}

pub fn remove_wall(
    commands: &mut Commands,
    emissions_energy_recalculate_all: &mut ResMut<EmissionsEnergyRecalculateAll>,
    obstacle_grid: &mut ResMut<ObstacleGrid>,
    grid_position: GridCoords,
) {
    let entity = match &obstacle_grid[grid_position] {
        Field::Wall(entity) => *entity,
        _ => panic!("Cannot remove a wall on a non-wall"),
    };
    commands.entity(entity).despawn();
    obstacle_grid.deprint_all(grid_position, WALL_GRID_IMPRINT);
    emissions_energy_recalculate_all.0 = true;
}


pub fn onclick_spawn_system(
    mut commands: Commands,
    mut emissions_energy_recalculate_all: ResMut<EmissionsEnergyRecalculateAll>,
    mut obstacle_grid: ResMut<ObstacleGrid>,
    mouse: Res<ButtonInput<MouseButton>>,
    mouse_info: Res<MouseInfo>,
    grid_object_placer: Single<&GridObjectPlacer>,
) {
    if !matches!(*grid_object_placer.into_inner(), GridObjectPlacer::Wall) { return; }
    let mouse_coords = mouse_info.grid_coords;
    if mouse_info.is_over_ui || !mouse_coords.is_in_bounds(obstacle_grid.bounds()) { return; }
    if mouse.pressed(MouseButton::Left) {
        // Place a wall
        if obstacle_grid[mouse_coords].is_empty() {
            let wall_entity = commands.spawn(BuilderWall::new(mouse_coords)).id();
            obstacle_grid.imprint(mouse_coords, Field::Wall(wall_entity), WALL_GRID_IMPRINT);
        }
    } else if mouse.pressed(MouseButton::Right) {
        // Remove a wall
        if obstacle_grid[mouse_coords].is_wall() {
            remove_wall(&mut commands, &mut emissions_energy_recalculate_all, &mut obstacle_grid, mouse_coords);
        }
    }
}

// TODO: decide whether to keep it
pub fn color_rotation_system(
    mut query: Query<&mut Sprite, With<Wall>>,
    time: Res<Time>,
) {
    for mut sprite in query.iter_mut() {
        if let Color::Hsla(Hsla{hue, ..}) = &mut sprite.color {
            *hue += time.delta_secs() * 100.;
            if *hue > 360. {
                *hue = 0.;
            }
        }
    }
}