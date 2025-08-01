use bevy::{
    asset::Assets,
    reflect::TypePath,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{AsBindGroup, ShaderRef, ShaderType, Extent3d, TextureDimension, TextureFormat},
        storage::ShaderStorageBuffer,
    },
    sprite::{AlphaMode2d, Material2d, Material2dPlugin, MeshMaterial2d},
};
use lib_grid::grids::energy_supply::EnergySupplyGrid;

use crate::prelude::*;
use crate::ui::display_info_panel::{UiMapObjectFocusedTrigger, UiMapObjectUnfocusedTrigger};
use crate::ui::grid_object_placer::GridObjectPlacer;

pub struct EnergySupplyOverlayPlugin;
impl Plugin for EnergySupplyOverlayPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(Material2dPlugin::<EnergySupplyHeatmapMaterial>::default())
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
                on_grid_placer_changed_system.run_if(in_state(UiInteraction::PlaceGridObject)),
            ));
        app.world_mut().add_observer(on_building_ui_focused_trigger);
        app.world_mut().add_observer(on_building_ui_unfocused_trigger);

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
    Highlight{ building: Entity },
    Placing{grid_coords: GridCoords, grid_imprint: GridImprint, range: usize},
}
impl EnergySupplyOverlaySecondaryMode {
    pub fn is_none(&self) -> bool { matches!(self, EnergySupplyOverlaySecondaryMode::None) }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, ShaderType)]
struct UniformData {
    grid_width: u32,
    grid_height: u32,
    highlight_enabled: u32,
}

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct EnergySupplyHeatmapMaterial {
    #[storage(0, read_only)]
    pub energy_cells: Handle<ShaderStorageBuffer>,
    #[uniform(1)]
    pub uniforms: UniformData,
}
impl Material2d for EnergySupplyHeatmapMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/energy_supply_map.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
impl EnergySupplyHeatmapMaterial {
    fn configure(&mut self, secondary_mode: &EnergySupplyOverlaySecondaryMode) {
        match secondary_mode {
            EnergySupplyOverlaySecondaryMode::Highlight{building: _} => {
                self.uniforms.highlight_enabled = 1;
            }
            _ => self.uniforms.highlight_enabled = 0,
        }
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
    *emission_material_visibility.single_mut().unwrap() = Visibility::Inherited;
}
fn on_overlay_hide_system(
    mut emission_material_visibility: Query<&mut Visibility, With<EnergySupplyOverlay>>,
) {
    *emission_material_visibility.single_mut().unwrap() = Visibility::Hidden;
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
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    mut materials: ResMut<Assets<EnergySupplyHeatmapMaterial>>,
    energy_supply_grid: Res<EnergySupplyGrid>,
    mut overlay_config: ResMut<EnergySupplyOverlayConfig>,
    energy_supply_overlay: Query<&MeshMaterial2d<EnergySupplyHeatmapMaterial>, With<EnergySupplyOverlay>>,
    mut last_secondary_mode: Local<EnergySupplyOverlaySecondaryMode>,
) {
    if overlay_config.grid_version != energy_supply_grid.version || overlay_config.secondary_mode != *last_secondary_mode {
        *last_secondary_mode = overlay_config.secondary_mode.clone();
        overlay_config.grid_version = energy_supply_grid.version;
        let Ok(heatmap_material_handle) = energy_supply_overlay.single() else { return; };
        let heatmap_material = materials.get_mut(heatmap_material_handle).unwrap();
        heatmap_material.configure(&overlay_config.secondary_mode);
        
        // Generate buffer data
        let overlay_creator = OverlayBufferCreator { energy_supply_grid: &energy_supply_grid };
        let buffer_data = match &overlay_config.secondary_mode {
            EnergySupplyOverlaySecondaryMode::None => {
                overlay_creator.generate_buffer_data(None)
            }
            EnergySupplyOverlaySecondaryMode::Highlight{ building } => {
                overlay_creator.generate_buffer_data(Some(*building))
            }
            EnergySupplyOverlaySecondaryMode::Placing{grid_coords, grid_imprint, range} => {
                // TODO: Handle placing mode - for now just use basic data
                overlay_creator.generate_buffer_data(None)
            }
        };
        
        // Create ShaderStorageBuffer
        let storage_buffer = ShaderStorageBuffer::from(buffer_data.as_slice());
        let buffer_handle = buffers.add(storage_buffer);
        
        // Update material with new buffer
        heatmap_material.energy_cells = buffer_handle;
        let bounds = energy_supply_grid.bounds();
        heatmap_material.uniforms.grid_width = bounds.0 as u32; // width is first element
        heatmap_material.uniforms.grid_height = bounds.1 as u32; // height is second element
    }
}

fn on_building_ui_focused_trigger(
    trigger: Trigger<UiMapObjectFocusedTrigger>,
    mut overlay_config: ResMut<EnergySupplyOverlayConfig>,
    buildings: Query<&BuildingType>,
) {
    let focused_building = trigger.target();
    if buildings.contains(focused_building) {
        overlay_config.secondary_mode = EnergySupplyOverlaySecondaryMode::Highlight{building: focused_building};
    } else {
        overlay_config.secondary_mode = EnergySupplyOverlaySecondaryMode::None;
    }
}

fn on_building_ui_unfocused_trigger(
    _trigger: Trigger<UiMapObjectUnfocusedTrigger>,
    mut overlay_config: ResMut<EnergySupplyOverlayConfig>,
) {
    overlay_config.secondary_mode = EnergySupplyOverlaySecondaryMode::None;
}

fn on_grid_placer_changed_system(
    almanach: Res<Almanach>,
    mut overlay_config: ResMut<EnergySupplyOverlayConfig>,
    grid_object_placer: Query<(&GridObjectPlacer, &GridCoords)>,
    mut last_grid_object_placer: Local<(GridObjectPlacer, GridCoords)>,
) {
    let Ok((grid_object_placer, grid_coords)) = grid_object_placer.single() else { return; };
    if grid_object_placer != &last_grid_object_placer.0 || grid_coords != &last_grid_object_placer.1 {
        *last_grid_object_placer = (grid_object_placer.clone(), *grid_coords);
        let (grid_object_placer, grid_coords) = (grid_object_placer, grid_coords);
        match grid_object_placer {
            GridObjectPlacer::Building(building_type) if building_type.is_energy_supplier() => {
                overlay_config.secondary_mode = EnergySupplyOverlaySecondaryMode::Placing{
                    grid_coords: *grid_coords,
                    grid_imprint: almanach.get_building_info(*building_type).grid_imprint,
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
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(1.0, 1.0))),
        MeshMaterial2d(materials.add(EnergySupplyHeatmapMaterial {
            energy_cells: Handle::default(),
            uniforms: UniformData {
                grid_width: 0,
                grid_height: 0,
                highlight_enabled: 0,
            },
        })),
        Transform::from_xyz(full_world_size / 2., full_world_size / 2., Z_OVERLAY_ENERGY_SUPPLY)
                .with_scale(Vec3::new(full_world_size, -full_world_size, full_world_size)), // Flip vertically due to coordinate system
        Visibility::Hidden,
        EnergySupplyOverlay
    ));
}

/// Energy supply cell data for GPU buffer
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable, ShaderType)]
struct EnergySupplyCell {
    /// Whether this cell has energy supply (0 = false, 1 = true)
    has_supply: u32,
    /// Whether this cell has power connection (0 = false, 1 = true) 
    has_power: u32,
    /// Highlight level: 0 = None, 1 = Dimmed, 2 = Highlighted
    highlight_level: u32,
    /// Entity ID of primary supplier (0 if none)
    supplier_entity: u32,
}

