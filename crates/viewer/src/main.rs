use bevy::asset::RenderAssetUsages;
use bevy::math::DVec2;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use chaotic::*;

// Constants taken from the original Chaos main
const G: f64 = 1.1; // Gravitational constant
const DT: f64 = 0.31; // Time step for simulation
const UPDATES_PER_ITERATION: usize = 1;
const MUTATION: f64 = 0.000001;
const DIMENSIONS: Dimensions = Dimensions::new_static(&[128, 128]);

#[derive(Resource)]
struct ViewerState {
    initial_sample: ThreeBody,
    samples: Samples<ThreeBody>,
    display_sample: usize,
    // Image handles for each generated layer (latest at the end)
    layers: Vec<Handle<Image>>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<ClearColor>()
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_input,))
        .run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    // 2D camera is enough for now; we stack layers along Z
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection::default_3d()),
        Transform::from_translation(Vec3::ONE * 300.0).looking_at(Vec3::ZERO, Vec3::Z),
    ));

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

    let samples = Samples::new(initial_sample.clone(), DIMENSIONS.clone(), &[MUTATION; 12]);

    // Create first layer image and show it
    let first_layer = build_image(&samples, &mut images);

    let sprite_z = 0.0f32;
    commands.spawn((
        Sprite::from_image(first_layer.clone()),
        Transform::from_xyz(0.0, 0.0, sprite_z),
    ));

    commands.insert_resource(ViewerState {
        initial_sample,
        samples,
        display_sample: 0,
        layers: vec![first_layer],
    });
}

fn handle_input(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut images: ResMut<Assets<Image>>,
    mut state: ResMut<ViewerState>,
) {
    // Space: advance the simulation for all samples and add a new layer
    if keys.pressed(KeyCode::Space) {
        println!("Advancing simulation...");
        state.samples.update(UPDATES_PER_ITERATION, DT);
        let new_layer = build_image(&state.samples, &mut images);

        let z = state.layers.len() as f32 * 1.0; // stack slightly in front
        let entity = commands
            .spawn((
                Sprite::from_image(new_layer.clone()),
                Transform::from_xyz(0.0, 0.0, z),
            ))
            .id();

        // Keep the entity alive; handle cleanup later as needed
        let _ = entity; // suppress warning if not used elsewhere yet

        state.layers.push(new_layer);
    }

    // R: reset all samples to the initial state
    if keys.just_pressed(KeyCode::KeyR) {
        println!("Resetting samples to initial state...");
        let samples = Samples::new(
            state.initial_sample.clone(),
            DIMENSIONS.clone(),
            &[MUTATION; 12],
        );
        state.samples = samples;

        // Replace the latest layer preview
        let new_layer = build_image(&state.samples, &mut images);
        state.layers.clear();
        state.layers.push(new_layer.clone());
        // Optionally, we could also despawn previous layer entities; skipped for brevity
    }

    // F: select most stable system index (for potential inspection)
    if keys.just_pressed(KeyCode::KeyF) {
        println!("Selecting most stable system...");
        if let Some((index, _)) = state
            .samples
            .samples
            .iter()
            .map(ChaoticSystem::chaosity)
            .enumerate()
            .min_by(|a, b| a.1.total_cmp(&b.1))
        {
            state.display_sample = index;
            println!("Inspecting most stable simulation at {}", index);
        }
    }
}

fn build_image(samples: &Samples<ThreeBody>, images: &mut Assets<Image>) -> Handle<Image> {
    assert_eq!(DIMENSIONS.len(), 2, "Expected 2D dimensions for draw_2d");

    let width = DIMENSIONS[0] as u32;
    let height = DIMENSIONS[1] as u32;

    // Allocate RGBA8 buffer
    let mut data = vec![0u8; (width * height * 4) as usize];

    for (index, pos) in DIMENSIONS.iter().enumerate() {
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
