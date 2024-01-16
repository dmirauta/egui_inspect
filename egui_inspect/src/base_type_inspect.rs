use crate::InspectNumber;
use crate::InspectString;
use egui::Stroke;
use egui::{Color32, Ui};

macro_rules! impl_inspect_float {
    ($($t:ty),+) => {
        $(
            impl crate::InspectNumber for $t {
                fn inspect_with_slider(&mut self, label: &str, ui: &mut egui::Ui, min: f32, max: f32) {
                    ui.horizontal(|ui| {
                        ui.label(label.to_owned() + ":");
                        ui.add(egui::Slider::new(self, (min as $t)..=(max as $t)));
                    });
                }
                fn inspect_with_log_slider(&mut self, label: &str, ui: &mut egui::Ui, min: f32, max: f32) {
                    ui.horizontal(|ui| {
                        ui.label(label.to_owned() + ":");
                        ui.add(egui::Slider::new(self, (min as $t)..=(max as $t)).logarithmic(true));
                    });
                }
                fn inspect_with_drag_value(&mut self, label: &str, ui: &mut egui::Ui) {
                    ui.horizontal(|ui| {
                        ui.label(label.to_owned() + ":");
                        ui.add(egui::DragValue::new(self));
                    });
                }
            }

            impl crate::EguiInspect for $t {
                fn inspect(&self, label: &str, ui: &mut egui::Ui) {
                    ui.horizontal(|ui| {
                        ui.label(label.to_owned() + ":");
                        ui.label(self.to_string());
                    });
                }
                fn inspect_mut(&mut self, label: &str, ui: &mut egui::Ui) {
                    self.inspect_with_slider(label, ui, 0.0f32, 100.0f32);
                }
            }
        )*
    }
}

macro_rules! impl_inspect_int {
    ($($t:ty),+) => {
        $(
        impl crate::InspectNumber for $t {
            fn inspect_with_slider(&mut self, label: &str, ui: &mut egui::Ui, min: f32, max: f32) {
                ui.horizontal(|ui| {
                    ui.label(label.to_owned() + ":");
                    ui.add(egui::Slider::new(self, (min as $t)..=(max as $t)));
                });
            }
            fn inspect_with_log_slider(&mut self, label: &str, ui: &mut egui::Ui, min: f32, max: f32) {
                ui.horizontal(|ui| {
                    ui.label(label.to_owned() + ":");
                    ui.add(egui::Slider::new(self, (min as $t)..=(max as $t)).logarithmic(true));
                });
            }
            fn inspect_with_drag_value(&mut self, label: &str, ui: &mut egui::Ui) {
                ui.horizontal(|ui| {
                    ui.label(label.to_owned() + ":");
                    ui.add(egui::DragValue::new(self));
                });
            }
        }

        impl crate::EguiInspect for $t {
            fn inspect(&self, label: &str, ui: &mut egui::Ui) {
                ui.horizontal(|ui| {
                    ui.label(label.to_owned() + ":");
                    ui.label(self.to_string());
                });
            }
            fn inspect_mut(&mut self, label: &str, ui: &mut egui::Ui) {
                self.inspect_with_slider(label, ui, 0.0, 100.0);
            }
        }
        )*
    }
}

impl_inspect_float!(f32, f64);

impl_inspect_int!(i8, u8);
impl_inspect_int!(i16, u16);
impl_inspect_int!(i32, u32);
impl_inspect_int!(i64, u64);
impl_inspect_int!(isize, usize);

impl crate::EguiInspect for &'static str {
    fn inspect(&self, label: &str, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(label.to_owned() + ":");
            ui.label(self.to_string());
        });
    }
    fn inspect_mut(&mut self, label: &str, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(label.to_owned() + ":");
            ui.colored_label(Color32::from_rgb(255, 0, 0), self.to_string())
                .on_hover_text("inspect_mut is not implemented for &'static str");
        });
    }
}

impl crate::EguiInspect for String {
    fn inspect(&self, label: &str, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(label.to_owned() + ":");
            ui.label(self);
        });
    }
    fn inspect_mut(&mut self, label: &str, ui: &mut egui::Ui) {
        self.inspect_mut_singleline(label, ui);
    }
}

