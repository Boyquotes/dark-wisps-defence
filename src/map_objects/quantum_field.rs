use bevy::prelude::*;
use crate::common::Z_OBSTACLE;
use crate::grids::common::{GridCoords, GridImprint};
use crate::grids::obstacles::{Field, ObstacleGrid};
use crate::map_objects::common::{ExpeditionTargetMarker, ExpeditionZone};
use crate::mouse::MouseInfo;
use crate::ui::common::AdvancedInteraction;
use crate::ui::grid_object_placer::{GridObjectPlacer, GridObjectPlacerRequest};

pub const QUANTUM_FIELD_MIN_IMPRINT_SIZE: i32 = 3;
pub const QUANTUM_FIELD_MAX_IMPRINT_SIZE: i32 = 6;

#[derive(Component, Clone, Debug)]
pub struct QuantumField {
    pub grid_imprint: GridImprint,
}
impl QuantumField {
    fn from_size(size: i32) -> Self {
        Self { grid_imprint: GridImprint::Rectangle { width: size, height: size } }
    }
}
impl Default for QuantumField {
    fn default() -> Self {
        Self { grid_imprint: GridImprint::Rectangle { width: QUANTUM_FIELD_MIN_IMPRINT_SIZE, height: QUANTUM_FIELD_MIN_IMPRINT_SIZE } }
    }
}

#[derive(Bundle)]
pub struct BuilderQuantumField {
    sprite: SpriteBundle,
    grid_position: GridCoords,
    quantum_field: QuantumField,
    expedition_zone: ExpeditionZone,
}
impl BuilderQuantumField {
    pub fn new(grid_position: GridCoords, grid_imprint: GridImprint) -> Self {
        Self {
            sprite: get_quantum_field_sprite_bundle(grid_position, grid_imprint),
            grid_position,
            quantum_field: QuantumField { grid_imprint },
            expedition_zone: ExpeditionZone::default(),
        }
    }
    pub fn spawn(self, commands: &mut Commands, obstacles_grid: &mut ObstacleGrid) -> Entity {
        let grid_position = self.grid_position;
        let grid_imprint = self.quantum_field.grid_imprint;
        // TODO: Remove ExpeditionTargetMarker as users should set targets themselves
        let entity = commands.spawn(self).insert(ExpeditionTargetMarker).id();
        obstacles_grid.imprint(grid_position, Field::QuantumField(entity), grid_imprint);
        entity
    }
}

pub fn get_quantum_field_sprite_bundle(grid_position: GridCoords, grid_imprint: GridImprint) -> SpriteBundle {
    SpriteBundle {
        sprite: Sprite {
            custom_size: Some(grid_imprint.world_size()),
            color: Color::INDIGO,
            ..Default::default()
        },
        transform: Transform::from_translation(
            grid_position.to_world_position_centered(grid_imprint).extend(Z_OBSTACLE)
        ),
        ..Default::default()
    }
}

pub fn remove_quantum_field(
    commands: &mut Commands,
    obstacle_grid: &mut ResMut<ObstacleGrid>,
    entity: Entity,
    grid_position: GridCoords,
    grid_imprint: GridImprint,
) {
    commands.entity(entity).despawn();
    obstacle_grid.deprint(grid_position, grid_imprint);
}


pub fn onclick_spawn_system(
    mut commands: Commands,
    mut obstacles_grid: ResMut<ObstacleGrid>,
    mouse: Res<Input<MouseButton>>,
    mouse_info: Res<MouseInfo>,
    grid_object_placer: Query<&GridObjectPlacer>,
    quantum_fields_query: Query<&GridCoords, With<QuantumField>>,
) {
    let GridObjectPlacer::QuantumField(quantum_field) = grid_object_placer.single() else { return; };
    let mouse_coords = mouse_info.grid_coords;
    if mouse_info.is_over_ui || !mouse_coords.is_in_bounds(obstacles_grid.bounds()) { return; }
    if mouse.pressed(MouseButton::Left) {
        // Place a quantum_field
        if obstacles_grid.imprint_query_all(mouse_coords, quantum_field.grid_imprint, |field| field.is_empty()) {
            BuilderQuantumField::new(mouse_coords, quantum_field.grid_imprint)
                .spawn(&mut commands, &mut obstacles_grid);
        }
    } else if mouse.pressed(MouseButton::Right) {
        // Remove a quantum_field
        match obstacles_grid[mouse_coords] {
            Field::QuantumField(entity) => {
                if let Ok(quantum_field_coords) = quantum_fields_query.get(entity) {
                    remove_quantum_field(&mut commands, &mut obstacles_grid, entity, *quantum_field_coords, quantum_field.grid_imprint);
                }
            },
            _ => {}
        }
    }
}

