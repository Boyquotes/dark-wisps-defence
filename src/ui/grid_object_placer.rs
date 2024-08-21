use bevy::prelude::*;
use crate::buildings::common::{BuildingType, TowerType};
use crate::buildings::common_components::Building;
use crate::buildings::mining_complex::MINING_COMPLEX_GRID_IMPRINT;
use crate::grids::common::GridImprint;
use crate::grids::energy_supply::EnergySupplyGrid;
use crate::grids::obstacles::{ObstacleGrid};
use crate::map_objects::dark_ore::{DARK_ORE_GRID_IMPRINT};
use crate::map_objects::quantum_field::QuantumField;
use crate::map_objects::walls::WALL_GRID_IMPRINT;
use crate::mouse::MouseInfo;
use crate::ui::interaction_state::UiInteractionState;


#[derive(Resource, Default)]
pub struct GridObjectPlacerRequest(pub Option<GridObjectPlacer>);
impl GridObjectPlacerRequest {
    pub fn set(&mut self, request: GridObjectPlacer) { self.0 = Some(request); }
    pub fn clear(&mut self) { self.0 = None; }
}

#[derive(Component, Default, Clone, Debug)]
pub enum GridObjectPlacer {
    #[default]
    None,
    Building(Building),
    Wall,
    DarkOre,
    MiningComplex,
    QuantumField(QuantumField),
}
impl GridObjectPlacer {
    pub fn as_grid_imprint(&self) -> GridImprint {
        match self {
            GridObjectPlacer::Building(building) => building.grid_imprint,
            GridObjectPlacer::Wall => WALL_GRID_IMPRINT,
            GridObjectPlacer::DarkOre => DARK_ORE_GRID_IMPRINT,
            GridObjectPlacer::MiningComplex => MINING_COMPLEX_GRID_IMPRINT,
            GridObjectPlacer::QuantumField(quantum_field) => quantum_field.grid_imprint,
            GridObjectPlacer::None => unreachable!(),
        }
    }
}

impl From<BuildingType> for GridObjectPlacer {
    fn from(building_type: BuildingType) -> Self {
        match building_type {
            BuildingType::MiningComplex => GridObjectPlacer::MiningComplex,
            _ => GridObjectPlacer::Building(building_type.into()),
        }
    }
}

pub fn create_grid_object_placer_system(mut commands: Commands) {
    commands.spawn((
        GridObjectPlacer::default(),
        SpriteBundle::default()
    ));
}

pub fn update_grid_object_placer_system(
    obstacle_grid: Res<ObstacleGrid>,
    energy_supply_grid: Res<EnergySupplyGrid>,
    ui_interaction_state: Res<UiInteractionState>,
    mouse_info: Res<MouseInfo>,
    mut placer: Query<(&mut Transform, &mut Sprite, &mut Visibility, &mut GridObjectPlacer)>,
) {
    let (mut transform, mut sprite, mut visibility, mut grid_object_placer) = placer.single_mut();
    match *ui_interaction_state {
        UiInteractionState::PlaceGridObject => {}
        _ => {
            *visibility = Visibility::Hidden;
            *grid_object_placer = GridObjectPlacer::None;
            return;
        }
    };
    transform.translation = mouse_info.grid_coords.to_world_position().extend(10.);
    let is_imprint_placable = match &*grid_object_placer {
        GridObjectPlacer::None => false,
        GridObjectPlacer::MiningComplex => {
            transform.translation += MINING_COMPLEX_GRID_IMPRINT.world_center().extend(0.);
            obstacle_grid.imprint_query_all(mouse_info.grid_coords, MINING_COMPLEX_GRID_IMPRINT, |field| field.is_dark_ore())

        }
        placed => {
            let imprint = placed.as_grid_imprint();
            transform.translation += imprint.world_center().extend(0.);
            obstacle_grid.imprint_query_all(mouse_info.grid_coords, imprint, |field| field.is_empty())
        }
    };
    let (needs_energy_supply, is_imprint_suppliable) = match &*grid_object_placer {
        GridObjectPlacer::Building(building) => match building.building_type {
            BuildingType::Tower(_) | BuildingType::ExplorationCenter => {
                (true, energy_supply_grid.is_imprint_suppliable(mouse_info.grid_coords, building.grid_imprint))
            },
            _ => (false, false)
        },
        GridObjectPlacer::MiningComplex => (true, energy_supply_grid.is_imprint_suppliable(mouse_info.grid_coords, MINING_COMPLEX_GRID_IMPRINT)),
        _ => (false, false)
    };

    sprite.color = if is_imprint_placable {
        if needs_energy_supply && !is_imprint_suppliable {
            Color::srgba(1.0, 1.0, 0.0, 0.2)
        } else {
            Color::srgba(0.0, 1.0, 0.0, 0.2)
        }
    } else {
        Color::srgba(1.0, 0.0, 0.0, 0.2)
    };
}

pub fn keyboard_input_system(
    mut grid_object_placer_request: ResMut<GridObjectPlacerRequest>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let placer_request = {
        if keys.just_pressed(KeyCode::KeyW) {
            GridObjectPlacer::Wall
        } else if keys.just_pressed(KeyCode::KeyO) {
            GridObjectPlacer::DarkOre
        } else if keys.just_pressed(KeyCode::KeyQ) {
            GridObjectPlacer::QuantumField(QuantumField::default())
        } else if keys.just_pressed(KeyCode::KeyM) {
            GridObjectPlacer::MiningComplex
        } else if keys.just_pressed(KeyCode::KeyE) {
            GridObjectPlacer::Building(BuildingType::EnergyRelay.into())
        } else if keys.just_pressed(KeyCode::KeyX) {
            GridObjectPlacer::Building((BuildingType::ExplorationCenter).into())
        } else if keys.just_pressed(KeyCode::Digit1) {
            GridObjectPlacer::Building(BuildingType::Tower(TowerType::Blaster).into())
        } else if keys.just_pressed(KeyCode::Digit2) {
            GridObjectPlacer::Building(BuildingType::Tower(TowerType::Cannon).into())
        } else if keys.just_pressed(KeyCode::Digit3) {
            GridObjectPlacer::Building(BuildingType::Tower(TowerType::RocketLauncher).into())
        } else {
            return
        }
    };
    grid_object_placer_request.set(placer_request);
}

pub fn on_request_grid_object_placer_system(
    mut ui_interaction_state: ResMut<UiInteractionState>,
    mut placer: Query<(&mut Sprite, &mut Visibility, &mut GridObjectPlacer)>,
    mut placer_request: ResMut<GridObjectPlacerRequest>,
) {
    let Some(placer_request) = placer_request.0.take() else { return; };
    if !matches!(*ui_interaction_state, UiInteractionState::Free | UiInteractionState::PlaceGridObject) {
        return;
    }
    let (mut sprite, mut visibility, mut grid_object_placer) = placer.single_mut();
    *ui_interaction_state = UiInteractionState::PlaceGridObject;
    *visibility = Visibility::Visible;
    *grid_object_placer = placer_request;
    sprite.custom_size = match &*grid_object_placer {
        GridObjectPlacer::None => None,
        placer => Some(placer.as_grid_imprint().world_size()),
    }
}
