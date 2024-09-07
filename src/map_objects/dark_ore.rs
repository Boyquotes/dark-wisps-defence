use crate::prelude::*;
use crate::grids::emissions::EmissionsEnergyRecalculateAll;
use crate::grids::obstacles::{Field, ObstacleGrid};
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
            ));
    }
}

pub const DARK_ORE_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: 3, height: 3 };
pub const DARK_ORE_BASE_IMAGE: &str = "map_objects/dark_ore.png";

#[derive(Component)]
pub struct DarkOre {
    amount: isize,
    grid_imprint: GridImprint,
}

#[derive(Event)]
pub struct BuilderDarkOre {
    pub entity: LazyEntity,
    pub grid_position: GridCoords,
}

impl BuilderDarkOre {
    pub fn new(grid_position: GridCoords) -> Self {
        Self { entity: LazyEntity::default(), grid_position }
    }
    pub fn spawn_system(
        mut commands: Commands,
        mut events: EventReader<BuilderDarkOre>,
        asset_server: Res<AssetServer>,
        mut emissions_energy_recalculate_all: ResMut<EmissionsEnergyRecalculateAll>,
    ) {
        for &BuilderDarkOre { mut entity, grid_position } in events.read() {
            let entity = entity.get(&mut commands);
            commands.entity(entity).insert((
                get_dark_ore_sprite_bundle(grid_position, &asset_server),
                grid_position,
                DarkOre { amount: 10000, grid_imprint: DARK_ORE_GRID_IMPRINT },
            ));
            emissions_energy_recalculate_all.0 = true;
        }
    }
}
impl Command for BuilderDarkOre {
    fn apply(self, world: &mut World) {
        world.send_event(self);
    }
}

pub fn get_dark_ore_sprite_bundle(grid_position: GridCoords, asset_server: &AssetServer) -> SpriteBundle {
    SpriteBundle {
        sprite: Sprite {
            custom_size: Some(DARK_ORE_GRID_IMPRINT.world_size()),
            ..Default::default()
        },
        texture: asset_server.load(DARK_ORE_BASE_IMAGE),
        transform: Transform::from_translation(
            grid_position.to_world_position_centered(DARK_ORE_GRID_IMPRINT).extend(Z_OBSTACLE)
        ),
        ..Default::default()
    }
}

pub fn remove_dark_ore(
    commands: &mut Commands,
    emissions_energy_recalculate_all: &mut ResMut<EmissionsEnergyRecalculateAll>,
    obstacle_grid: &mut ResMut<ObstacleGrid>,
    grid_position: GridCoords,
) {
    let entity = match &obstacle_grid[grid_position] {
        Field::DarkOre(entity) => *entity,
        _ => panic!("Cannot remove a dark_ore from a non-dark_ore field"),
    };
    commands.entity(entity).despawn();
    obstacle_grid.deprint(grid_position, DARK_ORE_GRID_IMPRINT);
    emissions_energy_recalculate_all.0 = true;
}


pub fn onclick_spawn_system(
    mut commands: Commands,
    mut emissions_energy_recalculate_all: ResMut<EmissionsEnergyRecalculateAll>,
    mut obstacle_grid: ResMut<ObstacleGrid>,
    mouse: Res<ButtonInput<MouseButton>>,
    mouse_info: Res<MouseInfo>,
    grid_object_placer: Query<&GridObjectPlacer>,
    dark_ores_query: Query<&GridCoords, With<DarkOre>>,
) {
    if !matches!(*grid_object_placer.single(), GridObjectPlacer::DarkOre) { return; }
    let mouse_coords = mouse_info.grid_coords;
    if mouse_info.is_over_ui || !mouse_coords.is_in_bounds(obstacle_grid.bounds()) { return; }
    if mouse.pressed(MouseButton::Left) {
        // Place a dark_ore
        if obstacle_grid.imprint_query_all(mouse_coords, DARK_ORE_GRID_IMPRINT, |field| field.is_empty()) {
            let mut dark_ore_builder = BuilderDarkOre::new(mouse_coords);
            let dark_ore_entity = dark_ore_builder.entity.get(&mut commands);
            obstacle_grid.imprint(mouse_coords, Field::DarkOre(dark_ore_entity), DARK_ORE_GRID_IMPRINT);
            commands.add(dark_ore_builder)
        }
    } else if mouse.pressed(MouseButton::Right) {
        // Remove a dark_ore
        match obstacle_grid[mouse_coords] {
            Field::DarkOre(entity) => {
                if let Ok(dark_ore_coords) = dark_ores_query.get(entity) {
                    remove_dark_ore(&mut commands, &mut emissions_energy_recalculate_all, &mut obstacle_grid, *dark_ore_coords);
                }
            },
            _ => {}
        }
    }
}