impl crate::InspectString for String {
    fn inspect_mut_multiline(&mut self, label: &str, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(label.to_owned() + ":");
            ui.text_edit_multiline(self);
        });
    }

    fn inspect_mut_singleline(&mut self, label: &str, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(label.to_owned() + ":");
            ui.text_edit_singleline(self);
        });
    }
}

impl crate::EguiInspect for bool {
    fn inspect(&self, label: &str, ui: &mut egui::Ui) {
        ui.add_enabled(false, egui::Checkbox::new(&mut self.clone(), label));
    }
    fn inspect_mut(&mut self, label: &str, ui: &mut egui::Ui) {
        ui.checkbox(self, label);
    }
}

impl<T: crate::EguiInspect, const N: usize> crate::EguiInspect for [T; N] {
    fn inspect(&self, label: &str, ui: &mut Ui) {
        let n = self.len();
        ui.collapsing(format!("{label} (len {n})"), |ui| {
            for item in self.iter() {
                item.inspect("item", ui);
            }
        });
    }

    fn inspect_mut(&mut self, label: &str, ui: &mut Ui) {
        let n = self.len();
        ui.collapsing(format!("{label} (len {n})"), |ui| {
            for item in self.iter_mut() {
                item.inspect_mut("item", ui);
            }
        });
    }
}

impl<T: crate::EguiInspect + Default> crate::EguiInspect for Vec<T> {
    fn inspect(&self, label: &str, ui: &mut Ui) {
        let n = self.len();
        ui.collapsing(format!("{label} (len {n})"), |ui| {
            for (i, item) in self.iter().enumerate() {
                item.inspect(format!("{label}[{i}]").as_str(), ui);
            }
        });
    }

    fn inspect_mut(&mut self, label: &str, ui: &mut Ui) {
        let n = self.len();
        ui.collapsing(format!("{label} (len {n})"), |ui| {
            let mut to_remove = None;
            let mut to_swap = None;
            for (i, item) in self.iter_mut().enumerate() {
                ui.horizontal_top(|ui| {
                    item.inspect_mut(format!("{label}[{i}]").as_str(), ui);

                    if ui.button("Remove").clicked() {
                        to_remove = Some(i);
                    }

                    if i < n - 1 {
                        if ui.button("Swap with next").clicked() {
                            to_swap = Some(i);
                        }
                    }
                });
            }

            if let Some(i) = to_remove {
                self.remove(i);
            }
            if let Some(i) = to_swap {
                let e = self.remove(i);
                self.insert(i + 1, e);
            }

            if ui.button("Push default").clicked() {
                self.push(T::default());
            }
        });
    }
}

impl<T: crate::EguiInspect + Default> crate::EguiInspect for Option<T> {
    fn inspect(&self, label: &str, ui: &mut egui::Ui) {
        match self {
            Some(v) => {
                v.inspect(label, ui);
            }
            None => {
                ui.label(format!("{label} is None").as_str());
            }
        }
    }

    fn inspect_mut(&mut self, label: &str, ui: &mut egui::Ui) {
        ui.horizontal_top(|ui| match self {
            Some(v) => {
                ui.vertical(|ui| {
                    v.inspect_mut(label, ui);
                });
                if ui.button(format!("Set to None").as_str()).clicked() {
                    *self = None;
                }
            }
            None => {
                ui.label(format!("\"{label}\" is None").as_str());
                if ui.button(format!("Set to default").as_str()).clicked() {
                    *self = Some(T::default());
                }
            }
        });
    }
}

//// Egui style types

impl crate::EguiInspect for Color32 {
    fn inspect(&self, label: &str, ui: &mut egui::Ui) {
        ui.label(format!("{label}: {:?}", self));
    }

    fn inspect_mut(&mut self, label: &str, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(label);
            ui.color_edit_button_srgba(self);
        });
    }
}

impl crate::EguiInspect for Stroke {
    fn inspect(&self, label: &str, ui: &mut egui::Ui) {
        ui.label(format!("{label}: {:?}", self));
    }

    fn inspect_mut(&mut self, label: &str, ui: &mut egui::Ui) {
        egui::stroke_ui(ui, self, label);
    }
}
