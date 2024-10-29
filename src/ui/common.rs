use std::backtrace;

use bevy::{color::palettes::css::{BLACK, GREEN}, text::BreakLineOn};

use crate::prelude::*;

pub struct UiCommonPlugin;
impl Plugin for UiCommonPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<UiInteraction>()
            .add_systems(PreUpdate, (
                keyboard_input_system,
                mouse_release_system,
            ))
            .add_systems(Update, (
                on_healthbar_changed_system,
            ))
            .world_mut().observe(on_healthbar_added_trigger);
    }
}

#[derive(Default, Clone, Debug, States, PartialEq, Eq, Hash)]
pub enum UiInteraction {
    #[default]
    Free, // No interaction
    PlaceGridObject,
    DisplayInfoPanel,
}

fn keyboard_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut ui_interaction_state: ResMut<NextState<UiInteraction>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        ui_interaction_state.set(UiInteraction::Free);
    }
}


#[derive(Component, Default)]
pub struct AdvancedInteraction {
    pub was_just_released: bool,
}

fn mouse_release_system(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut advanced_interaction: Query<(&Interaction, &mut AdvancedInteraction)>,
) {
    let was_mouse_just_released = mouse_button_input.just_released(MouseButton::Left);
    for (interaction, mut advanced_interaction) in advanced_interaction.iter_mut() {
        advanced_interaction.was_just_released = was_mouse_just_released && matches!(interaction, Interaction::Hovered);
    }
}

////////////////////////////////////////////
////          Healthbar Widget          ////
////////////////////////////////////////////

#[derive(Bundle, Default)]
pub struct HealthbarBundle {
    pub node: NodeBundle,
    pub healthbar: Healthbar,
}
#[derive(Component)]
pub struct Healthbar {
    pub value: f32,
    pub max_value: f32,
    pub font_size: f32,
    pub color: Color,
}
impl Healthbar {
    pub fn get_percent(&self) -> f32 {
        if self.max_value == 0. { 100. }
        else { self.value / self.max_value * 100. }
    }
}
impl Default for Healthbar {
    fn default() -> Self {
        Self { value: 0., max_value: 0., font_size: 16., color: GREEN.into() }
    }
}
#[derive(Component)]
struct HealthbarChildren {
    value_rectangle: Entity,
    value_text: Entity,
}
#[derive(Component)]
struct HealthbarValueText;
#[derive(Component)]
struct HealthbarValueRectangle;

fn on_healthbar_added_trigger(
    trigger: Trigger<OnAdd, Healthbar>,
    mut commands: Commands,
    healthbars: Query<&Healthbar>,
) {
    let healthbar_entity = trigger.entity();
    let Ok(healthbar) = healthbars.get(healthbar_entity) else { return; };
    let mut healthbar_children = HealthbarChildren {
        value_rectangle: Entity::PLACEHOLDER,
        value_text: Entity::PLACEHOLDER,
    };
    commands.entity(healthbar_entity).with_children(|parent| {
        parent.spawn((
            // Bottom rectangle(background)
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                background_color: Color::linear_rgba(0., 0., 0., 0.).into(),
                border_color: Color::linear_rgba(0., 0.2, 1., 1.).into(),
                ..default()
            },
        )).with_children(|parent| {
            // Top rectangle(health)
            healthbar_children.value_rectangle = parent.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(healthbar.get_percent()),
                        height: Val::Percent(100.),
                        ..default()
                    },
                    background_color: healthbar.color.into(),
                    ..default()
                },
                HealthbarValueRectangle,
            )).id();
            // Current hp text
            parent.spawn(NodeBundle {
                // This additional container is needed to center the text as no combination of flex_direction, justify_content and align_items work
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default() 
            }).with_children(|parent| {
                healthbar_children.value_text = parent.spawn((
                    TextBundle {
                        text: Text {
                            sections: vec![TextSection::new(format!("{:.0} / {:.0}", healthbar.value, healthbar.max_value), TextStyle{ color: BLACK.into(), font_size: healthbar.font_size, ..default() })],
                            linebreak_behavior: BreakLineOn::NoWrap,
                            ..default() 
                        },
                        style: Style {
                            top: Val::Px(-2.0), // Looks more centered
                            ..default()
                        },
                        ..default()
                    },
                    HealthbarValueText,
                )).id();
            });
        });
    }).insert(healthbar_children);
}

fn on_healthbar_changed_system(
    healthbars: Query<(&Healthbar, &HealthbarChildren), Changed<Healthbar>>,
    mut value_rectangles: Query<(&mut Style, &mut BackgroundColor), With<HealthbarValueRectangle>>,
    mut texts: Query<&mut Text, With<HealthbarValueText>>,
) {
    for (healthbar, children) in healthbars.iter() {
        let Ok((mut style, mut background_color)) = value_rectangles.get_mut(children.value_rectangle) else { unreachable!() };
        style.width = Val::Percent(healthbar.get_percent());
        background_color.0 = healthbar.color;
        let Ok(mut text) = texts.get_mut(children.value_text) else { unreachable!() };
        text.sections[0].value = format!("{} / {}", healthbar.value, healthbar.max_value);
    }
}