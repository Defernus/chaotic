use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::EguiContexts;

const MAX_ZOOM_IN: f32 = 0.5;
const MAX_ZOOM_OUT: f32 = 6.0;
const ZOOM_SCALE_SPEED: f32 = 0.003;

#[derive(Component, Default)]
pub struct MainCamera {
    pub cursor_position: Vec2,
    pub move_detection: u32,
    pub rotate_cursor_position: Vec2,
    pub rotate_detection: u32,
}

pub fn camera_zoom(
    mut wheel_input: EventReader<MouseWheel>,
    mut camera: Query<(&mut Projection, &mut Transform), With<MainCamera>>,
    window: Query<&Window, With<PrimaryWindow>>,
) -> Result<(), BevyError> {
    let Some(mouse_event) = wheel_input.read().last() else {
        return Ok(());
    };

    let (mut camera_projection, mut transform) = camera.single_mut()?;

    let Projection::Orthographic(ref mut camera_projection) = *camera_projection else {
        error!("Expected orthographic projection");
        return Ok(());
    };

    let scroll = -mouse_event.y * ZOOM_SCALE_SPEED;
    if scroll == 0.0 {
        return Ok(());
    }

    let scroll = scroll * camera_projection.scale;

    let prev_scale = camera_projection.scale;
    camera_projection.scale += scroll;
    camera_projection.scale = camera_projection.scale.clamp(MAX_ZOOM_IN, MAX_ZOOM_OUT);
    if camera_projection.scale == prev_scale {
        return Ok(());
    }
    let scroll = camera_projection.scale - prev_scale;

    let window = window.single()?;

    // if cursor position is None for some reason, scale from the center of the screen
    let screen_size = vec2(window.resolution.width(), window.resolution.height());
    let mouse_position = window
        .cursor_position()
        .unwrap_or_else(|| screen_size / 2.0);

    let x_dir = transform.right();
    let y_dir = -Vec3::Z * 2.0f32.sqrt();

    transform.translation -= x_dir * (mouse_position.x - screen_size.x / 2.0) * scroll;
    transform.translation -= y_dir * (mouse_position.y - screen_size.y / 2.0) * scroll;

    Ok(())
}

pub fn camera_move_by_mouse(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut camera: Query<(&mut Transform, &mut MainCamera, &Projection), With<MainCamera>>,
    mut contexts: EguiContexts,
) -> Result<(), BevyError> {
    if contexts.ctx_mut()?.is_pointer_over_area() {
        return Ok(());
    }

    if mouse_button_input.pressed(MouseButton::Left) {
        let (mut transform, mut cam, projection) = camera.single_mut()?;
        let Projection::Orthographic(ref projection) = *projection else {
            error!("Expected orthographic projection");
            return Ok(());
        };

        let x_dir = transform.right();
        let y_dir = -Vec3::Z * 2.0f32.sqrt();

        if cam.move_detection >= 2 {
            for event in cursor_moved_events.read() {
                if cam.cursor_position.x == 0.0 {
                    cam.cursor_position.x = event.position.x;
                    cam.cursor_position.y = event.position.y;
                }
                let dif_x = cam.cursor_position.x - event.position.x;
                let dif_y = cam.cursor_position.y - event.position.y;
                transform.translation += x_dir * dif_x * projection.scale;
                transform.translation += y_dir * dif_y * projection.scale;

                cam.cursor_position.x = event.position.x;
                cam.cursor_position.y = event.position.y;
            }
        } else {
            cam.move_detection += 1;
        }
    }

    if mouse_button_input.just_released(MouseButton::Left) {
        for (_, mut cam, _) in camera.iter_mut() {
            cam.move_detection = 0;
            cam.cursor_position.x = 0.0;
            cam.cursor_position.y = 0.0;
        }
    }

    Ok(())
}

const ROTATE_SPEED: f32 = 0.005; // radians per pixel

/// Compute the closest point on the world Z axis (x=0,y=0 line) to the camera's forward ray.
fn closest_point_on_z_axis_to_camera_ray(origin: Vec3, dir: Vec3) -> Vec3 {
    let u = dir.normalize_or_zero();
    let v = Vec3::Z;
    let w0 = origin;

    let a = u.dot(u);
    let b = u.dot(v);
    let c = v.dot(v);
    let d = u.dot(w0);
    let e = v.dot(w0);

    let denom = a * c - b * b;
    if denom.abs() < 1e-8 {
        return Vec3::new(0.0, 0.0, origin.z);
    }
    let t = (a * e - b * d) / denom;
    Vec3::new(0.0, 0.0, t)
}

pub fn rotate_camera(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut camera: Query<(&mut Transform, &mut MainCamera), With<MainCamera>>,
    mut contexts: EguiContexts,
) -> Result<(), BevyError> {
    if contexts.ctx_mut()?.is_pointer_over_area() {
        return Ok(());
    }

    if mouse_button_input.pressed(MouseButton::Right) {
        let (mut transform, mut cam) = camera.single_mut()?;

        let forward: Vec3 = transform.forward().into();
        let pivot = closest_point_on_z_axis_to_camera_ray(transform.translation, forward);

        if cam.rotate_detection >= 2 {
            for event in cursor_moved_events.read() {
                if cam.rotate_cursor_position.x == 0.0 {
                    cam.rotate_cursor_position = event.position;
                }
                let dif_x = event.position.x - cam.rotate_cursor_position.x;
                let angle = dif_x * ROTATE_SPEED;
                if angle.abs() > 0.0 {
                    let rot = Quat::from_axis_angle(Vec3::Z, angle);
                    transform.rotate_around(pivot, rot);
                }
                cam.rotate_cursor_position = event.position;
            }
        } else {
            cam.rotate_detection += 1;
        }
    }

    if mouse_button_input.just_released(MouseButton::Right) {
        for (_, mut cam) in camera.iter_mut() {
            cam.rotate_detection = 0;
            cam.rotate_cursor_position = Vec2::ZERO;
        }
    }

    Ok(())
}
