use bevy::prelude::*;
use bevy::ui::FocusPolicy;

const NOT_HOVERED_ALPHA: f32 = 0.2;

#[derive(Component, Default)]
pub struct ConstructMenuButton {
    pub is_hovered: bool,
}
#[derive(Component, Default)]
pub struct ConstructMenuListPicker {
    pub is_hovered: bool,
}

pub fn create_construct_menu(
    commands: &mut Commands,
    asset_server: &AssetServer,
) -> Entity {
    let construct_towers_button = commands.spawn((NodeBundle {
        // Main Menu node
        style: Style {
            position_type: PositionType::Absolute,
            top: Val::Percent(40.),
            left: Val::Px(5.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..Default::default()
        },
        ..Default::default()
    })).with_children(|parent| {
        // Construct towers button
        parent.spawn((ButtonBundle {
                style: Style {
                    width: Val::Px(65.),
                    height: Val::Px(64.),
                    ..Default::default()
                },
                background_color: (*Color::WHITE.set_a(NOT_HOVERED_ALPHA)).into(),
                image: UiImage::new(asset_server.load("ui/construct_towers.png")),
                ..Default::default()
            },
            ConstructMenuButton::default(),
        )).with_children(|parent| {
            // Construct towers list picker
            parent.spawn((ButtonBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        left: Val::Px(65.),
                        ..Default::default()
                    },
                    background_color: Color::BLACK.into(),
                    z_index: ZIndex::Global(-1),
                    ..Default::default()
                },
                ConstructMenuListPicker::default(),
            )).with_children(|parent| {
                // Specific tower to construct
                parent.spawn((ButtonBundle {
                    style: Style {
                        width: Val::Px(48.),
                        height: Val::Px(48.),
                        ..Default::default()
                    },
                    focus_policy: FocusPolicy::Pass,
                    ..Default::default()
                }));
            });
        });
        parent.spawn((ButtonBundle {
            // Construct buildings button
                style: Style {
                    width: Val::Px(65.),
                    height: Val::Px(64.),
                    ..Default::default()
                },
                background_color: (*Color::WHITE.set_a(NOT_HOVERED_ALPHA)).into(),
                image: UiImage::new(asset_server.load("ui/construct_buildings.png")),
                ..Default::default()
            },
            ConstructMenuButton::default(),
        ));
    }).id();
    construct_towers_button
}

pub fn initialize_construction_menu_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    create_construct_menu(&mut commands, &asset_server);
}

pub fn menu_hover_system(
    mut menu_buttons: Query<(&Interaction, &mut ConstructMenuButton), Changed<Interaction>>,
    mut list_pickers: Query<(&Interaction, &mut ConstructMenuListPicker), Changed<Interaction>>,
) {
    for (interaction, mut menu_button) in menu_buttons.iter_mut() {
        menu_button.is_hovered = !matches!(interaction, Interaction::None);
    }
    for (interaction, mut list_picker) in list_pickers.iter_mut() {
        list_picker.is_hovered = !matches!(interaction, Interaction::None);
    }
}

pub fn menu_activation_system(
    mut menu_buttons: Query<(&ConstructMenuButton, &mut BackgroundColor, &Children)>,
    mut list_pickers: Query<(&ConstructMenuListPicker, &mut Visibility)>,
) {
    for (menu_button, mut background, children) in menu_buttons.iter_mut() {
        let list_picker_entity = children.get(0).unwrap();
        let (list_picker, mut list_picker_visibility) = list_pickers.get_mut(*list_picker_entity).unwrap();
        if menu_button.is_hovered || list_picker.is_hovered {
            background.0.set_a(1.);
            *list_picker_visibility = Visibility::Inherited;
        } else {
            background.0.set_a(NOT_HOVERED_ALPHA);
            *list_picker_visibility = Visibility::Hidden;
        }
    }
}