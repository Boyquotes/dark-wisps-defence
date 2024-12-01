use crate::prelude::*;

pub struct BadgesPlugin;
impl Plugin for BadgesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, initialize_badges_system)
            .add_systems(Update, sync_dark_ore_badge_system.run_if(resource_changed::<Stock>));
    }
}

pub fn sync_dark_ore_badge_system(
    mut dark_ore_text: Query<(&mut Text, &ResourceBadgeText)>,
    dark_ore_stock: Res<Stock>,
) {
    let (mut text, badge) = dark_ore_text.single_mut();
    text.0 = dark_ore_stock.get(badge.0).to_string();
}


#[derive(Component)]
pub struct ResourceBadgeText(ResourceType);

pub fn initialize_badges_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(5.0),
            ..Default::default()
        },
    )).with_children(|parent| {
        let dare_ore_badge = parent.spawn((
            Node {
                width: Val::Px(101.),
                height: Val::Px(115.),
                ..Default::default()
            },
            ImageNode::new(asset_server.load("ui/dark_ore_badge.png")),
        )).with_children(|parent| {
            parent.spawn((
                Text::new("####"),
                TextFont::default().with_font_size(12.),
                Node {
                    top: Val::Px(83.),
                    left: Val::Px(39.),
                    ..Default::default()
                },
                Label,
                ResourceBadgeText(ResourceType::DarkOre),
            ));
        });
    });
}
