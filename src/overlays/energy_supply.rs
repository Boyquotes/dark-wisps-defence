use std::collections::VecDeque;

use bevy::reflect::TypePath;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat};
use bevy::sprite::{Material2d, MaterialMesh2dBundle};
use crate::prelude::*;
use crate::search::common::{CARDINAL_DIRECTIONS, VISITED_GRID};
use crate::ui::display_building_info::BuildingUiFocusChangedEvent;
use crate::grids::base::GridVersion;
use crate::grids::energy_supply::EnergySupplyGrid;
use crate::ui::grid_object_placer::GridObjectPlacer;

pub struct EnergySupplyOverlayPlugin;
impl Plugin for EnergySupplyOverlayPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<EnergySupplyOverlayState>()
            .init_resource::<EnergySupplyOverlayConfig>()
            .add_systems(Startup, (
                create_energy_supply_overlay_startup_system,
            ))
            .add_systems(OnEnter(EnergySupplyOverlayState::Show), on_overlay_show_system)
            .add_systems(OnExit(EnergySupplyOverlayState::Show), on_overlay_hide_system)
            .add_systems(OnExit(UiInteraction::PlaceGridObject), on_grid_placer_exited_system)
            .add_systems(Update, (
                on_config_change_system.run_if(resource_changed::<EnergySupplyOverlayConfig>),
                refresh_display_system.run_if(in_state(EnergySupplyOverlayState::Show)),
                manage_energy_supply_overlay_global_mode_system,
                on_building_ui_focus_changed_system.run_if(on_event::<BuildingUiFocusChangedEvent>()),
                on_grid_placer_changed_system.run_if(in_state(UiInteraction::PlaceGridObject)),
            ));
    }
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum EnergySupplyOverlayState {
    #[default]
    Hide,
    Show,
}
#[derive(Resource, Default)]
pub struct EnergySupplyOverlayConfig {
    // Determines whether the energy supply overlay is globally enabled. We need that information for example when we are in building placing mode,
    // showing the highlihted building, and then we need to disable the highligh. We have to either hide the overlay or change it to `All` depending on the state
    // before the placing had started.
    pub is_overlay_globally_enabled: bool,
    pub grid_version: GridVersion, // Grid version for which we show the overlay
    pub secondary_mode: EnergySupplyOverlaySecondaryMode,
}
#[derive(Default, Clone, Debug, PartialEq)]
pub enum EnergySupplyOverlaySecondaryMode {
    #[default]
    None,
    Highlight(BuildingId),
    Placing{grid_coords: GridCoords, grid_imprint: GridImprint, range: usize},
}
impl EnergySupplyOverlaySecondaryMode {
    pub fn is_none(&self) -> bool { matches!(self, EnergySupplyOverlaySecondaryMode::None) }
    pub fn is_highlighting(&self) -> bool { matches!(self, EnergySupplyOverlaySecondaryMode::Highlight(_)) }
    pub fn is_placing(&self) -> bool { matches!(self, EnergySupplyOverlaySecondaryMode::Placing{..}) }
}

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct EnergySupplyHeatmapMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub heatmap: Handle<Image>,
    #[uniform(2)]
    pub highlight_enabled: u32, // 1 if the display is in highlight mode and should make the specific building range highlighted
}
impl EnergySupplyHeatmapMaterial {
    fn configure(&mut self, secondary_mode: &EnergySupplyOverlaySecondaryMode) {
        match secondary_mode {
            EnergySupplyOverlaySecondaryMode::Highlight(_) => {
                self.highlight_enabled = 1;
            }
            _ => self.highlight_enabled = 0,
        }
    }
}
impl Material2d for EnergySupplyHeatmapMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/energy_supply_map.wgsl".into()
    }
}

#[derive(Component)]
pub struct EnergySupplyOverlay;

fn on_config_change_system(
    overlay_config: Res<EnergySupplyOverlayConfig>,
    mut overlay_state: ResMut<NextState<EnergySupplyOverlayState>>,
) {
    if overlay_config.is_overlay_globally_enabled || !overlay_config.secondary_mode.is_none() {
        overlay_state.set(EnergySupplyOverlayState::Show);
    } else {
        overlay_state.set(EnergySupplyOverlayState::Hide);
    }
}
fn on_overlay_show_system(
    mut emission_material_visibility: Query<&mut Visibility, With<EnergySupplyOverlay>>,
) {
    *emission_material_visibility.single_mut() = Visibility::Inherited;
}
fn on_overlay_hide_system(
    mut emission_material_visibility: Query<&mut Visibility, With<EnergySupplyOverlay>>,
) {
    *emission_material_visibility.single_mut() = Visibility::Hidden;
}


