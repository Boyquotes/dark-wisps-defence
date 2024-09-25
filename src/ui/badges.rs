use crate::prelude::*;

#[derive(Component)]
pub struct MarkerDarkOreBadgeText;


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
            ..Default::default()
        },
        UiImage::new(asset_server.load("ui/dark_ore_badge.png")),
    )).with_children(|parent| {
        parent.spawn((
            TextBundle {
                text: Text::from_section("####", TextStyle { font_size: 12., ..default() }),
                style: Style {
                    top: Val::Px(83.),
                    left: Val::Px(39.),
                    ..Default::default()
                },
                ..Default::default()
            },
            Label,
            MarkerDarkOreBadgeText,
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

pub fn sync_dark_ore_badge_system(
    mut dark_ore_text: Query<&mut Text, With<MarkerDarkOreBadgeText>>,
    dark_ore_stock: Res<Stock>,
) {
    let mut text = dark_ore_text.single_mut();
    text.sections[0].value = dark_ore_stock.get(ResourceType::DarkOre).to_string();
}