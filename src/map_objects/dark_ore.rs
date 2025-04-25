use std::f32::consts::PI;

use nanorand::Rng;

use crate::prelude::*;
use crate::grids::obstacles::{BelowField, Field, ObstacleGrid};
use crate::mouse::MouseInfo;
use crate::ui::grid_object_placer::GridObjectPlacer;

pub struct DarkOrePlugin;
impl Plugin for DarkOrePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<BuilderDarkOre>()
            .add_systems(PostUpdate, (
                BuilderDarkOre::spawn_system,
            ))
            .add_systems(Update, (
                onclick_spawn_system,
                remove_empty_dark_ore_system,
            ));
    }
}

pub const DARK_ORE_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: 1, height: 1 };
pub const DARK_ORE_BASE_IMAGES: [&str; 2] = ["map_objects/dark_ore_1.png", "map_objects/dark_ore_2.png"];

#[derive(Component)]
pub struct DarkOre {
    pub amount: i32,
}

#[derive(Event)]
pub struct BuilderDarkOre {
    pub entity: Entity,
    pub grid_position: GridCoords,
}

impl BuilderDarkOre {
    pub fn new(entity: Entity, grid_position: GridCoords) -> Self {
        Self { entity, grid_position }
    }
    pub fn spawn_system(
        mut commands: Commands,
        mut events: EventReader<BuilderDarkOre>,
        asset_server: Res<AssetServer>,
    ) {
        let mut rng = nanorand::tls_rng();
        for &BuilderDarkOre { entity, grid_position } in events.read() {
            commands.entity(entity).insert((
                Sprite {
                    image: asset_server.load(DARK_ORE_BASE_IMAGES[rng.generate_range(0usize..2usize)]),
                    custom_size: Some(DARK_ORE_GRID_IMPRINT.world_size()),
                    ..Default::default()
                },
                Transform {
                    translation: grid_position.to_world_position_centered(DARK_ORE_GRID_IMPRINT).extend(Z_OBSTACLE),
                    // select one of: Left, Up, Right, Down
                    rotation: Quat::from_rotation_z([0., PI / 2., PI, 3. * PI / 2.][rng.generate_range(0usize..4usize)] as f32),
                    ..default()
                },
                grid_position,
                DarkOre { amount: 1000 },
                DARK_ORE_GRID_IMPRINT,
            ));
        }
    }
}
impl Command for BuilderDarkOre {
    fn apply(self, world: &mut World) {
        world.send_event(self);
    }
}

pub fn remove_dark_ore(
    commands: &mut Commands,
    obstacle_grid: &mut ResMut<ObstacleGrid>,
    grid_position: GridCoords,
) {
    let (dark_ore_entity, new_field) = match &obstacle_grid[grid_position] {
        Field::DarkOre(entity) => (*entity, Field::Empty),
        Field::Building(building_entity, BuildingType::MiningComplex, BelowField::DarkOre(entity)) => (*entity, Field::Building(*building_entity, BuildingType::MiningComplex, BelowField::Empty)),
        _ => panic!("Cannot remove a dark_ore from a non-dark_ore field"),
    };
    commands.entity(dark_ore_entity).despawn();
    obstacle_grid.imprint_custom(grid_position, DARK_ORE_GRID_IMPRINT, &|field| *field = new_field.clone());
}


fn onclick_spawn_system(
    mut commands: Commands,
    mut obstacle_grid: ResMut<ObstacleGrid>,
    mouse: Res<ButtonInput<MouseButton>>,
    mouse_info: Res<MouseInfo>,
    grid_object_placer: Query<&GridObjectPlacer>,
    dark_ores_query: Query<&GridCoords, With<DarkOre>>,
) {
    if !matches!(*grid_object_placer.single().unwrap(), GridObjectPlacer::DarkOre) { return; }
    let mouse_coords = mouse_info.grid_coords;
    if mouse_info.is_over_ui || !mouse_coords.is_in_bounds(obstacle_grid.bounds()) { return; }
    if mouse.pressed(MouseButton::Left) {
        // Place a dark_ore
        if obstacle_grid.imprint_query_all(mouse_coords, DARK_ORE_GRID_IMPRINT, |field| field.is_empty()) {
            let dark_ore_entity = commands.spawn_empty().id();
            commands.queue(BuilderDarkOre::new(dark_ore_entity, mouse_coords));
            obstacle_grid.imprint(mouse_coords, Field::DarkOre(dark_ore_entity), DARK_ORE_GRID_IMPRINT);
        }
    } else if mouse.pressed(MouseButton::Right) {
        // Remove a dark_ore
        match obstacle_grid[mouse_coords] {
            Field::DarkOre(entity) => {
                if let Ok(dark_ore_coords) = dark_ores_query.get(entity) {
                    remove_dark_ore(&mut commands, &mut obstacle_grid, *dark_ore_coords);
                }
            },
            _ => {}
        }
    }
}

fn remove_empty_dark_ore_system(
    mut commands: Commands,
    mut obstacle_grid: ResMut<ObstacleGrid>,
    dark_ores_query: Query<(&GridCoords, &DarkOre), Changed<DarkOre>>,
) {
    for (grid_coords, dark_ore) in dark_ores_query.iter() {
        if dark_ore.amount <= 0 {
            remove_dark_ore(&mut commands, &mut obstacle_grid, *grid_coords);
        }
    }
}