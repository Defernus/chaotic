use bevy::asset::RenderAssetUsages;
use bevy::math::DVec2;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use chaotic::*;

// Constants taken from the original Chaos main
const G: f64 = 1.1; // Gravitational constant
const DT: f64 = 0.31; // Time step for simulation
const UPDATES_PER_ITERATION: usize = 1;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .init_resource::<ClearColor>()
        .insert_resource(ClearColor(Color::BLACK))
        .init_resource::<InitData>()
        .init_resource::<LayerData>()
        .add_systems(Startup, setup)
        .add_systems(Update, (reset_layers_sys, process_layers_sys))
        .add_systems(EguiPrimaryContextPass, gui_system)
        .run();
}

#[derive(Resource)]
pub struct InitData {
    pub width: usize,
    pub height: usize,

    pub mutation_scale: Vec<f64>,
    pub dimensions: Dimensions,

    pub initial_sample: ThreeBody,
}

impl Default for InitData {
    fn default() -> Self {
        // Build initial ThreeBody system (matching the original Chaos main)
        let angle_a = 0.0;
        let angle_b = std::f64::consts::PI * (1.0 / 3.0) * 2.0;
        let angle_c = std::f64::consts::PI * (2.0 / 3.0) * 2.0;
        let mass = 0.1;
        let velocity = 0.31;

        let initial_sample = ThreeBody::new(
            G,
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
        );

        Self {
            initial_sample,
            width: 128,
            height: 128,
            mutation_scale: vec![0.00001, 0.00001],
            dimensions: Dimensions::new_static(&[128, 128]),
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
            target_depth: 32,
            current_depth: 0,
            request_update: false,
        }
    }
}

#[derive(Resource)]
pub struct ViewerState {
    pub samples: Samples<ThreeBody>,
}

#[derive(Component)]
pub struct Layer;

pub fn gui_system(mut contexts: EguiContexts, mut layer_data: ResMut<LayerData>) -> Result {
    egui::Window::new("Control").show(contexts.ctx_mut()?, |ui| {
        ui.label("Target Depth:");
        ui.add(egui::DragValue::new(&mut layer_data.target_depth).speed(1));
    });

    Ok(())
}

fn setup(mut commands: Commands, init_data: Res<InitData>) {
    // 2D camera is enough for now; we stack layers along Z
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection::default_3d()),
        Transform::from_translation(Vec3::ONE * 300.0).looking_at(Vec3::ZERO, Vec3::Z),
    ));

    let samples = Samples::new(
        init_data.initial_sample.clone(),
        init_data.dimensions.clone(),
        &init_data.mutation_scale,
    );

    commands.insert_resource(ViewerState { samples });
}

fn reset_layers_sys(
    mut commands: Commands,
    mut state: ResMut<ViewerState>,
    init_data: Res<InitData>,
    mut layer_data: ResMut<LayerData>,
    layers_q: Query<Entity, With<Layer>>,
) -> Result<(), BevyError> {
    if layer_data.request_update {
        for layer in layers_q.iter() {
            commands.entity(layer).despawn();
        }

        state.samples = Samples::new(
            init_data.initial_sample.clone(),
            init_data.dimensions.clone(),
            &init_data.mutation_scale,
        );

        layer_data.current_depth = 0;
    }
    Ok(())
}

fn process_layers_sys(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut state: ResMut<ViewerState>,
    init_data: Res<InitData>,
    mut layer_data: ResMut<LayerData>,
    mut camera_q: Query<&mut Transform, With<Camera2d>>,
) -> Result<(), BevyError> {
    if layer_data.current_depth < layer_data.target_depth {
        let mut camera_transform = camera_q.single_mut()?;

        println!("Advancing simulation...");
        state.samples.update(UPDATES_PER_ITERATION, DT);
        let new_layer = build_image(&state.samples, &init_data.dimensions, &mut images);

        camera_transform.translation.z = layer_data.current_depth as f32 + 200.0;
        commands.spawn((
            Layer,
            Sprite::from_image(new_layer.clone()),
            Transform::from_xyz(0.0, 0.0, layer_data.current_depth as f32),
        ));

        layer_data.current_depth += 1;
    }

    // // F: select most stable system index (for potential inspection)
    // if keys.just_pressed(KeyCode::KeyF) {
    //     println!("Selecting most stable system...");
    //     if let Some((index, _)) = state
    //         .samples
    //         .samples
    //         .iter()
    //         .map(ChaoticSystem::chaosity)
    //         .enumerate()
    //         .min_by(|a, b| a.1.total_cmp(&b.1))
    //     {
    //         state.display_sample = index;
    //         println!("Inspecting most stable simulation at {}", index);
    //     }
    // }

    Ok(())
}

fn build_image(
    samples: &Samples<ThreeBody>,
    dimensions: &Dimensions,
    images: &mut Assets<Image>,
) -> Handle<Image> {
    assert_eq!(dimensions.len(), 2, "Expected 2D dimensions for draw_2d");

    let width = dimensions[0] as u32;
    let height = dimensions[1] as u32;

    // Allocate RGBA8 buffer
    let mut data = vec![0u8; (width * height * 4) as usize];

    for (index, pos) in dimensions.iter().enumerate() {
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
