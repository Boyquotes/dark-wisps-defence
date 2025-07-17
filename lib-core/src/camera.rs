use bevy::{core_pipeline::bloom::Bloom, input::mouse::MouseWheel};
use crate::lib_prelude::*;

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
        Camera2d::default(),
        Camera {
            hdr: true,
            ..default()
        },
        Transform::from_xyz(500., 500., 0.),
        Bloom {
            high_pass_frequency: 0.5,
            ..default()
        },
        MainCamera
    ));
}

fn camera_zoom(
    mut camera: Query<&mut Projection, With<MainCamera>>,
    time: Res<Time>,
    mut mouse_wheel_events: EventReader<MouseWheel>
) {
    let mut scroll = 0.0;
    for event in mouse_wheel_events.read() {
        scroll += event.y;
    }

    let Ok(mut projection) = camera.single_mut() else { return; };
    match &mut *projection {
        Projection::Orthographic(orthographic) => {
            let mut log_scale = orthographic.scale.ln();
            log_scale -= scroll * ZOOM_SPEED * time.delta_secs();
            orthographic.scale = log_scale.exp().clamp(ZOOM_MIN, ZOOM_MAX);
        }
        _ => panic!("Only orthographic projections are supported for zooming"),
    }
}


fn camera_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query_camera: Query<&mut Transform, With<MainCamera>>,
    time: Res<Time>
) {
    let mut translation = Vec3::ZERO;

    // 'i' moves the camera up
    if keyboard_input.pressed(KeyCode::KeyI) {
        translation.y += 1.0;
    }

    // 'k' moves the camera down
    if keyboard_input.pressed(KeyCode::KeyK) {
        translation.y -= 1.0;
    }

    // 'j' moves the camera to the left
    if keyboard_input.pressed(KeyCode::KeyJ) {
        translation.x -= 1.0;
    }

    // 'l' moves the camera to the right
    if keyboard_input.pressed(KeyCode::KeyL) {
        translation.x += 1.0;
    }

    // Apply the camera movement
    for mut transform in query_camera.iter_mut() {
        transform.translation += SLIDE_SPEED * time.delta_secs() * translation;
    }
}