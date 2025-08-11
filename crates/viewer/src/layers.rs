use crate::MainCamera;
use bevy::asset::RenderAssetUsages;
use bevy::math::DVec2;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use chaotic::{
    Body,
    ChaoticSystem,
    Dimensions,
    Mandelbrot,
    MandelbrotColorSchema,
    NBody,
    NBodyColorSchema,
    Samples,
};
use std::time::{Duration, Instant};

// Constants taken from the original Chaos main
const G: f64 = 1.0; // Gravitational constant

#[derive(Resource)]
pub struct InitData<T> {
    pub mutation_scale: Vec<f64>,
    pub all_scale: f64,
    pub initial_mutation: Vec<f64>,
    pub dimensions: Dimensions,

    pub initial_sample: T,
    pub dt: f64,
    pub updates_per_iteration: usize,
}

impl<T: ChaoticSystem + Clone> InitData<T> {
    pub fn init(&self) -> ViewerState<T> {
        let mut initial_sample = self.initial_sample.clone();
        initial_sample.mutate(&self.initial_mutation);
        let samples = Samples::new(
            initial_sample,
            self.dimensions.clone(),
            &self.mutation_scale,
            self.all_scale,
        );

        ViewerState {
            initial_mutation: self.initial_mutation.clone(),
            mutation_scale: self.mutation_scale.clone(),
            all_scale: self.all_scale,
            dt: self.dt,
            updates_per_iteration: self.updates_per_iteration,
            samples,
        }
    }
}

impl Default for InitData<NBody> {
    fn default() -> Self {
        // Build initial ThreeBody system (matching the original Chaos main)
        let angle_a = 0.0;
        let angle_b = std::f64::consts::PI * (1.0 / 3.0) * 2.0;
        let angle_c = std::f64::consts::PI * (2.0 / 3.0) * 2.0;
        let mass = 0.1;
        let velocity = 0.31;

        let initial_sample = NBody::new(
            G,
            vec![
                Body::new(
                    mass,
                    rotate(DVec2::X, angle_a),
                    rotate(DVec2::Y, angle_a) * velocity,
                ),
                Body::new(
                    mass,
                    rotate(DVec2::X, angle_b),
                    rotate(DVec2::Y, angle_b) * velocity,
                ),
                Body::new(
                    mass,
                    rotate(DVec2::X, angle_c),
                    rotate(DVec2::Y, angle_c) * velocity,
                ),
            ],
            NBodyColorSchema::VelocityToRgb { v0: 1.0 },
        );

        Self {
            dt: 0.33,
            updates_per_iteration: 1,
            initial_sample,
            mutation_scale: vec![1.0, 1.0],
            all_scale: 0.01,
            initial_mutation: vec![0.0, 0.0],
            dimensions: Dimensions::new_static(&[256, 256]),
        }
    }
}

impl Default for InitData<Mandelbrot> {
    fn default() -> Self {
        Self {
            dt: 0.01,
            updates_per_iteration: 1,
            initial_sample: Mandelbrot::new(MandelbrotColorSchema::Distance),
            mutation_scale: vec![1.0, 1.0],
            all_scale: 0.01,
            initial_mutation: vec![0.0, 0.0],
            dimensions: Dimensions::new_static(&[256, 256]),
        }
    }
}

#[derive(Resource)]
pub struct LayerData {
    pub target_depth: usize,
    pub current_depth: usize,

    pub request_update: bool,
}

impl Default for LayerData {
    fn default() -> Self {
        Self {
            target_depth: 256,
            current_depth: 0,
            request_update: false,
        }
    }
}

#[derive(Resource)]
pub struct ViewerState<T> {
    pub initial_mutation: Vec<f64>,
    pub mutation_scale: Vec<f64>,
    pub all_scale: f64,
    pub dt: f64,
    pub updates_per_iteration: usize,
    pub samples: Samples<T>,
}

#[derive(Component)]
pub struct Layer;

pub fn reset_layers_sys<T: ChaoticSystem + Clone>(
    mut commands: Commands,
    mut state: ResMut<ViewerState<T>>,
    init_data: Res<InitData<T>>,
    mut layer_data: ResMut<LayerData>,
    layers_q: Query<Entity, With<Layer>>,
    mut camera_q: Query<&mut Transform, With<MainCamera>>,
) -> Result<(), BevyError> {
    if layer_data.request_update {
        for layer in layers_q.iter() {
            commands.entity(layer).despawn();
        }

        *state = init_data.init();

        let mut camera_transform = camera_q.single_mut()?;
        camera_transform.translation.z -= layer_data.current_depth as f32;

        layer_data.current_depth = 0;
        layer_data.request_update = false;
    }

    Ok(())
}

pub fn process_layers_sys<T: ChaoticSystem>(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut state: ResMut<ViewerState<T>>,
    mut layer_data: ResMut<LayerData>,
    mut camera_q: Query<&mut Transform, With<MainCamera>>,
) -> Result<(), BevyError> {
    if layer_data.current_depth < layer_data.target_depth {
        let dt = state.dt;
        let updates_per_iteration = state.updates_per_iteration;
        let start_time = Instant::now();
        let mut current_time = start_time;

        while current_time - start_time < Duration::from_millis(10) {
            let mut camera_transform = camera_q.single_mut()?;
            camera_transform.translation.z += 1.0;
            state.samples.update(updates_per_iteration, dt);
            let new_layer = build_image(&state.samples, &mut images);

            commands.spawn((
                Layer,
                Sprite::from_image(new_layer.clone()),
                Transform::from_xyz(0.0, 0.0, layer_data.current_depth as f32),
            ));

            layer_data.current_depth += 1;
            if layer_data.current_depth >= layer_data.target_depth {
                break;
            }

            current_time = Instant::now();
        }
    }

    Ok(())
}

fn build_image<T: ChaoticSystem>(
    samples: &Samples<T>,
    images: &mut Assets<Image>,
) -> Handle<Image> {
    assert_eq!(
        samples.dimensions.len(),
        2,
        "Expected 2D dimensions for draw_2d"
    );

    let width = samples.dimensions[0] as u32;
    let height = samples.dimensions[1] as u32;

    // Allocate RGBA8 buffer
    let mut data = vec![0u8; (width * height * 4) as usize];

    for (index, pos) in samples.dimensions.iter().enumerate() {
        let color = samples.samples[index].color();

        let rgba = color.to_srgba();
        let idx = (pos[1] as u32 * width + pos[0] as u32) as usize * 4;
        data[idx] = (rgba.red * 255.0).round().clamp(0.0, 255.0) as u8;
        data[idx + 1] = (rgba.green * 255.0).round().clamp(0.0, 255.0) as u8;
        data[idx + 2] = (rgba.blue * 255.0).round().clamp(0.0, 255.0) as u8;
        data[idx + 3] = (rgba.alpha * 255.0).round().clamp(0.0, 255.0) as u8;
    }

    let image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    images.add(image)
}

// Simple 2D rotation for DVec2 by angle (radians)
fn rotate(v: DVec2, angle: f64) -> DVec2 {
    let (s, c) = angle.sin_cos();
    DVec2::new(v.x * c - v.y * s, v.x * s + v.y * c)
}
