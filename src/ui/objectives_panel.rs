use crate::{inventory::objectives::BuilderObjective, prelude::*};

pub struct ObjectivesPanelPlugin;
impl Plugin for ObjectivesPanelPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<ObjectivesPanelState>()
            .add_systems(Startup, initialize_objectives_panel_system)
            .add_systems(Update, (
                on_objective_created_system.run_if(on_event::<BuilderObjective>()),
                panel_transition_to_hidden_system.run_if(in_state(ObjectivesPanelState::TransitionToHidden)),
                panel_transition_to_visible_system.run_if(in_state(ObjectivesPanelState::TransitionToVisible)),
                on_click_show_hide_objectives_system,
            ));
    }
}

const SLIDING_SPEED: f32 = 800.;
const VISIBLE_TOP_POSITION: f32 = 5.;

#[derive(Component)]
pub struct ObjectivesPanel;
#[derive(Component)]
pub struct ObjectivesShowHideButton;
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum ObjectivesPanelState {
    Hidden,
    TransitionToVisible,
    #[default]
    Visible,
    TransitionToHidden,
}

fn initialize_objectives_panel_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Px(300.0),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                top: Val::Px(VISIBLE_TOP_POSITION),
                right: Val::Px(5.0),
                padding: UiRect::all(Val::Px(8.0)),
                row_gap: Val::Px(2.0),
                ..default()
            },
            ..default()
        },
        UiImage::new(asset_server.load("ui/objectives_panel.png")),
        ImageScaleMode::Sliced(TextureSlicer {
            border: BorderRect::square(20.0),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 1.0,
        }),
        ObjectivesPanel,
    )).with_children(|parent| {
        parent.spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(32.0),
                    height: Val::Px(32.0),
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(-34.0),
                    right: Val::Px(5.0),
                    ..default()
                },
                image: UiImage::new(asset_server.load("ui/objectives_panel.png")),
                ..default()
            },
            ObjectivesShowHideButton,
        ));
    });
}

fn on_objective_created_system(
    mut commands: Commands,
    mut events: EventReader<BuilderObjective>,
    objectives_panel: Query<Entity, With<ObjectivesPanel>>,
) {
    let objectives_panel = objectives_panel.single();
    for &BuilderObjective { entity, .. } in events.read() {
        commands.entity(objectives_panel).add_child(entity);
    }
}

fn panel_transition_to_visible_system(
    time: Res<Time>,
    mut next_state: ResMut<NextState<ObjectivesPanelState>>,
    mut objectives_panel: Query<&mut Style, With<ObjectivesPanel>>,
) {
    let mut style = objectives_panel.single_mut();
    let current_top = match style.top {
        Val::Px(top) => top,
        _ => unreachable!(),
    };
    let new_top = current_top + time.delta_seconds() * SLIDING_SPEED;
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
    mut objectives_panel: Query<(&Node, &mut Style), With<ObjectivesPanel>>,
) {
    let (node, mut style) = objectives_panel.single_mut();
    let current_top = match style.top {
        Val::Px(top) => top,
        _ => unreachable!(),
    };
    let new_top = current_top - time.delta_seconds() * SLIDING_SPEED;
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
    let Ok(interaction) = objectives_show_hide_button.get_single() else { return; };
    if matches!(*interaction, Interaction::Pressed) {
        match current_state.get() {
            ObjectivesPanelState::Hidden => next_state.set(ObjectivesPanelState::TransitionToVisible),
            ObjectivesPanelState::Visible => next_state.set(ObjectivesPanelState::TransitionToHidden),
            _ => {}
        }
    }
}