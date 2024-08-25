use crate::prelude::*;
use serde::{Deserialize, Serialize};

/// Defines what must happen for an objective to become active
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ObjectiveRequirements {
    None,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ObjectiveType {
    ClearQuantumFields,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectiveDetails {
    id_name: String,
    objective_type: ObjectiveType,
    requirements: ObjectiveRequirements,
}

#[derive(Resource, Default)]
pub struct ObjectivesCheckInactiveFlag(bool);

#[derive(Component)]
pub struct ObjectiveMarkerInactive;

#[derive(Component)]
pub struct ObjectiveMarkerInProgress;

#[derive(Component)]
pub struct ObjectiveMarkerCompleted;


pub struct BuilderObjective {
    pub objective_details: ObjectiveDetails,
}
impl BuilderObjective {
    pub fn new(objective_details: ObjectiveDetails) -> Self {
        Self {
            objective_details
        }
    }
    pub fn spawn(
        self, commands: &mut Commands,
        objectives_check_inactive_flag: &mut ObjectivesCheckInactiveFlag
    ) -> Entity {
        objectives_check_inactive_flag.0 = true;
        commands.spawn((
            Objective {
                text: String::new(),
                details: self.objective_details,
            },
            ObjectiveMarkerInactive,
        )).id()
    }
}


#[derive(Component)]
pub struct Objective {
    pub text: String,
    pub details: ObjectiveDetails,
}
#[derive(Component, Default)]
pub struct ObjectiveClearQuantumFields {
    total_quantum_fields: usize,
    completed_quantum_fields: usize,
}