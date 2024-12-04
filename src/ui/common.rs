use bevy::color::palettes::css::{BLUE, GREEN, RED};
use bevy::text::LineBreak;

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
                on_cost_indicator_changed_system,
                calculate_cost_indicator_has_required_resources_system.run_if(resource_changed::<Stock>),
            ));
        app.world_mut().add_observer(on_healthbar_added_trigger);
        app.world_mut().add_observer(on_cost_indicator_added_trigger);
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
    pub node: Node,
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
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor::from(Color::linear_rgba(0., 0., 0., 0.)),
            BorderColor::from(Color::linear_rgba(0., 0.2, 1., 1.)),
        )).with_children(|parent| {
            // Top rectangle(health)
            healthbar_children.value_rectangle = parent.spawn((
                Node {
                    width: Val::Percent(healthbar.get_percent()),
                    height: Val::Percent(100.),
                    ..default()
                },
                BackgroundColor::from(healthbar.color),
                HealthbarValueRectangle,
            )).id();
            // Current hp text
            parent.spawn((
                // This additional container is needed to center the text as no combination of flex_direction, justify_content and align_items work
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
            )).with_children(|parent| {
                    healthbar_children.value_text = parent.spawn((
                        Text::new(format!("{:.0} / {:.0}", healthbar.value, healthbar.max_value)),
                        TextFont::default().with_font_size(healthbar.font_size),
                        TextColor::BLACK,
                        TextLayout::new_with_linebreak(LineBreak::NoWrap),
                        Node {
                            top: Val::Px(-2.0), // Looks more centered
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
    mut value_rectangles: Query<(&mut Node, &mut BackgroundColor), With<HealthbarValueRectangle>>,
    mut texts: Query<&mut Text, With<HealthbarValueText>>,
) {
    for (healthbar, children) in healthbars.iter() {
        let Ok((mut style, mut background_color)) = value_rectangles.get_mut(children.value_rectangle) else { unreachable!() };
        style.width = Val::Percent(healthbar.get_percent());
        background_color.0 = healthbar.color;
        let Ok(mut text) = texts.get_mut(children.value_text) else { unreachable!() };
        text.0 = format!("{} / {}", healthbar.value, healthbar.max_value);
    }
}

////////////////////////////////////////////
////          Cost Indicator            ////
////////////////////////////////////////////
#[derive(Bundle, Default)]
pub struct CostIndicatorBundle {
    pub node: Node,
    pub cost_indicator: CostIndicator,
}
#[derive(Component)]
pub struct CostIndicator {
    pub cost: Cost,
    pub has_required_resources: bool,
    pub font_size: f32,
    pub font_color: Color,
}
impl Default for CostIndicator {
    fn default() -> Self {
        Self {
            cost: Cost {
                resource_type: ResourceType::DarkOre,
                amount: 0,
            },
            has_required_resources: false,
            font_size: 14.,
            font_color: Color::WHITE,
        }
    }
}
#[derive(Component)]
struct CostIndicatorChildren {
    border_rectangle: Entity,
    icon: Entity,
    value_text: Entity,
}
#[derive(Component)]
struct CostIndicatorIcon;
#[derive(Component)]
struct CostIndicatorValueText;
#[derive(Component)]
struct CostIndicatorBorderRectangle;

fn on_cost_indicator_added_trigger(
    trigger: Trigger<OnAdd, CostIndicator>,
    mut commands: Commands,
    stock: Res<Stock>,
    mut cost_indicators: Query<&mut CostIndicator>,
) {
    let cost_indicator_entity = trigger.entity();
    let Ok(mut cost_indicator) = cost_indicators.get_mut(cost_indicator_entity) else { return; };
    // Update the cost indicator state
    cost_indicator.has_required_resources = stock.can_cover(&cost_indicator.cost);
    // Spawn the full cost indicator structure
    let mut cost_indicator_children = CostIndicatorChildren {
        border_rectangle: Entity::PLACEHOLDER,
        icon: Entity::PLACEHOLDER,
        value_text: Entity::PLACEHOLDER,
    };
    commands.entity(cost_indicator_entity).with_children(|parent| {
        cost_indicator_children.border_rectangle = parent.spawn((
            Node {
                width: Val::Px(32.),
                height: Val::Px(32.),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor::from(Color::linear_rgba(0., 0., 0., 0.)),
            BorderColor::from(BLUE),
            CostIndicatorBorderRectangle,
        )).id();
        cost_indicator_children.icon = parent.spawn((
            ImageNode::default(),
            Node {
                width: Val::Px(16.),
                height: Val::Px(16.),
                ..default()
            },
            CostIndicatorIcon,
        )).id();
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(2.0),
                left: Val::Px(2.0),
                ..default()
            },
        )).with_children(|parent| {
            cost_indicator_children.value_text = parent.spawn((
                Text::new(format!("{}", cost_indicator.cost.amount)),
                TextFont::default().with_font_size(cost_indicator.font_size),
                TextColor::from(cost_indicator.font_color),
                TextLayout::new_with_linebreak(LineBreak::NoWrap),
                CostIndicatorValueText,
            )).id();
        });
    }).insert(cost_indicator_children);
}

fn on_cost_indicator_changed_system(
    cost_indicators: Query<(&CostIndicator, &CostIndicatorChildren), Changed<CostIndicator>>,
    mut texts: Query<&mut Text, With<CostIndicatorValueText>>,
    mut border_rectangles: Query<&mut BorderColor, With<CostIndicatorBorderRectangle>>,
) {
    for (cost_indicator, children) in cost_indicators.iter() {
        let Ok(mut text) = texts.get_mut(children.value_text) else { unreachable!() };
        text.0 = format!("{}", cost_indicator.cost.amount);

        let Ok(mut border_color) = border_rectangles.get_mut(children.border_rectangle) else { unreachable!() };
        border_color.0 = if cost_indicator.has_required_resources{ GREEN.into() } else { RED.into() };
    }
}

fn calculate_cost_indicator_has_required_resources_system(
    stock: Res<Stock>,
    mut cost_indicators: Query<&mut CostIndicator>,
) {
    for mut cost_indicator in cost_indicators.iter_mut() {
        cost_indicator.has_required_resources = stock.can_cover(&cost_indicator.cost);
    }
}


////////////////////////////////////////////
////               Utils                ////
////////////////////////////////////////////
pub fn recolor_background_on<E>(color: Color) -> impl Fn(Trigger<E>, Query<&mut BackgroundColor>) {
    move |event, mut background_colors| {
        let Ok(mut background_color) = background_colors.get_mut(event.entity()) else {
            return;
        };
        background_color.0 = color;
    }
}
