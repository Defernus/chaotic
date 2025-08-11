use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use chaotic::ChaoticSystem;
use viewer::*;

type System = chaotic::NBody;

fn main() {
    App::new()
        .init_gizmo_group::<AreaGizmos>()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .init_resource::<ClearColor>()
        .insert_resource(ClearColor(Color::BLACK))
        .init_resource::<InitData<System>>()
        .init_resource::<LayerData>()
        .add_systems(Startup, setup::<System>)
        .add_systems(
            Update,
            (
                camera_zoom,
                camera_move_by_mouse,
                rotate_camera,
                reset_layers_sys::<System>,
                process_layers_sys::<System>,
                visualize_area::<System>,
            ),
        )
        .add_systems(EguiPrimaryContextPass, gui_system::<System>)
        .run();
}

fn setup<T: ChaoticSystem + Clone>(mut commands: Commands, init_data: Res<InitData<T>>) {
    // 2D camera is enough for now; we stack layers along Z
    commands.spawn((
        Camera2d,
        Transform::from_translation(Vec3::ONE * 10000.0).looking_at(Vec3::ZERO, Vec3::Z),
        MainCamera::default(),
        Projection::Orthographic(OrthographicProjection {
            far: 200000.0,
            ..OrthographicProjection::default_3d()
        }),
    ));

    let state = init_data.init();

    commands.insert_resource(state);
}
