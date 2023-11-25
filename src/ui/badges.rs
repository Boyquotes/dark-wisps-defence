use bevy::prelude::*;


pub fn create_dark_ore_badge(
    commands: &mut Commands,
    asset_server: &AssetServer,
) -> Entity {
    let dare_ore_badge = commands.spawn((
        NodeBundle {
            style: Style {
                // Position in the top left
                position_type: PositionType::Absolute,
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                width: Val::Px(101.),
                height: Val::Px(115.),
                ..Default::default()
            },
            background_color: Color::WHITE.into(),
            ..Default::default()
        },
        UiImage::new(asset_server.load("ui/dark_ore_badge.png"))
    )).with_children(|parent| {
        parent.spawn((
            TextBundle {
                text: Text::from_section("####", TextStyle::default()),
                style: Style {
                    top: Val::Px(83.),
                    left: Val::Px(39.),
                    ..Default::default()
                },
                ..Default::default()
            },
            Label,
        ));
    }).id();
    dare_ore_badge
}

pub fn initialize_badges_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    create_dark_ore_badge(&mut commands, &asset_server);
}