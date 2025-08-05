use core::f64;
use double_pendulum::*;
use nannou::color::rgb::Rgb;
use nannou::image;
use nannou::prelude::*;

const INITIAL_ANGLE1: f64 = f64::consts::PI / 3.0;
const INITIAL_ANGLE2: f64 = f64::consts::PI / 2.0;
const L1: f64 = 1.0;
const L2: f64 = 1.0;
const M1: f64 = 1.0;
const M2: f64 = 1.0;
const ANGLE_MUTATION: f64 = 0.000001;
const GRAVITY: f64 = 0.001;
const UPDATES_PER_ITERATION: usize = 1;
const WIDTH: usize = 700;
const HEIGHT: usize = 700;

fn main() {
    nannou::app(model)
        .size(WIDTH as u32, HEIGHT as u32)
        .update(update)
        .simple_window(view)
        .run();
}

struct Model {
    pub sample: Vec<DoublePendulum>,
    pub image: image::DynamicImage,
    pub update_row: usize,
}

fn model(_app: &App) -> Model {
    let mut image = image::DynamicImage::new_rgb8(WIDTH as u32, HEIGHT as u32);

    let mut sample = Vec::with_capacity(WIDTH);
    for i in 0..WIDTH {
        sample.push(
            DoublePendulum::new(L1, L2, M1, M2)
                .with_angle1(INITIAL_ANGLE1 + ANGLE_MUTATION * i as f64)
                .with_angle2(INITIAL_ANGLE2 - ANGLE_MUTATION * i as f64),
        );
        let image::DynamicImage::ImageRgb8(image) = &mut image else {
            panic!("Expected image to be of type ImageRgb8");
        };
        image.put_pixel(i as u32, 0, pendulum_to_color(&sample[i]));
    }

    for j in 1..HEIGHT {
        for i in 0..WIDTH {
            for _ in 0..UPDATES_PER_ITERATION {
                sample[i].update(GRAVITY);
            }
            let image::DynamicImage::ImageRgb8(image) = &mut image else {
                panic!("Expected image to be of type ImageRgb8");
            };
            image.put_pixel(i as u32, j as u32, pendulum_to_color(&sample[i]));
        }
    }

    Model {
        sample,
        image,
        update_row: 0,
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    for (i, pendulum) in model.sample.iter_mut().enumerate() {
        for _ in 0..UPDATES_PER_ITERATION {
            pendulum.update(GRAVITY);
        }

        let image::DynamicImage::ImageRgb8(image) = &mut model.image else {
            panic!("Expected image to be of type ImageRgb8");
        };
        image.put_pixel(
            i as u32,
            model.update_row as u32,
            pendulum_to_color(pendulum),
        );
    }

    model.update_row += 1;
    if model.update_row >= HEIGHT {
        model.update_row = 0;
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let win = app.window_rect();

    let texture = wgpu::Texture::from_image(app, &model.image);

    // Draw texture 2 times with offsets to simulate infinite scrolling
    draw.texture(&texture)
        .wh(win.wh())
        .y(model.update_row as f32 - HEIGHT as f32 / 2.0 - win.h() * 0.5);
    draw.texture(&texture)
        .wh(win.wh())
        .y(model.update_row as f32 - HEIGHT as f32 / 2.0 + HEIGHT as f32 - win.h() * 0.5);

    draw.to_frame(app, &frame).unwrap();
}

fn pendulum_to_color(double_pendulum: &DoublePendulum) -> image::Rgb<u8> {
    let rgb: Rgb = Hsv::new(
        (normalize_angle(double_pendulum.angle1) * 360.0) as f32,
        ((double_pendulum.angle2.sin() + 1.0) * 0.5) as f32,
        1.0,
    )
    .into();

    image::Rgb([
        (rgb.red * 255.0) as u8,
        (rgb.green * 255.0) as u8,
        (rgb.blue * 255.0) as u8,
    ])
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
