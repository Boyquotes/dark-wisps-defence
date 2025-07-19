use bevy::ui::widget::NodeImageMode;

use crate::{objectives::Objective, prelude::*};

pub struct ObjectivesPanelPlugin;
impl Plugin for ObjectivesPanelPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<ObjectivesPanelState>()
            .add_systems(Startup, |mut commands: Commands| { commands.spawn(ObjectivesPanel); })
            .add_systems(Update, (
                panel_transition_to_hidden_system.run_if(in_state(ObjectivesPanelState::TransitionToHidden)),
                panel_transition_to_visible_system.run_if(in_state(ObjectivesPanelState::TransitionToVisible)),
                on_click_show_hide_objectives_system,
            ))
            .add_observer(on_objective_created_observer)
            .add_observer(ObjectivesPanel::on_add);
    }
}

const SLIDING_SPEED: f32 = 800.;
const VISIBLE_TOP_POSITION: f32 = 5.;

#[derive(Component)]
#[require(Button)]
pub struct ObjectivesShowHideButton;
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum ObjectivesPanelState {
    Hidden,
    TransitionToVisible,
    #[default]
    Visible,
    TransitionToHidden,
}

#[derive(Component)]
pub struct ObjectivesPanel;
impl ObjectivesPanel {
    fn on_add(
        trigger: Trigger<OnAdd, ObjectivesPanel>,
        mut commands: Commands,
        asset_server: Res<AssetServer>,
    ) {
        let entity = trigger.target();
        commands.entity(entity).insert((
            Node {
                width: Val::Px(300.0),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                top: Val::Px(VISIBLE_TOP_POSITION),
                right: Val::Px(5.0),
                padding: UiRect::all(Val::Px(8.0)),
                row_gap: Val::Px(2.0),
                ..default()
            },
            ImageNode {
                image: asset_server.load("ui/objectives_panel.png"),
                image_mode: NodeImageMode::Sliced(TextureSlicer {
                    border: BorderRect::all(20.0),
                    center_scale_mode: SliceScaleMode::Stretch,
                    sides_scale_mode: SliceScaleMode::Stretch,
                    max_corner_scale: 1.0,
                }),
                ..default()
            },
            children![(
                Node {
                    width: Val::Px(32.0),
                    height: Val::Px(32.0),
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(-34.0),
                    right: Val::Px(5.0),
                    ..default()
                },
                ImageNode::new(asset_server.load("ui/objectives_panel.png")),
                ObjectivesShowHideButton,
            )],
        ));
    }
}

fn on_objective_created_observer(
    trigger: Trigger<OnAdd, Objective>,
    mut commands: Commands,
    objectives_panel: Query<Entity, With<ObjectivesPanel>>,
) {
    let Ok(objectives_panel) = objectives_panel.single() else { return; };
    let objective_entity = trigger.target();
    commands.entity(objectives_panel).add_child(objective_entity);
}

fn panel_transition_to_visible_system(
    time: Res<Time>,
    mut next_state: ResMut<NextState<ObjectivesPanelState>>,
    mut objectives_panel: Query<&mut Node, With<ObjectivesPanel>>,
) {
    let Ok(mut style) = objectives_panel.single_mut() else { return; };
    let current_top = match style.top {
        Val::Px(top) => top,
        _ => unreachable!(),
    };
    let new_top = current_top + time.delta_secs() * SLIDING_SPEED;
    if new_top < VISIBLE_TOP_POSITION {
        style.top = Val::Px(new_top);
    } else {
        style.top = Val::Px(VISIBLE_TOP_POSITION);
        next_state.set(ObjectivesPanelState::Visible);
    }
}

fn panel_transition_to_hidden_system(
    time: Res<Time>,
    mut next_state: ResMut<NextState<ObjectivesPanelState>>,
    mut objectives_panel: Query<(&ComputedNode, &mut Node), With<ObjectivesPanel>>,
) {
    let Ok((node, mut style)) = objectives_panel.single_mut() else { return; };
    let current_top = match style.top {
        Val::Px(top) => top,
        _ => unreachable!(),
    };
    let new_top = current_top - time.delta_secs() * SLIDING_SPEED;
    if new_top > -node.size().y {
        style.top = Val::Px(new_top);
    } else {
        style.top = Val::Px(-node.size().y);
        next_state.set(ObjectivesPanelState::Hidden);
    }
}

fn on_click_show_hide_objectives_system(
    current_state: Res<State<ObjectivesPanelState>>,
    mut next_state: ResMut<NextState<ObjectivesPanelState>>,
    objectives_show_hide_button: Query<&Interaction, (With<ObjectivesShowHideButton>, Changed<Interaction>)>,
) {
    let Ok(interaction) = objectives_show_hide_button.single() else { return; };
    if matches!(*interaction, Interaction::Pressed) {
        match current_state.get() {
            ObjectivesPanelState::Hidden => next_state.set(ObjectivesPanelState::TransitionToVisible),
            ObjectivesPanelState::Visible => next_state.set(ObjectivesPanelState::TransitionToHidden),
            _ => {}
        }
    }
}