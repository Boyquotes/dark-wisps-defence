use crate::map_objects::quantum_field::QuantumField;
use crate::prelude::*;

use lib_inventory::stats::StatsWispsKilled;

pub struct ObjectivesPlugin;
impl Plugin for ObjectivesPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ObjectivesReassesInactiveFlag>()
            .add_systems(PreUpdate, (
                (
                    reassess_inactive_objectives_system,
                ).run_if(in_state(GameState::Running)),
            ))
            .add_systems(Update, (
                (
                    on_objective_completed_system,
                    on_objective_failed_system,
                    update_clear_all_quantum_fields_system,
                    update_kill_wisps_system,
                ).run_if(in_state(GameState::Running)),
            ))
            .add_observer(Objective::on_add)
            .add_observer(ObjectiveDetails::on_add);
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
    KillWisps(usize),
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
#[require(Objective)]
pub struct ObjectiveDetails {
    pub id_name: String,
    pub objective_type: ObjectiveType,
    pub prerequisites: ObjectivePrerequisities,
}
impl ObjectiveDetails {
    fn on_add(
        trigger: Trigger<OnAdd, ObjectiveDetails>,
        mut commands: Commands,
        objective_details: Query<&ObjectiveDetails>,
        stats_wisps_killed: Res<StatsWispsKilled>,
    ) {
        let entity = trigger.target();
        let Ok(objective_details) = objective_details.get(entity) else { return; };
        match objective_details.objective_type {
            ObjectiveType::ClearAllQuantumFields => {
                commands.entity(entity).insert(ObjectiveClearAllQuantumFields::default());
            }
            ObjectiveType::KillWisps(target_amount) => {
                commands.entity(entity).insert(ObjectiveKillWisps{target_amount, started_amount: stats_wisps_killed.0});
            }
        }
    }
}

// When true, assess prerequisities of inactive objectives to see if they should become active
// You can use it as a command to change its value
#[derive(Resource, Default)]
pub struct ObjectivesReassesInactiveFlag(bool);
impl Command for ObjectivesReassesInactiveFlag {
    fn apply(self, world: &mut World) {
        world.resource_mut::<ObjectivesReassesInactiveFlag>().0 = self.0
    }
}

#[derive(Component, Default)]
pub struct ObjectiveMarkerInactive;
#[derive(Component)]
pub struct ObjectiveMarkerInProgress;
#[derive(Component)]
pub struct ObjectiveMarkerCompleted;
#[derive(Component)]
pub struct ObjectiveMarkerFailed;

#[derive(Component)]
pub struct ObjectiveCheckmark;
#[derive(Component)]
pub struct ObjectiveText;


#[derive(Component)]
#[require(ObjectiveMarkerInactive, MapBound)]
pub struct Objective {
    pub checkmark: Entity,
    pub text: Entity,
}
impl Default for Objective {
    fn default() -> Self {
        Self {
            checkmark: Entity::PLACEHOLDER,
            text: Entity::PLACEHOLDER,
        }
    }
}
impl Objective {
    fn on_add(
        trigger: Trigger<OnAdd, Objective>,
        mut commands: Commands,
        mut objectives: Query<(&mut Objective, &ObjectiveDetails)>,
        asset_server: Res<AssetServer>,
        mut objectives_check_inactive_flag: ResMut<ObjectivesReassesInactiveFlag>,
    ) {
        let entity = trigger.target();
        let Ok((mut objective, objective_details)) = objectives.get_mut(entity) else { 
            panic!("Objective is missing ObjectiveDetails!");
         };
        
        // Order re-checking objectives state
        objectives_check_inactive_flag.0 = true;

        // Create UI children
        objective.checkmark = commands.spawn((
            Node {
                width: Val::Px(16.),
                height: Val::Px(16.),
                left: Val::Px(2.),
                ..default()
            },
            ImageNode::new(asset_server.load("ui/objectives_check_active.png")),
            ObjectiveCheckmark,
        )).id();
        objective.text = commands.spawn((
            Text::new(objective_details.id_name.clone()),
            TextFont::default().with_font_size(12.),
            ObjectiveText,
        )).id();
        commands.entity(entity)
            .insert((
                ObjectiveMarkerInactive,
                Node {
                    width: Val::Percent(100.),
                    border: UiRect::all(Val::Px(2.)),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(5.),
                    ..default()
                },
                BackgroundColor::from(Color::linear_rgba(0.1, 0.3, 0.8, 0.7)),
                BorderRadius::all(Val::Px(7.)),
                BorderColor::from(Color::linear_rgba(0., 0.2, 0.8, 0.9)),
            ))
            .add_children(&[objective.checkmark, objective.text]);
    }
}

