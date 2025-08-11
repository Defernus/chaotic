use crate::*;
use bevy::prelude::*;
use chaotic::ChaoticSystem;

// We can create our own gizmo config group!
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct AreaGizmos;

pub fn visualize_area<T: ChaoticSystem>(
    state: Res<ViewerState<T>>,
    layer_data: Res<LayerData>,
    init_data: Res<InitData<T>>,
    mut area_gizmos: Gizmos<AreaGizmos>,
) {
    let origin_x = state.initial_mutation[0];
    let origin_y = state.initial_mutation[1];
    let new_x = init_data.initial_mutation[0];
    let new_y = init_data.initial_mutation[1];

    let x_scale = state.all_scale * state.mutation_scale[0];
    let y_scale = state.all_scale * state.mutation_scale[1];
    let x_new_scale = init_data.all_scale * init_data.mutation_scale[0];
    let y_new_scale = init_data.all_scale * init_data.mutation_scale[1];

    let delta_x = new_x - origin_x;
    let delta_y = origin_y - new_y;

    let center_x = (delta_x / x_scale) as f32;
    let center_y = (delta_y / y_scale) as f32;

    let center = Vec3::X * center_x + Vec3::Y * center_y;
    let height = Vec3::Z * layer_data.current_size();

    area_gizmos.line(center, center + height, Color::WHITE);

    let x_h_range = (init_data.dimensions[0] as f64 / 2.0 * x_new_scale / x_scale) as f32;
    let y_h_range = (init_data.dimensions[1] as f64 / 2.0 * y_new_scale / y_scale) as f32;
    let a = center + Vec3::X * x_h_range + Vec3::Y * y_h_range;
    let b = center + Vec3::X * x_h_range - Vec3::Y * y_h_range;
    let c = center - Vec3::X * x_h_range + Vec3::Y * y_h_range;
    let d = center - Vec3::X * x_h_range - Vec3::Y * y_h_range;

    area_gizmos.line(a, a + height, Color::WHITE);
    area_gizmos.line(b, b + height, Color::WHITE);
    area_gizmos.line(c, c + height, Color::WHITE);
    area_gizmos.line(d, d + height, Color::WHITE);
}