pub fn manage_energy_supply_overlay_global_mode_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut overlay_config: ResMut<EnergySupplyOverlayConfig>
) {
    if keys.just_pressed(KeyCode::KeyY) {
        overlay_config.is_overlay_globally_enabled = !overlay_config.is_overlay_globally_enabled;
    }
}

fn refresh_display_system(
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<EnergySupplyHeatmapMaterial>>,
    energy_supply_grid: Res<EnergySupplyGrid>,
    mut overlay_config: ResMut<EnergySupplyOverlayConfig>,
    energy_supply_overlay: Query<&Handle<EnergySupplyHeatmapMaterial>, With<EnergySupplyOverlay>>,
    mut last_secondary_mode: Local<EnergySupplyOverlaySecondaryMode>,
) {
    if overlay_config.grid_version != energy_supply_grid.version || overlay_config.secondary_mode != *last_secondary_mode {
        *last_secondary_mode = overlay_config.secondary_mode.clone();
        overlay_config.grid_version = energy_supply_grid.version;
        let heatmap_material_handle = energy_supply_overlay.single();
        let heatmap_material = materials.get_mut(heatmap_material_handle).unwrap();
        heatmap_material.configure(&overlay_config.secondary_mode);
        let heatmap_image = images.get_mut(&heatmap_material.heatmap).unwrap();
        let mut overlay_creator = OverlayHeatmapCreator { energy_supply_grid: &energy_supply_grid, heatmap_data: &mut heatmap_image.data };
        match &overlay_config.secondary_mode {
            EnergySupplyOverlaySecondaryMode::None => {
                overlay_creator.imprint_current_state(None); 
            }
            EnergySupplyOverlaySecondaryMode::Highlight(building_id) => {
                overlay_creator.imprint_current_state(Some(*building_id)); 
            }
            EnergySupplyOverlaySecondaryMode::Placing{grid_coords, grid_imprint, range} => {
                overlay_creator.imprint_current_state(None); 
                if grid_coords.is_in_bounds(energy_supply_grid.bounds()) {
                    overlay_creator.flood_potential_energy_supply_to_overlay_heatmap(
                        grid_imprint.covered_coords(*grid_coords).iter().filter(|coords| coords.is_in_bounds(energy_supply_grid.bounds())), 
                        *range
                    ); 
                }
            }
        }
    }
}

fn on_building_ui_focus_changed_system(
    mut events: EventReader<BuildingUiFocusChangedEvent>,
    mut overlay_config: ResMut<EnergySupplyOverlayConfig>,
) {
    for event in events.read() {
        match event {
            BuildingUiFocusChangedEvent::Focus(building_id) => {
                overlay_config.secondary_mode = EnergySupplyOverlaySecondaryMode::Highlight(*building_id);
            }
            BuildingUiFocusChangedEvent::Unfocus => {
                overlay_config.secondary_mode = EnergySupplyOverlaySecondaryMode::None;
            }
        }
    }
}

fn on_grid_placer_changed_system(
    almanach: Res<Almanach>,
    mut overlay_config: ResMut<EnergySupplyOverlayConfig>,
    grid_object_placer: Query<(&GridObjectPlacer, &GridCoords)>,
    mut last_grid_object_placer: Local<(GridObjectPlacer, GridCoords)>,
) {
    let Ok((grid_object_placer, grid_coords)) = grid_object_placer.get_single() else { return; };
    if grid_object_placer != &last_grid_object_placer.0 || grid_coords != &last_grid_object_placer.1 {
        *last_grid_object_placer = (grid_object_placer.clone(), *grid_coords);
        let (grid_object_placer, grid_coords) = (grid_object_placer, grid_coords);
        match grid_object_placer {
            GridObjectPlacer::Building(building_type) => {
                overlay_config.secondary_mode = EnergySupplyOverlaySecondaryMode::Placing{
                    grid_coords: *grid_coords,
                    grid_imprint: almanach.get_building_grid_imprint(*building_type),
                    range: 15,
                };
            }
            _ => {
                overlay_config.secondary_mode = EnergySupplyOverlaySecondaryMode::None;
            }
        }
    }
}

