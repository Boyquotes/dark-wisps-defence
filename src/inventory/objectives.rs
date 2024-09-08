use crate::{map_objects::quantum_field::QuantumField, prelude::*};
use serde::{Deserialize, Serialize};

pub struct ObjectivesPlugin;
impl Plugin for ObjectivesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<BuilderObjective>()
            .add_systems(PostUpdate, (
                BuilderObjective::spawn_system.run_if(on_event::<BuilderObjective>()),
            ))
            .add_systems(Update, (
                update_clear_all_quantum_fields_system,
            ));
    }
}

/// Defines what must happen for an objective to become active
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum ObjectivePrerequisities {
    None,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum ObjectiveType {
    ClearAllQuantumFields,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct ObjectiveDetails {
    pub id_name: String,
    pub objective_type: ObjectiveType,
    pub prerequisites: ObjectivePrerequisities,
}

// When true, assess prerequisities of inactive objectives to see if they should become active
#[derive(Resource, Default)]
pub struct ObjectivesCheckInactiveFlag(bool);

#[derive(Component)]
pub struct ObjectiveMarkerInactive;

#[derive(Component)]
pub struct ObjectiveMarkerInProgress;

#[derive(Component)]
pub struct ObjectiveMarkerCompleted;

#[derive(Component)]
pub struct Objective {
    pub text: Entity,
}

#[derive(Event)]
pub struct BuilderObjective {
    pub entity: Entity,
    pub objective_details: ObjectiveDetails,
}
impl BuilderObjective {
    pub fn new(entity: Entity, objective_details: ObjectiveDetails) -> Self {
        Self { entity, objective_details }
    }
    pub fn spawn_system(
        mut commands: Commands,
        mut events: EventReader<BuilderObjective>,
        mut objectives_check_inactive_flag: ResMut<ObjectivesCheckInactiveFlag>,
    ) {
        for BuilderObjective { entity, objective_details } in events.read() {
            objectives_check_inactive_flag.0 = true;
            let mut objective = Objective {
                text: Entity::PLACEHOLDER,
            };
            commands.entity(*entity).insert((
                objective_details.clone(),
                ObjectiveMarkerInactive,
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.),
                        border: UiRect::all(Val::Px(2.)),
                        ..default()
                    },
                    background_color: Color::linear_rgba(0.1, 0.3, 0.8, 0.7).into(),
                    border_radius: BorderRadius::all(Val::Px(7.)),
                    border_color: Color::linear_rgba(0., 0.2, 0.8, 0.9).into(),
                    ..default()
                }
            )).with_children(|parent| {
                objective.text = parent.spawn(TextBundle::from_sections([objective_details.id_name.clone().into()])).id();
            }).insert(objective);
        }
    }
}
impl Command for BuilderObjective {
    fn apply(self, world: &mut World) {
        world.send_event(self);
    }
}

#[derive(Component, Default)]
pub struct ObjectiveClearAllQuantumFields {
    total_quantum_fields: usize,
    completed_quantum_fields: usize,
}

// TODO: make it trigger only on quantum fieds change event
fn update_clear_all_quantum_fields_system(
    mut objectives: Query<&mut ObjectiveClearAllQuantumFields, With<ObjectiveMarkerInProgress>>,
    quantum_fields: Query<&QuantumField>,
) {
    for mut objective in &mut objectives {
        objective.completed_quantum_fields = 0;
        objective.total_quantum_fields = quantum_fields.iter().count();
    }
}