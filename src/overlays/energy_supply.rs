use bevy::prelude::*;
use bevy::reflect::{TypePath};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat};
use bevy::sprite::{Material2d, MaterialMesh2dBundle};
use crate::buildings::common::BuildingId;
use crate::grids::base::GridVersion;
use crate::grids::common::CELL_SIZE;
use crate::grids::energy_supply::EnergySupplyGrid;

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

pub fn create_energy_supply_overlay_startup_system(
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
            // TODO: MIGRATION TODO, was: mesh: meshes.add(Mesh::from(shape::Quad::flipped(-Vec2::new(1.0, 1.0)))).into(),
            //mesh: meshes.add(Mesh::from(shape::Quad::flipped(-Vec2::new(1.0, 1.0)))).into(),
            mesh: meshes.add(Rectangle::new(1.0, 1.0)).into(),
            transform: Transform::from_xyz(full_world_size / 2., full_world_size / 2., 10.).with_scale(Vec3::new(full_world_size, -full_world_size, full_world_size)),// .with_scale(Vec3::splat(full_world_size)),
            material: materials.add(EnergySupplyHeatmapMaterial {
                heatmap: image,
            }),
            visibility: Visibility::Hidden,
            ..Default::default()
        }
    ).insert(EnergySupplyOverlay);
}

#[derive(Resource)]
pub enum EnergySupplyOverlayMode {
    None, // Do not show the overlay
    All(GridVersion), // Show all suppliers
    Single(BuildingId), // Show a single supplier
}

pub fn manage_energy_supply_overlay_mode_system(
    mut emissions_overlay_mode: ResMut<EnergySupplyOverlayMode>,
    keys: Res<ButtonInput<KeyCode>>,
    mut emission_material_visibility: Query<&mut Visibility, With<EnergySupplyOverlay>>,
) {
    if keys.just_pressed(KeyCode::KeyY) {
        let mut visibility = emission_material_visibility.single_mut();
        match *visibility {
            Visibility::Hidden => {
                *emissions_overlay_mode = EnergySupplyOverlayMode::All(GridVersion::default());
                *visibility = Visibility::Inherited;
            },
            _ => {
                *emissions_overlay_mode = EnergySupplyOverlayMode::None;
                *visibility = Visibility::Hidden;
            }
        }
    }
}

pub fn update_energy_supply_overlay_system(
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<EnergySupplyHeatmapMaterial>>,
    energy_supply_grid: Res<EnergySupplyGrid>,
    mut energy_supply_overlay_mode: ResMut<EnergySupplyOverlayMode>,
    energy_supply_overlay: Query<&Handle<EnergySupplyHeatmapMaterial>, With<EnergySupplyOverlay>>,
) {
    match &mut *energy_supply_overlay_mode {
        EnergySupplyOverlayMode::None => { return; },
        EnergySupplyOverlayMode::All(version) => {
            if *version != energy_supply_grid.version {
                *version = energy_supply_grid.version;
                let heatmap_material_handle = energy_supply_overlay.single();
                let heatmap_material = materials.get_mut(heatmap_material_handle).unwrap();
                let heatmap_image = images.get_mut(&heatmap_material.heatmap).unwrap();
                energy_supply_grid.imprint_into_heatmap(&mut heatmap_image.data);
            }
        }
        EnergySupplyOverlayMode::Single(id) => {
            todo!();
        }
    }
}