#[derive(Debug, Clone, PartialEq)]
enum HighlightLevel {
    /// No supply - completely transparent
    None = 0,
    /// Has supply but another building is highlighted - dimmed display
    Dimmed = 1,
    /// This building's energy supply is highlighted
    Highlighted = 2,
}

impl EnergySupplyCell {
    /// Create a cell representing no energy supply
    fn none() -> Self {
        Self {
            has_supply: 0,
            has_power: 0,
            highlight_level: HighlightLevel::None as u32,
            supplier_entity: 0,
        }
    }

    /// Create a cell with supply and power status
    fn with_supply(has_power: bool, highlight_level: HighlightLevel, supplier_entity: Option<Entity>) -> Self {
        Self {
            has_supply: 1,
            has_power: has_power as u32,
            highlight_level: highlight_level as u32,
            supplier_entity: supplier_entity.map(|e| e.index()).unwrap_or(0),
        }
    }
}
pub struct OverlayBufferCreator<'a> {
    energy_supply_grid: &'a EnergySupplyGrid,
}
impl OverlayBufferCreator<'_> {
    /// Generate buffer data for the entire energy supply grid
    fn generate_buffer_data(&self, highlight_supplier: Option<Entity>) -> Vec<EnergySupplyCell> {
        (0..self.energy_supply_grid.grid.len())
            .map(|idx| self.create_cell_for_grid_field(idx, highlight_supplier))
            .collect()
    }
    
    /// If `highlight_supplier` is provided, only its range will be shown at full color, other ranges will be dimmed
    fn create_cell_for_grid_field(&self, idx: usize, highlight_supplier: Option<Entity>) -> EnergySupplyCell {
        let grid_field = &self.energy_supply_grid.grid[idx];
        
        if !grid_field.has_supply() {
            return EnergySupplyCell::none();
        }
        
        let highlight_level = match highlight_supplier {
            Some(supplier) if grid_field.has_supplier(supplier) => HighlightLevel::Highlighted,
            Some(_) => HighlightLevel::Dimmed, // There is supplier, but it's not our supplier.
            None => HighlightLevel::Highlighted, // Every supplier is highlighted
        };
        
        // TODO: Need to access primary supplier properly when public API is available
        let primary_supplier = None; // grid_field doesn't expose suppliers publicly
        EnergySupplyCell::with_supply(grid_field.has_power(), highlight_level, primary_supplier)
    }
    
    // TODO: Implement placing mode preview for buffer-based approach
    // The old texture-based flood method has been removed since we now use GPU buffers
}