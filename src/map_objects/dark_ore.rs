use crate::prelude::*;
use crate::grids::emissions::EmissionsEnergyRecalculateAll;
use crate::grids::obstacles::{Field, ObstacleGrid};
use crate::mouse::MouseInfo;
use crate::ui::grid_object_placer::GridObjectPlacer;

pub const DARK_ORE_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: 3, height: 3 };
pub const DARK_ORE_BASE_IMAGE: &str = "map_objects/dark_ore.png";

#[derive(Component)]
pub struct DarkOre {
    amount: isize,
    grid_imprint: GridImprint,
}

#[derive(Bundle)]
pub struct BuilderDarkOre {
    sprite: SpriteBundle,
    grid_position: GridCoords,
    dark_ore: DarkOre,
}

impl BuilderDarkOre {
    pub fn new(grid_position: GridCoords, asset_server: &AssetServer) -> Self {
        Self {
            sprite: get_dark_ore_sprite_bundle(grid_position, asset_server),
            grid_position,
            dark_ore: DarkOre { amount: 10000, grid_imprint: DARK_ORE_GRID_IMPRINT },
        }
    }
    pub fn spawn(
        self,
        commands: &mut Commands,
        emissions_energy_recalculate_all: &mut EmissionsEnergyRecalculateAll,
        obstacles_grid: &mut ObstacleGrid
    ) -> Entity {
        let position = self.grid_position;
        let entity = commands.spawn(self).id();
        obstacles_grid.imprint(position, Field::DarkOre(entity), DARK_ORE_GRID_IMPRINT);
        emissions_energy_recalculate_all.0 = true;
        entity
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
    asset_server: Res<AssetServer>,
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
            BuilderDarkOre::new(mouse_coords, &asset_server)
                .spawn(&mut commands, &mut emissions_energy_recalculate_all, &mut obstacle_grid);
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