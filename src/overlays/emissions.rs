use bevy::input::common_conditions::input_just_released;
use bevy::reflect::TypePath;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat};
use bevy::sprite::{AlphaMode2d, Material2d, Material2dPlugin};

use lib_grid::grids::emissions::{EmissionsGrid, EmissionsType};

use crate::map_editor::MapInfo;
use crate::prelude::*;

pub struct EmissionsPlugin;
impl Plugin for EmissionsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(Material2dPlugin::<EmissionHeatmapMaterial>::default())
            .insert_resource(EmissionsOverlayMode::Energy) 
            .add_systems(OnEnter(MapLoadingStage::ResetGridsAndResources), EmissionsOverlay::create)
            .add_systems(Update, (
                update_emissions_overlay_system.run_if(resource_changed::<EmissionsOverlayMode>.or(resource_changed::<EmissionsGrid>)),
                manage_emissions_overlay_mode_system.run_if(input_just_released(KeyCode::Digit6)), // Switch overlay on/off 
            ));
    }
}

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct EmissionHeatmapMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub heatmap: Handle<Image>,
}

impl Material2d for EmissionHeatmapMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/emissions_map.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

#[derive(Component)]
pub struct EmissionsOverlay;
impl EmissionsOverlay {
    pub fn create(
        mut commands: Commands,
        map_info: Res<MapInfo>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut images: ResMut<Assets<Image>>,
        mut materials: ResMut<Assets<EmissionHeatmapMaterial>>,
        overlay: Query<Entity, With<EmissionsOverlay>>,
    ) {
        // First remove old overlay if exists
        if let Ok(overlay_entity) = overlay.single() {
            commands.entity(overlay_entity).despawn();
        };

        let image = images.add(
            Image::new_fill(
                Extent3d{
                    width: map_info.grid_width as u32,
                    height: map_info.grid_height as u32,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                &[0, 0, 0, 0],
                TextureFormat::Rgba8Unorm,
                RenderAssetUsages::default()
            )
        );
    
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(1.0, 1.0))),
            MeshMaterial2d(materials.add(EmissionHeatmapMaterial {
                heatmap: image,
            })),
            Transform::from_xyz(map_info.world_width as f32 / 2., map_info.world_height as f32 / 2., 0.)
                .with_scale(Vec3::new(map_info.world_width as f32, -map_info.world_height as f32, 1.)), // Flip vertically due to coordinate system
            EmissionsOverlay,
            Visibility::Hidden,
        ));
    }
}

/// Keep tracks of which version does the overlay use
#[derive(Resource)]
pub enum EmissionsOverlayMode {
    None,
    Energy,
}

pub fn manage_emissions_overlay_mode_system(
    mut emissions_overlay_mode: ResMut<EmissionsOverlayMode>,
) {
    if matches!(*emissions_overlay_mode, EmissionsOverlayMode::None) {
        *emissions_overlay_mode = EmissionsOverlayMode::Energy;
    } else {
        *emissions_overlay_mode = EmissionsOverlayMode::None;
    }
}

pub fn update_emissions_overlay_system(
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<EmissionHeatmapMaterial>>,
    emissions_grid: Res<EmissionsGrid>,
    mut emissions_overlay_mode: ResMut<EmissionsOverlayMode>,
    mut emissions_overlay: Query<(&mut Visibility, &MeshMaterial2d<EmissionHeatmapMaterial>), With<EmissionsOverlay>>,
    mut last_grid_version: Local<GridVersion>,
) {
    let Ok((mut visibility, heatmap_material_handle)) = emissions_overlay.single_mut() else { return; };
    match &mut *emissions_overlay_mode {
        EmissionsOverlayMode::None => { 
            *visibility = Visibility::Hidden;
        },
        EmissionsOverlayMode::Energy => {
            *visibility = Visibility::Inherited;
            if *last_grid_version != emissions_grid.version.energy {
                *last_grid_version = emissions_grid.version.energy;
                let heatmap_material = materials.get_mut(heatmap_material_handle).unwrap();
                let heatmap_image = images.get_mut(&heatmap_material.heatmap).unwrap();
                emissions_grid.imprint_into_heatmap(&mut heatmap_image.data.as_mut().unwrap(), EmissionsType::Energy);
            }
        }
    }
}