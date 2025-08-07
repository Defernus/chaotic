use chaotic::*;
use core::f64;
use nannou::image;
use nannou::prelude::*;

const G: f64 = 1.1; // Gravitational constant
const DT: f64 = 0.31; // Time step for simulation

const UPDATES_PER_ITERATION: usize = 1;
const MUTATION: f64 = 0.0001;
const SAMPLE_SIZE: usize = 1024;
const HEIGHT: usize = 512;

fn main() {
    nannou::app(model)
        .size(SAMPLE_SIZE as u32, HEIGHT as u32)
        .update(update)
        .event(event)
        .simple_window(view)
        .run();
}

struct Model<System> {
    pub samples: Samples<System>,
    pub display_sample: usize,
    pub image: image::DynamicImage,
    pub update_row: usize,
    pub is_paused: bool,
}

impl<System: ChaoticSystem + Clone> Model<System> {
    fn reset(&mut self) {
        let (samples, initial_system_index) =
            create_samples(self.samples.samples[0].clone(), &mut self.image);
        self.samples = samples;
        self.display_sample = initial_system_index;
    }
}

fn model(_app: &App) -> Model<ThreeBody> {
    let angle_a = 0.0;
    let angle_b = 1.0 / 3.0 * std::f64::consts::PI * 2.0;
    let angle_c = 2.0 / 3.0 * std::f64::consts::PI * 2.0;
    let mass = 0.1;
    let velocity = 0.31;

    let initial_system = ThreeBody::new(
        G,
        Body::new(
            mass,
            rotate_dvec2(DVec2::X, angle_a),
            rotate_dvec2(DVec2::Y, angle_a) * velocity,
        ),
        Body::new(
            mass,
            rotate_dvec2(DVec2::X, angle_b),
            rotate_dvec2(DVec2::Y, angle_b) * velocity,
        ),
        Body::new(
            mass,
            rotate_dvec2(DVec2::X, angle_c),
            rotate_dvec2(DVec2::Y, angle_c) * velocity,
        ),
    );
    let mut image = image::DynamicImage::new_rgb8(SAMPLE_SIZE as u32, HEIGHT as u32);

    let (samples, initial_system_index) = create_samples(initial_system, &mut image);

    println!("sample {:?}", samples.samples.last());

    Model {
        samples,
        display_sample: initial_system_index,
        image,
        update_row: 0,
        is_paused: true,
    }
}

fn create_samples<System: ChaoticSystem + Clone>(
    initial_system: System,
    image: &mut image::DynamicImage,
) -> (Samples<System>, usize) {
    let mut samples = Samples::new(initial_system, SAMPLE_SIZE, MUTATION);

    for j in 1..HEIGHT {
        samples.update(UPDATES_PER_ITERATION, DT);
        samples.draw_line(image, j);
    }

    (samples, 0)
}

fn update<System: ChaoticSystem + Clone>(_app: &App, model: &mut Model<System>, _update: Update) {
    if model.is_paused {
        return;
    }

    model.samples.update(UPDATES_PER_ITERATION, DT);
    model.samples.draw_line(&mut model.image, model.update_row);

    model.update_row += 1;
    if model.update_row >= HEIGHT {
        model.update_row = 0;
    }
}

fn event<System: ChaoticSystem + Clone>(app: &App, model: &mut Model<System>, event: Event) {
    match event {
        Event::WindowEvent {
            simple: Some(simple),
            ..
        } => match simple {
            WindowEvent::KeyPressed(Key::Space) => {
                println!("Toggling pause");
                model.is_paused = !model.is_paused;
            }
            WindowEvent::KeyPressed(Key::R) => {
                println!("Resetting simulation");
                model.reset();
            }
            WindowEvent::KeyPressed(Key::Left) => {
                if model.display_sample == 0 {
                    model.display_sample = SAMPLE_SIZE - 1;
                } else {
                    model.display_sample -= 1;
                }

                println!("Inspecting simulation at {}", model.display_sample);
            }
            WindowEvent::KeyPressed(Key::Right) => {
                model.display_sample += 1;
                if model.display_sample >= SAMPLE_SIZE {
                    model.display_sample = 0;
                }

                println!("Inspecting simulation at {}", model.display_sample);
            }
            WindowEvent::KeyPressed(Key::F) => {
                let most_stable_system_index = model
                    .samples
                    .samples
                    .iter()
                    .map(ChaoticSystem::chaosity)
                    .enumerate()
                    .min_by(|a, b| a.1.total_cmp(&b.1))
                    .map(|(index, _)| index)
                    .unwrap_or(model.display_sample);

                model.display_sample = most_stable_system_index;

                println!(
                    "Inspecting most stable simulation at {}",
                    model.display_sample
                );
            }
            WindowEvent::MousePressed(MouseButton::Left) => {
                let win = app.window_rect();
                let width = win.w();

                model.display_sample =
                    map_f32_to_index(app.mouse.x + width * 0.5, SAMPLE_SIZE, width);

                println!("Inspecting simulation at {}", model.display_sample);
            }
            _ => {}
        },
        _ => {}
    }
}

fn view(app: &App, model: &Model<ThreeBody>, frame: Frame) {
    let draw = app.draw();
    let win = app.window_rect();

    let texture = wgpu::Texture::from_image(app, &model.image);

    draw.background().color(WHITE);

    // Draw texture 2 times with offsets to simulate infinite scrolling

    draw.texture(&texture)
        .wh(win.wh())
        .y(model.update_row as f32 - texture.height() as f32);
    draw.texture(&texture)
        .wh(win.wh())
        .y(model.update_row as f32);

    let scale = 10.0;
    for body in model.samples.samples[model.display_sample].iter() {
        let pos = body.position * scale;
        draw.ellipse()
            .x_y(pos.x as f32, pos.y as f32)
            .radius(5.0)
            .color(BLUE);
    }

    draw.to_frame(app, &frame).unwrap();
}