fn on_grid_placer_exited_system(
    mut overlay_config: ResMut<EnergySupplyOverlayConfig>,
) {
    overlay_config.secondary_mode = EnergySupplyOverlaySecondaryMode::None;
}

fn create_energy_supply_overlay_startup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<EnergySupplyHeatmapMaterial>>,
) {
    let image = images.add(
        Image::new_fill(
            Extent3d{
                width: 100,
                height: 100,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &[0, 0, 0, 0],
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::default(),
        )
    );

    let full_world_size = 100. * CELL_SIZE;
    commands.spawn(
        MaterialMesh2dBundle {
            mesh: meshes.add(Rectangle::new(1.0, 1.0)).into(),
            transform: Transform::from_xyz(full_world_size / 2., full_world_size / 2., Z_OVERLAY_ENERGY_SUPPLY)
                .with_scale(Vec3::new(full_world_size, -full_world_size, full_world_size)), // Flip vertically due to coordinate system
            material: materials.add(EnergySupplyHeatmapMaterial {
                highlight_enabled: 0,
                heatmap: image,
            }),
            visibility: Visibility::Hidden,
            ..Default::default()
        }
    ).insert(EnergySupplyOverlay);
}

pub struct OverlayHeatmapCreator<'a> {
    energy_supply_grid: &'a EnergySupplyGrid,
    heatmap_data: &'a mut Vec<u8>
}
impl OverlayHeatmapCreator<'_> {
    const ALPHA_VALUE: u8 = 15;

    /// Imprint the current state of the energy supply grid into the heatmap
    /// Rules as as follows:
    /// - Alpha value(chunk[3]) above 0 means it has energy supply
    /// - Red value(chunk[0]) == 255 means it has supply but no power
    fn imprint_current_state(&mut self, highlight_supplier: Option<BuildingId>) {
        let mut idx = 0;
        let alpha_value = if highlight_supplier.is_some() { 5 } else { Self::ALPHA_VALUE };
        self.heatmap_data.chunks_mut(4).for_each(|chunk| {
            chunk[0] = 0;
            let grid_field =  &self.energy_supply_grid.grid[idx];
            if grid_field.has_supply() {
                // Mark as has supply
                chunk[3] = alpha_value;
                if let Some(highlighted_supplier) = highlight_supplier {
                    if grid_field.has_supplier(*highlighted_supplier) {
                        chunk[3] = Self::ALPHA_VALUE;
                    }
                }
                // Additional mark if it has no power
                if !grid_field.has_power() {
                    chunk[0] = 255;
                }
            } else {
                chunk[3] = 0;
            }
            idx += 1;
        });
    }
    fn coords_to_index(&self, coords: &GridCoords) -> usize {
        (coords.x * 4 + coords.y * self.energy_supply_grid.height * 4) as usize
    }

    /// Special version of `flooding::flood_energy_supply` to add the energy supply of a building we are currently placing to the overlay heatmap.
    /// It writes directly to the overlay texture, so it's only a visual cue that does not affect the actual grid.
    fn flood_potential_energy_supply_to_overlay_heatmap<'a>(
        &mut self,
        start_coords: impl Iterator<Item = &'a GridCoords>,
        range: usize,
    ) {
        VISITED_GRID.with_borrow_mut(|visited_grid| {
            visited_grid.resize_and_reset(self.energy_supply_grid.bounds());
            let mut queue = VecDeque::new();
            start_coords.for_each(|coords| {
                let heatmap_index = self.coords_to_index(coords) + 3;
                self.heatmap_data[heatmap_index] = Self::ALPHA_VALUE;
                queue.push_back((0, *coords));
                visited_grid.set_visited(*coords);
            });
            while let Some((distance, coords)) = queue.pop_front() {
                for (delta_x, delta_y) in CARDINAL_DIRECTIONS {
                    let new_coords = coords.shifted((delta_x, delta_y));
                    if !new_coords.is_in_bounds(self.energy_supply_grid.bounds())
                        || visited_grid.is_visited(new_coords)
                    {
                        continue;
                    }

                    visited_grid.set_visited(new_coords);

                    let heatmap_index = self.coords_to_index(&new_coords) + 3;
                    self.heatmap_data[heatmap_index] = Self::ALPHA_VALUE;

                    let new_distance = distance + 1;
                    if new_distance < range {
                        queue.push_back((new_distance, new_coords));
                    }
                }
            }
        });
    }
}