/// Widget for selecting QuantumField grid imprint size during construction
/// The widget consists of one horizontal layer containing left arrow button, text label specifying the imprint size and right arrow button
#[derive(Component)]
pub struct GridPlacerUiForQuantumField {
    pub imprint_size: i32,
}
impl GridPlacerUiForQuantumField {
    pub fn imprint_str(&self) -> String {
        format!("Quantum Field {}x{}", self.imprint_size, self.imprint_size)
    }
}
#[derive(Component)]
pub enum ArrowButton {
    Decrease,
    Increase,
}
impl ArrowButton {
    fn text(&self) -> &str {
        match self {
            ArrowButton::Decrease => "<",
            ArrowButton::Increase => ">",
        }
    }
}

pub fn create_grid_placer_ui_for_quantum_field_system(
    mut commands: Commands,
) {
    struct ArrowButtonBundle {
        button: ButtonBundle,
        text: TextBundle,
        arrow_button: ArrowButton,
        advanced_interaction: AdvancedInteraction,
    }
    impl ArrowButtonBundle {
        fn new(arrow_button: ArrowButton) -> Self {
            Self {
                button: ButtonBundle {
                    style: Style {
                        width: Val::Px(16.),
                        height: Val::Px(16.),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    background_color: Color::BLACK.into(),
                    z_index: ZIndex::Global(-1),
                    ..Default::default()
                },
                text: TextBundle::from_section(arrow_button.text(), TextStyle::default()),
                arrow_button,
                advanced_interaction: Default::default(),
            }
        }
        pub fn spawn(self, builder: &mut ChildBuilder) {
            builder.spawn((self.button, self.arrow_button, self.advanced_interaction)).with_children(|parent| {
                parent.spawn(self.text);
            });
        }
    }

    let grid_placer_ui_for_quantum_field = GridPlacerUiForQuantumField{ imprint_size: QUANTUM_FIELD_MIN_IMPRINT_SIZE };
    let ui_text = grid_placer_ui_for_quantum_field.imprint_str();
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(5.0),
                left: Val::Percent(50.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(2.5),
                ..default()
            },
            ..default()
        },
        grid_placer_ui_for_quantum_field,
    )).with_children(|parent| {
        ArrowButtonBundle::new(ArrowButton::Decrease).spawn(parent);
        parent.spawn(TextBundle::from_section(ui_text, TextStyle::default()));
        ArrowButtonBundle::new(ArrowButton::Increase).spawn(parent);
    });
}

pub fn operate_arrows_for_grid_placer_ui_for_quantum_field_system(
    mut ui: Query<(&mut Visibility, &Children, &mut GridPlacerUiForQuantumField)>,
    mut arrows: Query<(&ArrowButton, &AdvancedInteraction)>,
    mut texts: Query<&mut Text>,
    grid_object_placer: Query<&GridObjectPlacer>,
    mut placer_request: ResMut<GridObjectPlacerRequest>,
) {
    let (mut visibility, ui_children, mut grid_placer_ui) = ui.single_mut();
    let GridObjectPlacer::QuantumField(_) = grid_object_placer.single() else {
        *visibility = Visibility::Hidden;
        return;
    };
    *visibility = Visibility::Inherited;

    for (arrow_button, advanced_interaction) in arrows.iter_mut() {
        if advanced_interaction.was_just_released {
            match arrow_button {
                ArrowButton::Decrease => {
                    if grid_placer_ui.imprint_size > QUANTUM_FIELD_MIN_IMPRINT_SIZE {
                        grid_placer_ui.imprint_size -= 1;
                    }
                },
                ArrowButton::Increase => {
                    if grid_placer_ui.imprint_size < QUANTUM_FIELD_MAX_IMPRINT_SIZE {
                        grid_placer_ui.imprint_size += 1;
                    }
                },
            }

            let ui_text = grid_placer_ui.imprint_str();
            texts.get_mut(ui_children[1]).unwrap().sections[0].value = ui_text;
            placer_request.set(GridObjectPlacer::QuantumField(QuantumField::from_size(grid_placer_ui.imprint_size)));
        }
    }
}