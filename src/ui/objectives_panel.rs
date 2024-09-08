use crate::{inventory::objectives::BuilderObjective, prelude::*};

pub struct ObjectivesPanelPlugin;
impl Plugin for ObjectivesPanelPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, initialize_objectives_panel_system)
            .add_systems(Update, (
                on_objective_created_system.run_if(on_event::<BuilderObjective>()),
            ));
    }
}

#[derive(Component)]
pub struct ObjectivesPanel;

fn initialize_objectives_panel_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Px(500.0),
                height: Val::Px(125.0),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                top: Val::Px(5.0),
                right: Val::Px(5.0),
                padding: UiRect::all(Val::Px(6.0)),
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
    ));
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