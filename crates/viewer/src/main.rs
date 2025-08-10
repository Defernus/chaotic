use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use viewer::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .init_resource::<ClearColor>()
        .insert_resource(ClearColor(Color::BLACK))
        .init_resource::<InitData>()
        .init_resource::<LayerData>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                camera_zoom,
                camera_move_by_mouse,
                rotate_camera,
                reset_layers_sys,
                process_layers_sys,
            ),
        )
        .add_systems(EguiPrimaryContextPass, gui_system)
        .run();
}

fn setup(mut commands: Commands, init_data: Res<InitData>) {
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

    let samples = init_data.init();

    commands.insert_resource(ViewerState { samples });
}