fn reassess_inactive_objectives_system(
    mut commands: Commands,
    mut reassesion_flag: ResMut<ObjectivesReassesInactiveFlag>,
    objectives: Query<(Entity, &ObjectiveDetails), With<ObjectiveMarkerInactive>>,
) {
    if !reassesion_flag.0 { return; }
    for (objective_entity, objective_details) in &objectives {
        match objective_details.prerequisites {
            ObjectivePrerequisities::None => (),
        }
        commands.entity(objective_entity)
            .insert(ObjectiveMarkerInProgress)
            .remove::<ObjectiveMarkerInactive>();
    }
    reassesion_flag.0 = false;
}

fn on_objective_completed_system(
    asset_server: Res<AssetServer>,
    mut objectives: Query<(&Objective, &mut BackgroundColor, &mut BorderColor), Added<ObjectiveMarkerCompleted>>,
    mut checkmarks: Query<&mut ImageNode, With<ObjectiveCheckmark>>,
) {
    for (objective, mut background_color, mut border_color) in &mut objectives {
        let mut checkmark = checkmarks.get_mut(objective.checkmark).unwrap();
        checkmark.image = asset_server.load("ui/objectives_check_completed.png");
        *background_color = Color::linear_rgba(0.1, 0.8, 0.3, 0.7).into();
        *border_color = Color::linear_rgba(0., 0.8, 0.2, 0.9).into();
    }
}

fn on_objective_failed_system(
    asset_server: Res<AssetServer>,
    mut objectives: Query<(&Objective, &mut BackgroundColor, &mut BorderColor), Added<ObjectiveMarkerFailed>>,
    mut checkmarks: Query<&mut ImageNode, With<ObjectiveCheckmark>>,
) {
    for (objective, mut background_color, mut border_color) in &mut objectives {
        let mut checkmark = checkmarks.get_mut(objective.checkmark).unwrap();
        checkmark.image = asset_server.load("ui/objectives_check_failed.png");
        *background_color = Color::linear_rgba(0.8, 0.1, 0.3, 0.7).into();
        *border_color = Color::linear_rgba(0.8, 0., 0.2, 0.9).into();
    }
}

// ---- SPECIFIC OBJECTIVES ----

#[derive(Component, Default)]
pub struct ObjectiveClearAllQuantumFields {
    completed_quantum_fields: usize,
}

// TODO: make it trigger only on quantum fieds change event
fn update_clear_all_quantum_fields_system(
    mut commands: Commands,
    mut objectives: Query<(Entity, &Objective, &mut ObjectiveClearAllQuantumFields), With<ObjectiveMarkerInProgress>>,
    quantum_fields: Query<&QuantumField>,
    mut texts: Query<&mut Text, With<ObjectiveText>>,
) {
    for (objective_entity, objective, mut objective_clear_all_quantum_fields) in &mut objectives {
        objective_clear_all_quantum_fields.completed_quantum_fields = 0;
        let total_quantum_fields = quantum_fields.iter().count();

        let mut text = texts.get_mut(objective.text).unwrap();
        text.0 = format!("Clear All Quantum Fields: {}/{}", objective_clear_all_quantum_fields.completed_quantum_fields, total_quantum_fields);

        if objective_clear_all_quantum_fields.completed_quantum_fields == total_quantum_fields {
            commands.entity(objective_entity)
                .remove::<ObjectiveMarkerInProgress>()
                .insert(ObjectiveMarkerCompleted);
            commands.queue(ObjectivesReassesInactiveFlag(true));
        }
    }
}

#[derive(Component, Default)]
pub struct ObjectiveKillWisps {
    target_amount: usize,
    started_amount: usize,
}

fn update_kill_wisps_system(
    mut commands: Commands,
    stats_wisps_killed: Res<StatsWispsKilled>,
    mut objectives: Query<(Entity, &Objective, &ObjectiveKillWisps), With<ObjectiveMarkerInProgress>>,
    mut texts: Query<&mut Text, With<ObjectiveText>>,
) {
    for (objective_entity, objective, objective_kill_wisps)  in &mut objectives {
        let current_amount = std::cmp::min(stats_wisps_killed.0 - objective_kill_wisps.started_amount, objective_kill_wisps.target_amount);
        let mut text = texts.get_mut(objective.text).unwrap();
        text.0 = format!("Kill Wisps: {}/{}", current_amount, objective_kill_wisps.target_amount);

        if current_amount == objective_kill_wisps.target_amount {
            commands.entity(objective_entity)
                .remove::<ObjectiveMarkerInProgress>()
                .insert(ObjectiveMarkerCompleted);
            commands.queue(ObjectivesReassesInactiveFlag(true));
        }

    }
}