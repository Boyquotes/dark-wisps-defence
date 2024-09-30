use crate::prelude::*;
use crate::ui::display_building_info::BuildingUiFocusChangedEvent;
use bevy::reflect::TypePath;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat};
use bevy::sprite::{Material2d, MaterialMesh2dBundle};
use crate::grids::base::GridVersion;
use crate::grids::energy_supply::EnergySupplyGrid;

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
            .add_systems(Update, (
                on_config_change_system.run_if(resource_changed::<EnergySupplyOverlayConfig>),
                refresh_display_system.run_if(in_state(EnergySupplyOverlayState::Show)),
                manage_energy_supply_overlay_global_mode_system,
                on_building_ui_focus_changed_system.run_if(on_event::<BuildingUiFocusChangedEvent>()),
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
    pub highlighted_supplier: Option<BuildingId>,
}
// pub struct CommandEnergySupplyOverlayHighlightStart(BuildingId);
// impl Command for CommandEnergySupplyOverlayHighlightStart {
//     fn apply(self, world: &mut World) {
//         let mut config = world.get_resource_mut::<EnergySupplyOverlayConfig>().unwrap();
//         config.highlighted_supplier = Some(self.0);
//     }
// }
// pub struct CommandEnergySupplyOverlayHighlightEnd;
// impl Command for CommandEnergySupplyOverlayHighlightEnd {
//     fn apply(self, world: &mut World) {
//         let mut config = world.get_resource_mut::<EnergySupplyOverlayConfig>().unwrap();
//         config.highlighted_supplier = None;
//     }
// }


#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct EnergySupplyHeatmapMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub heatmap: Handle<Image>,
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
    if overlay_config.is_overlay_globally_enabled || overlay_config.highlighted_supplier.is_some() {
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
    buildings: Query<(&GridCoords), With<Building>>,
) {
    if overlay_config.grid_version != energy_supply_grid.version || overlay_config.highlighted_supplier.is_some() {
        overlay_config.grid_version = energy_supply_grid.version;
        let heatmap_material_handle = energy_supply_overlay.single();
        let heatmap_material = materials.get_mut(heatmap_material_handle).unwrap();
        let heatmap_image = images.get_mut(&heatmap_material.heatmap).unwrap();
        let mut overlay_creator = OverlayHeatmapCreator { energy_supply_grid: &energy_supply_grid, heatmap_data: &mut heatmap_image.data };
        overlay_creator.imprint_current_state(overlay_config.highlighted_supplier); 
    }
}

fn on_building_ui_focus_changed_system(
    mut events: EventReader<BuildingUiFocusChangedEvent>,
    mut overlay_config: ResMut<EnergySupplyOverlayConfig>,
) {
    for event in events.read() {
        match event {
            BuildingUiFocusChangedEvent::Focus(building_id) => {
                overlay_config.highlighted_supplier = Some(*building_id);
            }
            BuildingUiFocusChangedEvent::Unfocus => {
                overlay_config.highlighted_supplier = None;
            }
        }
    }
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
            &[255, 255, 0, 127],
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::default(),
        )
    );

    let full_world_size = 100. * CELL_SIZE;
    commands.spawn(
        MaterialMesh2dBundle {
            mesh: meshes.add(Rectangle::new(1.0, 1.0)).into(),
            transform: Transform::from_xyz(full_world_size / 2., full_world_size / 2., 10.)
                .with_scale(Vec3::new(full_world_size, -full_world_size, full_world_size)), // Flip vertically due to coordinate system
            material: materials.add(EnergySupplyHeatmapMaterial {
                heatmap: image,
            }),
            visibility: Visibility::Hidden,
            ..Default::default()
        }
    ).insert(EnergySupplyOverlay);
}

struct OverlayHeatmapCreator<'a> {
    energy_supply_grid: &'a EnergySupplyGrid,
    heatmap_data: &'a mut Vec<u8>
}
impl OverlayHeatmapCreator<'_> {
    fn imprint_current_state(&mut self, highlight_supplier: Option<BuildingId>) {
        let mut idx = 0;
        let alpha_value = if highlight_supplier.is_some() { 5 } else { 15 };
        self.heatmap_data.chunks_mut(4).for_each(|chunk| {
            let grid_field =  &self.energy_supply_grid.grid[idx];
            if grid_field.has_supply() {
                chunk[3] = alpha_value;
                if let Some(highlighted_supplier) = highlight_supplier {
                    if grid_field.has_supplier(*highlighted_supplier) {
                        chunk[3] = 30;
                    }
                }
            } else {
                chunk[3] = 0;
            }
            idx += 1;
        });
    }
}