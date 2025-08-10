// use bevy_egui::egui;
// use std::str::FromStr;

// pub trait NumberInput {
//     fn input_from_str<T: FromStr>(&mut self, number: &mut T) -> bool;
// }

// impl NumberInput for egui::Ui {
//     fn input_from_str<T: FromStr>(&mut self, text: &mut String) -> Option<T> {
//         self.text_edit_singleline(text);

//         match text.parse::<T>() {
//             Ok(value) => value,
//             Err(err) => {}
//         }
//     }
// }
