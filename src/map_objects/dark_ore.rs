use std::f32::consts::PI;

use lib_grid::grids::obstacles::{BelowField, Field, ObstacleGrid};

use crate::prelude::*;
use crate::ui::grid_object_placer::GridObjectPlacer;

pub struct DarkOrePlugin;
impl Plugin for DarkOrePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                onclick_spawn_system.run_if(in_state(UiInteraction::PlaceGridObject)),
                remove_empty_dark_ore_system,
            ))
            .add_observer(BuilderDarkOre::on_add)
            .add_observer(dark_ore_area_scanner::DarkOreAreaScanner::on_add)
            .add_observer(dark_ore_area_scanner::DarkOreAreaScanner::on_remove_dark_ore);
    }
}

pub const DARK_ORE_GRID_IMPRINT: GridImprint = GridImprint::Rectangle { width: 1, height: 1 };
pub const DARK_ORE_BASE_IMAGES: [&str; 2] = ["map_objects/dark_ore_1.png", "map_objects/dark_ore_2.png"];

#[derive(Component)]
#[require(MapBound)]
pub struct DarkOre {
    pub amount: i32,
}

#[derive(Component)]
pub struct BuilderDarkOre {
    pub grid_position: GridCoords,
}
impl BuilderDarkOre {
    pub fn new(grid_position: GridCoords) -> Self {
        Self { grid_position }
    }
    
    fn on_add(
        trigger: Trigger<OnAdd, BuilderDarkOre>,
        mut commands: Commands,
        builders: Query<&BuilderDarkOre>,
        asset_server: Res<AssetServer>,
    ) {
        let entity = trigger.target();
        let Ok(builder) = builders.get(entity) else { return; };
        
        let mut rng = nanorand::tls_rng();
        commands.entity(entity)
            .remove::<BuilderDarkOre>()
            .insert((
                Sprite {
                    image: asset_server.load(DARK_ORE_BASE_IMAGES[rng.generate_range(0usize..2usize)]),
                    custom_size: Some(DARK_ORE_GRID_IMPRINT.world_size()),
                    ..Default::default()
                },
                Transform {
                    translation: builder.grid_position.to_world_position_centered(DARK_ORE_GRID_IMPRINT).extend(Z_OBSTACLE),
                    // select one of: Left, Up, Right, Down
                    rotation: Quat::from_rotation_z([0., PI / 2., PI, 3. * PI / 2.][rng.generate_range(0usize..4usize)] as f32),
                    ..default()
                },
                builder.grid_position,
                DarkOre { amount: 1000 },
                DARK_ORE_GRID_IMPRINT,
            ));
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
            let dark_ore_entity = commands.spawn(BuilderDarkOre::new(mouse_coords)).id();
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

pub mod dark_ore_area_scanner {
    use super::*;

    #[derive(Component, Default)]
    pub struct HasOreInScannerRange;
    #[derive(Component, Default)]
    pub struct NoOreInScannerRange;

    #[derive(Component, Clone)]
    #[component(immutable)]
    #[require(GridCoords, DarkOreInRange, NoOreInScannerRange)]
    pub struct DarkOreAreaScanner {
        pub range_imprint: GridImprint,
    }
    impl DarkOreAreaScanner {
        pub fn on_add(
            trigger: Trigger<OnAdd, DarkOreAreaScanner>,
            mut commands: Commands,
            scanners: Query<&DarkOreAreaScanner>
        ) {
            let entity = trigger.target();
            let scanner = scanners.get(entity).unwrap();
            commands.entity(entity)
                .observe(Self::scan_on_change)
                .insert(scanner.clone()); // Reinsert self to trigger initial scan; TODO: improve once Bevy introduces compound triggers
        }

        // Local triggers when entity that is interested in scanner info changes by moving or changing the scanner range
        fn scan_on_change(
            trigger: Trigger<OnInsert, (DarkOreAreaScanner, GridCoords)>,
            mut commands: Commands,
            obstacle_grid: Res<ObstacleGrid>,
            mut scanners: Query<(&DarkOreAreaScanner, &GridCoords, &mut DarkOreInRange)>,
        ) {
            let entity = trigger.target();
            let Ok((scanner, grid_coords, mut dark_ore_in_range)) = scanners.get_mut(entity) else { return; };
            let ore_entities_in_range = obstacle_grid.imprint_query_element(*grid_coords, scanner.range_imprint, Self::is_dark_ore_helper);
            if ore_entities_in_range.is_empty() {
                commands.entity(entity).insert(NoOreInScannerRange).remove::<HasOreInScannerRange>();
            } else {
                commands.entity(entity).insert(HasOreInScannerRange).remove::<NoOreInScannerRange>();
            }
            dark_ore_in_range.0 = ore_entities_in_range;
        }

        // Global trigger reacting to any dark ore removal to keep DarkOreinRange in sync
        pub fn on_remove_dark_ore(
            trigger: Trigger<OnRemove, DarkOre>,
            mut commands: Commands,
            dark_ores: Query<&GridCoords, With<DarkOre>>,
            mut scanners: Query<(Entity, &DarkOreAreaScanner, &mut DarkOreInRange, &GridCoords)>,
        ) {
            let entity = trigger.target();
            let dark_ore_grid_coords = dark_ores.get(entity).unwrap();
            for (scanner_entity, scanner, mut dark_ore_in_range, scanner_grid_coords) in scanners.iter_mut() {
                // TODO: This won't work when we want to implement Mining Complex range expansion, as the GridCoords won't match ScannerImprint coords
                // Ie, the expected mining range coords will shift in relation to the MiningComplex own's coords as they start in bottom left corner.
                if scanner.range_imprint.covers_coords(*scanner_grid_coords, *dark_ore_grid_coords) {
                    if let Some(index) = dark_ore_in_range.0.iter().position(|&x| x == entity) {
                        dark_ore_in_range.0.swap_remove(index);
                    }
                }
                if dark_ore_in_range.0.is_empty() {
                    commands.entity(scanner_entity).insert(NoOreInScannerRange).remove::<HasOreInScannerRange>();
                }
            }
        }

        // Helper to execute on every obstacle grid field to gather the dark_ore entities
        fn is_dark_ore_helper(field: &Field) -> Option<Entity> {
            if let Field::Building(_, BuildingType::MiningComplex, BelowField::DarkOre(dark_ore_entity)) = field { Some(*dark_ore_entity) } else { None }
        }
    }

    #[derive(Component, Default)]
    pub struct DarkOreInRange(pub Vec<Entity>);
}
