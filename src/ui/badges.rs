use crate::prelude::*;

pub struct BadgesPlugin;
impl Plugin for BadgesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, initialize_badges_system)
            .add_systems(Update, sync_resource_display_system.run_if(resource_changed::<Stock>));
    }
}

fn sync_resource_display_system(
    mut resource_text: Query<(&mut Text, &ResourceBadgeText)>,
    stock: Res<Stock>,
) {
    for (mut text, badge) in resource_text.iter_mut() {
        text.0 = stock.get(badge.0).to_string();
    }
}


#[derive(Component)]
pub struct ResourceBadgeText(ResourceType);

#[derive(Component)]
pub struct EssencesContainerMarker;

fn initialize_badges_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(5.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
    )).with_children(|parent| {
        let _dare_ore_badge = parent.spawn((
            Node {
                width: Val::Px(101.),
                height: Val::Px(115.),
                ..default()
            },
            ImageNode::new(asset_server.load("ui/dark_ore_badge.png")),
        )).with_children(|parent| {
            parent.spawn((
                Text::new("####"),
                TextFont::default().with_font_size(12.),
                Node {
                    top: Val::Px(83.),
                    left: Val::Px(39.),
                    ..default()
                },
                Label,
                ResourceBadgeText(ResourceType::DarkOre),
            ));
        });
        let _essences_container = parent.spawn((
            Node {
                width: Val::Px(101.),
                //height: Val::Px(20.),
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(1.)),
                ..default()
            },
            BackgroundColor::from(Color::linear_rgba(0., 0.3, 0.7, 0.1)),
            BorderColor::from(Color::linear_rgba(0., 0.3, 0.9, 0.3)),
            BorderRadius::all(Val::Px(2.)),
            EssencesContainerMarker,
        )).with_children(|parent| {
            for essence_type in EssenceType::VARIANTS {
                parent.spawn((
                    Node {
                        width: Val::Percent(100.),
                        height: Val::Px(20.),
                        left: Val::Px(2.),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                )).with_children(|parent| {
                    let (background_color, border_color) = essence_type_to_badge_colors(essence_type);
                    parent.spawn((
                        Node {
                            width: Val::Px(10.),
                            height: Val::Px(10.),
                            ..default()
                        },
                        background_color,
                        border_color,
                        BorderRadius::all(Val::Px(25.)),
                    ));
                    parent.spawn((
                        Text::new("###"),
                        TextFont::default().with_font_size(12.),
                        Node {
                            left: Val::Px(2.),
                            ..default()
                        },
                        ResourceBadgeText(ResourceType::Essence(essence_type)),
                    ));
                });
            }
        });
    });
}

fn essence_type_to_badge_colors(essence_type: EssenceType) -> (BackgroundColor, BorderColor) {
    match essence_type {
        EssenceType::Fire => (BackgroundColor::from(Color::linear_rgba(1., 0.05, 0.05, 0.8)), BorderColor::from(Color::linear_rgba(1., 0., 0., 0.9))),
        EssenceType::Water => (BackgroundColor::from(Color::linear_rgba(0.05, 0.05, 1., 0.8)), BorderColor::from(Color::linear_rgba(0., 0., 1., 0.9))),
        EssenceType::Light => (BackgroundColor::from(Color::linear_rgba(0.95, 0.95, 0.95, 0.8)), BorderColor::from(Color::linear_rgba(1., 1., 1., 0.9))),
        EssenceType::Electric => (BackgroundColor::from(Color::linear_rgba(1., 0.95, 0.05, 0.8)), BorderColor::from(Color::linear_rgba(1., 1., 0., 0.9))),
    }
}