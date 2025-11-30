use bevy::color::palettes::css::GRAY;

use lib_grid::grids::obstacles::{GridStructureType, ObstacleGrid, ReservedCoords};

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
            .register_db_loader::<BuilderWall>(MapLoadingStage2::SpawnMapElements)
            .register_db_saver(BuilderWall::on_game_save)
            .add_observer(BuilderWall::on_add);
    }
}

pub const WALL_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: 1, height: 1 };

#[derive(Component)]
#[require(MapBound, ObstacleGridObject = ObstacleGridObject::Wall, EmissionsGridSpreadAffector)]
pub struct Wall;

#[derive(Component, SSS)]
pub struct BuilderWall {
    pub grid_position: GridCoords,
    pub entity: Option<Entity>,
}
impl Saveable for BuilderWall {
    fn save(self, tx: &rusqlite::Transaction) -> rusqlite::Result<()> {
        let entity_index = self.entity.expect("BuilderWall for saving purpose must have an entity").index() as i64;

        // 1. Insert into walls table
        tx.save_marker("walls", entity_index)?;

        // 2. Insert into grid_positions table
        tx.save_grid_coords(entity_index, self.grid_position)?;
        Ok(())
    }
}
impl Loadable for BuilderWall {
    fn load(ctx: &mut LoadContext) -> rusqlite::Result<LoadResult> {
        const LIMIT: usize = 100;
        let offset = ctx.offset;

        let mut stmt = ctx.conn.prepare_cached(
            "SELECT w.id, gp.x, gp.y FROM walls w 
             JOIN grid_coords gp ON w.id = gp.entity_id 
             LIMIT ?1 OFFSET ?2"
        )?;

        let rows = stmt.query_map([LIMIT, offset], |row| {
            let entity_index: u64 = row.get(0)?;
            let x: i32 = row.get(1)?;
            let y: i32 = row.get(2)?;
            Ok((entity_index, GridCoords { x, y }))
        })?;

        let mut batch = Vec::new();
        for row in rows {
            let (old_id, grid_position) = row?;
            if let Some(new_entity) = ctx.get_new_entity_for_old(old_id) {
                batch.push((new_entity, BuilderWall::new_for_saving(grid_position, new_entity)));
            } else {
                eprintln!("Warning: Wall with old ID {} has no corresponding new entity", old_id);
            }
        }
        let batch_size = batch.len();
        ctx.commands.insert_batch(batch);
        
        Ok(batch_size.into())
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

pub fn onclick_spawn_system(
    mut commands: Commands,
    mut reserved_coords: ResMut<ReservedCoords>,
    obstacle_grid: Res<ObstacleGrid>,
    mouse: Res<ButtonInput<MouseButton>>,
    mouse_info: Res<MouseInfo>,
    grid_object_placer: Single<&GridObjectPlacer>,
) {
    if !matches!(*grid_object_placer.into_inner(), GridObjectPlacer::Wall) { return; }
    let mouse_coords = mouse_info.grid_coords;
    if mouse_info.is_over_ui || !mouse_coords.is_in_bounds(obstacle_grid.bounds()) { return; }
    if mouse.pressed(MouseButton::Left) {
        // Place a wall
        if obstacle_grid[mouse_coords].is_empty() && !reserved_coords.any_reserved(mouse_coords, WALL_GRID_IMPRINT) {
            commands.spawn(BuilderWall::new(mouse_coords));
            reserved_coords.reserve(mouse_coords, WALL_GRID_IMPRINT);
        }
    } else if mouse.pressed(MouseButton::Right) {
        // Remove a wall
        if let GridStructureType::Wall(entity) = obstacle_grid[mouse_coords].structure {
            commands.entity(entity).despawn();
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