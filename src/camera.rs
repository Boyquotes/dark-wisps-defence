use bevy::core_pipeline::bloom::BloomSettings;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

const ZOOM_MIN: f32 = 1.;
const ZOOM_MAX: f32 = 4.;
const ZOOM_SPEED: f32 = 20.;
const SLIDE_SPEED: f32 = 500.;

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
        app.add_systems(Update, (camera_zoom, camera_movement));
    }
}

#[derive(Component)]
pub struct MainCamera;

fn startup(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            transform: Transform::from_xyz(500., 500., 0.),
            ..Default::default()
        },
        BloomSettings::default(),
    )).insert(MainCamera);
}

fn camera_zoom(
    mut query: Query<&mut OrthographicProjection, With<MainCamera>>,
    time: Res<Time>,
    mut mouse_wheel_events: EventReader<MouseWheel>
) {
    let mut scroll = 0.0;
    for event in mouse_wheel_events.iter() {
        scroll += event.y;
    }

    for mut projection in query.iter_mut() {
        let mut log_scale = projection.scale.ln();
        log_scale -= scroll * ZOOM_SPEED * time.delta_seconds();
        projection.scale = log_scale.exp().clamp(ZOOM_MIN, ZOOM_MAX);
    }
}


fn camera_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut query_camera: Query<&mut Transform, With<MainCamera>>,
    time: Res<Time>
) {
    let mut translation = Vec3::ZERO;

    // 'i' moves the camera up
    if keyboard_input.pressed(KeyCode::I) {
        translation.y += 1.0;
    }

    // 'k' moves the camera down
    if keyboard_input.pressed(KeyCode::K) {
        translation.y -= 1.0;
    }

    // 'j' moves the camera to the left
    if keyboard_input.pressed(KeyCode::J) {
        translation.x -= 1.0;
    }

    // 'l' moves the camera to the right
    if keyboard_input.pressed(KeyCode::L) {
        translation.x += 1.0;
    }

    // Apply the camera movement
    for mut transform in query_camera.iter_mut() {
        transform.translation += SLIDE_SPEED * time.delta_seconds() * translation;
    }
}