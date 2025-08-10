use crate::{InitData, LayerData};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

pub fn gui_system(
    mut contexts: EguiContexts,
    mut layer_data: ResMut<LayerData>,
    mut init_data: ResMut<InitData>,
) -> Result {
    egui::Window::new("Control").show(contexts.ctx_mut()?, |ui| {
        ui.label("Target Depth:");
        ui.add(egui::DragValue::new(&mut layer_data.target_depth).speed(1));

        ui.label(format!("Current Depth: {}", layer_data.current_depth));

        ui.label("Width:");
        ui.add(egui::DragValue::new(&mut init_data.dimensions[0]).speed(1));
        ui.label("Height:");
        ui.add(egui::DragValue::new(&mut init_data.dimensions[1]).speed(1));

        ui.label("Mutation Scale:");

        let mutation_min = 0.000000001;
        let mutation_max = 100000.0;

        let speed = (init_data.all_scale / 20.0).clamp(mutation_min, mutation_max);
        ui.add(egui::DragValue::new(&mut init_data.all_scale).speed(speed));
        init_data.all_scale = init_data.all_scale.clamp(mutation_min, mutation_max);

        for (i, scale) in init_data.mutation_scale.iter_mut().enumerate() {
            let speed = (*scale / 20.0).clamp(mutation_min, mutation_max);
            ui.horizontal(|ui| {
                ui.label(format!("{}: ", i));
                ui.add(egui::DragValue::new(scale).speed(speed));
            });
            *scale = scale.clamp(mutation_min, mutation_max);
        }

        ui.label("Initial mutation:");
        for (i, mutation) in init_data.initial_mutation.iter_mut().enumerate() {
            let speed = (*mutation / 20.0).abs().clamp(mutation_min, mutation_max);
            ui.horizontal(|ui| {
                ui.label(format!("{}: ", i));
                ui.add(egui::DragValue::new(mutation).speed(speed));
            });
        }

        if ui.button("Redraw").clicked() {
            layer_data.request_update = true;
        }
    });

    Ok(())
}
