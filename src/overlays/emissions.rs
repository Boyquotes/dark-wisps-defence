use bevy::prelude::*;
use bevy::reflect::{TypePath};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat};
use bevy::sprite::{Material2d, MaterialMesh2dBundle};
use crate::grids::base::GridVersion;
use crate::grids::common::CELL_SIZE;
use crate::grids::emissions::{EmissionsGrid, EmissionsType};

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
}

#[derive(Component)]
pub struct EmissionsOverlay;

pub fn create_emissions_overlay_startup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<EmissionHeatmapMaterial>>,
) {
    let image = images.add(
        Image::new_fill(
            Extent3d{
                width: 100,
                height: 100,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &[255, 0, 0, 127],
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::default()
        )
    );


    let full_world_size = 100. * CELL_SIZE;
    commands.spawn(
        MaterialMesh2dBundle {
            mesh: meshes.add(Mesh::from(shape::Quad::flipped(-Vec2::new(1.0, 1.0)))).into(),
            transform: Transform::from_xyz(full_world_size / 2., full_world_size / 2., 0.).with_scale(Vec3::splat(full_world_size)),
            material: materials.add(EmissionHeatmapMaterial {
                heatmap: image,
            }),
            ..Default::default()
        }
    ).insert(EmissionsOverlay);
}

/// Keep tracks of which version does the overlay use
#[derive(Resource)]
pub enum EmissionsOverlayMode {
    None,
    Energy(GridVersion),
}

pub fn manage_emissions_overlay_mode_system(
    mut emissions_overlay_mode: ResMut<EmissionsOverlayMode>,
    keys: Res<ButtonInput<KeyCode>>,
    mut emission_material_visibility: Query<&mut Visibility, With<EmissionsOverlay>>,
) {
    let mut visibility = emission_material_visibility.single_mut();
    if keys.just_pressed(KeyCode::Digit6) {
        *emissions_overlay_mode = EmissionsOverlayMode::None;
        *visibility = Visibility::Hidden;
    } else if keys.just_pressed(KeyCode::Digit7) {
        if !matches!(*emissions_overlay_mode, EmissionsOverlayMode::Energy(_)) {
            *emissions_overlay_mode = EmissionsOverlayMode::Energy(GridVersion::default());
        }
        *visibility = Visibility::Inherited;
    }
}

pub fn update_emissions_overlay_system(
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<EmissionHeatmapMaterial>>,
    emissions_grid: Res<EmissionsGrid>,
    mut emissions_overlay_mode: ResMut<EmissionsOverlayMode>,
    emissions_overlay: Query<&Handle<EmissionHeatmapMaterial>, With<EmissionsOverlay>>,
) {
    match &mut *emissions_overlay_mode {
        EmissionsOverlayMode::None => { return; },
        EmissionsOverlayMode::Energy(version) => {
            if *version != emissions_grid.version.energy {
                *version = emissions_grid.version.energy;
                let heatmap_material_handle = emissions_overlay.single();
                let heatmap_material = materials.get_mut(heatmap_material_handle).unwrap();
                let heatmap_image = images.get_mut(&heatmap_material.heatmap).unwrap();
                emissions_grid.imprint_into_heatmap(&mut heatmap_image.data, EmissionsType::Energy);
            }
        }
    }
}