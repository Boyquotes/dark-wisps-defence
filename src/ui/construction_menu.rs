use bevy::prelude::*;

pub fn create_construct_menu(
    commands: &mut Commands,
    asset_server: &AssetServer,
) -> Entity {
    let construct_towers_button = commands.spawn((NodeBundle {
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
        parent.spawn(ButtonBundle {
            style: Style {
                width: Val::Px(65.),
                height: Val::Px(64.),
                ..Default::default()
            },
            background_color: (*Color::WHITE.set_a(0.2)).into(),
            image: UiImage::new(asset_server.load("ui/construct_towers.png")),
            ..Default::default()
        });
        parent.spawn(ButtonBundle {
            style: Style {
                width: Val::Px(65.),
                height: Val::Px(64.),
                ..Default::default()
            },
            background_color: (*Color::WHITE.set_a(0.2)).into(),
            image: UiImage::new(asset_server.load("ui/construct_buildings.png")),
            ..Default::default()
        });
    }).id();
    construct_towers_button
}

pub fn create_construct_buildings_button(
    commands: &mut Commands,
    asset_server: &AssetServer,
) -> Entity {
    let construct_towers_button = commands.spawn((
        ButtonBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Percent(40.),
                left: Val::Px(5.0),
                width: Val::Px(65.),
                height: Val::Px(64.),
                ..Default::default()
            },
            background_color: (*Color::WHITE.set_a(0.2)).into(),
            image: UiImage::new(asset_server.load("ui/construct_buildings.png")),
            ..Default::default()
        },
    )).id();
    construct_towers_button
}


pub fn initialize_construction_menu_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    create_construct_menu(&mut commands, &asset_server);
}