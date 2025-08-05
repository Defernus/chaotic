use core::f64;
use double_pendulum::*;
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).simple_window(view).run();
}

const INITIAL_ANGLE1: f64 = f64::consts::PI / 3.0;
const INITIAL_ANGLE2: f64 = f64::consts::PI / 2.0;
const L1: f64 = 1.0;
const L2: f64 = 1.0;
const M1: f64 = 1.0;
const M2: f64 = 1.0;
const ANGLE_MUTATION: f64 = 0.01;
const GRAVITY: f64 = 0.001;
const MOUSE_SENSITIVITY: f64 = 0.01;
const UPDATES_PER_ITERATION: usize = 1;
const SAMPLE_SIZE: usize = 256;
const ITERATIONS: usize = 256;

struct Model {
    pub states: Vec<Vec<DoublePendulum>>,
    pub prev_mouse: Option<Point2>,
}

fn model(_app: &App) -> Model {
    let mut states = Vec::with_capacity(ITERATIONS);

    let mut sample = Vec::with_capacity(SAMPLE_SIZE);

    for i in 0..SAMPLE_SIZE {
        sample.push(
            DoublePendulum::new(L1, L2, M1, M2)
                .with_angle1(INITIAL_ANGLE1 + ANGLE_MUTATION * i as f64)
                .with_angle2(INITIAL_ANGLE2 - ANGLE_MUTATION * i as f64),
        )
    }
    states.push(sample.clone());

    Model {
        states,
        prev_mouse: None,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let mouse = app.mouse.position();
    let delta = model.prev_mouse.map(|prev| mouse - prev);

    let mut sample = model.states.last().cloned().unwrap();

    for pendulum in sample.iter_mut() {
        for _ in 0..UPDATES_PER_ITERATION {
            pendulum.update(GRAVITY);
            if let Some(delta) = delta {
                pendulum.angle1 += delta.x as f64 * MOUSE_SENSITIVITY;
                pendulum.angle2 += delta.y as f64 * MOUSE_SENSITIVITY;
            }
        }
    }

    model.states.push(sample.clone());
    if model.states.len() > ITERATIONS {
        model.states.remove(0);
    }

    model.prev_mouse = Some(mouse);
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    let win = app.window_rect();

    for (i, sample) in model.states.iter().enumerate() {
        for (j, double_pendulum) in sample.iter().enumerate() {
            let width = (1.0 / sample.len() as f32) * win.w();
            let height = (1.0 / model.states.len() as f32) * win.h();

            let x = j as f32 * width - win.w() / 2.0 + width / 2.0;
            let y = i as f32 * height - win.h() / 2.0 + height;

            draw_double_pendulum(&draw, double_pendulum, pt2(x, y), pt2(width, height));
        }
    }

    draw.to_frame(app, &frame).unwrap();
}

fn draw_double_pendulum(draw: &Draw, double_pendulum: &DoublePendulum, pos: Vec2, size: Vec2) {
    let color = Hsv::new(
        (normalize_angle(double_pendulum.angle1) * 360.0) as f32,
        ((double_pendulum.angle2.sin() + 1.0) * 0.5) as f32,
        1.0,
    );
    // Draw the second mass
    draw.rect().xy(pos).wh(size).color(color);
}

/// Convert angle to a normalized value between 0 and 1
fn normalize_angle(angle: f64) -> f64 {
    let normalized = angle % (2.0 * f64::consts::PI);
    (if normalized < 0.0 {
        normalized + 2.0 * f64::consts::PI
    } else {
        normalized
    }) / (2.0 * f64::consts::PI)
}
