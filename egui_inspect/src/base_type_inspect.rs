use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

use crate::InspectString;
use egui::Stroke;
use egui::Vec2;
use egui::{Color32, Ui};

macro_rules! impl_inspect_num {
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
            fn inspect_with_drag_value(&mut self, label: &str, ui: &mut egui::Ui, min: f32, max: f32) {
                ui.horizontal(|ui| {
                    ui.label(label.to_owned() + ":");
                    ui.add(egui::DragValue::new(self).max_decimals(10).range((min as $t)..=(max as $t)));
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
                ui.horizontal(|ui| {
                    ui.label(label.to_owned() + ":");
                    ui.add(egui::DragValue::new(self).max_decimals(10));
                });
            }
        }
        )*
    }
}

impl_inspect_num!(f32, f64, i8, u8, i16, u16, i32, u32, i64, u64, isize, usize);

impl crate::EguiInspect for &'static str {
    fn inspect(&self, label: &str, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if !label.is_empty() {
                ui.label(label.to_owned() + ":");
            }
            ui.label(self.to_string());
        });
    }
    fn inspect_mut(&mut self, label: &str, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if !label.is_empty() {
                ui.label(label.to_owned() + ":");
            }
            ui.colored_label(Color32::from_rgb(255, 0, 0), self.to_string())
                .on_hover_text("inspect_mut is not implemented for &'static str");
        });
    }
}

impl crate::EguiInspect for String {
    fn inspect(&self, label: &str, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if !label.is_empty() {
                ui.label(label.to_owned() + ":");
            }
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
            if !label.is_empty() {
                ui.label(label.to_owned() + ":");
            }
            ui.text_edit_multiline(self);
        });
    }

    fn inspect_mut_singleline(&mut self, label: &str, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if !label.is_empty() {
                ui.label(label.to_owned() + ":");
            }
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
            for (i, item) in self.iter().enumerate() {
                item.inspect(format!("{label}[{i}]").as_str(), ui);
            }
        });
    }

    fn inspect_mut(&mut self, label: &str, ui: &mut Ui) {
        let n = self.len();
        ui.collapsing(format!("{label} (len {n})"), |ui| {
            for (i, item) in self.iter_mut().enumerate() {
                item.inspect_mut(format!("{label}[{i}]").as_str(), ui);
            }
        });
    }
}

impl<T: crate::EguiInspect + Default> crate::EguiInspect for Vec<T> {
    fn inspect(&self, label: &str, ui: &mut Ui) {
        ui.collapsing(label, |ui| {
            for (i, item) in self.iter().enumerate() {
                item.inspect(format!("{label}[{i}]").as_str(), ui);
            }
        });
    }

    fn inspect_mut(&mut self, label: &str, ui: &mut Ui) {
        let n = self.len();
        ui.collapsing(label, |ui| {
            let mut to_remove = None;
            let mut to_swap = None;
            for (i, item) in self.iter_mut().enumerate() {
                item.inspect_mut(format!("{label}[{i}]").as_str(), ui);

                ui.horizontal_top(|ui| {
                    if ui.button("Remove").clicked() {
                        to_remove = Some(i);
                    }

                    if i < n - 1 && ui.button("Swap with next").clicked() {
                        to_swap = Some(i);
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

thread_local! {
    pub static NEW_KEY: RefCell<String> = Default::default();
}

macro_rules! impl_inspect_map {
    ($($t:ident),+) => {
        $(
        impl<T: crate::EguiInspect + Default> crate::EguiInspect for $t<String, T> {
            fn inspect(&self, label: &str, ui: &mut egui::Ui) {
                ui.collapsing(format!("{label}"), |ui| {
                    for (key, item) in self.iter() {
                        item.inspect(key.as_str(), ui);
                    }
                });
            }

            fn inspect_mut(&mut self, label: &str, ui: &mut egui::Ui) {
                ui.collapsing(format!("{label}"), |ui| {
                    let mut to_remove = None;
                    for (key, item) in self.iter_mut() {
                        item.inspect_mut(key.as_str(), ui);

                        if ui.button("Remove").clicked() {
                            to_remove = Some(key.clone());
                        }
                    }

                    if let Some(key) = to_remove {
                        self.remove(&key);
                    }

                    ui.menu_button("Insert default", |ui| {
                        NEW_KEY.with_borrow_mut(|s| {
                            s.inspect_mut("new key", ui);
                            if ui.button("Insert").clicked() {
                                self.insert(s.clone(), T::default());
                                ui.close_menu();
                            }
                        });
                    });
                });
            }
        }
        )*
    };
}

impl_inspect_map!(HashMap, BTreeMap);

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
                if ui.button("Set to None").clicked() {
                    *self = None;
                }
            }
            None => {
                ui.label(format!("\"{label}\" is None").as_str());
                if ui.button("Set to default").clicked() {
                    *self = Some(T::default());
                }
            }
        });
    }
}

impl<T: crate::EguiInspect> crate::EguiInspect for Arc<Mutex<T>> {
    fn inspect(&self, label: &str, ui: &mut egui::Ui) {
        if let Ok(guard) = self.try_lock() {
            guard.inspect(label, ui);
        }
    }

    fn inspect_mut(&mut self, label: &str, ui: &mut egui::Ui) {
        if let Ok(mut guard) = self.try_lock() {
            guard.inspect_mut(label, ui);
        }
    }
}

impl<T: crate::EguiInspect> crate::EguiInspect for Rc<RefCell<T>> {
    fn inspect(&self, label: &str, ui: &mut egui::Ui) {
        if let Ok(guard) = self.try_borrow() {
            guard.inspect(label, ui);
        }
    }

    fn inspect_mut(&mut self, label: &str, ui: &mut egui::Ui) {
        if let Ok(mut guard) = self.try_borrow_mut() {
            guard.inspect_mut(label, ui);
        }
    }
}

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
        ui.horizontal(|ui| {
            ui.label(format!("{label}: "));
            ui.add(self);
        });
    }
}

impl crate::EguiInspect for Vec2 {
    fn inspect(&self, label: &str, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(label);
            self.x.inspect("x", ui);
            self.y.inspect("y", ui);
        });
    }

    fn inspect_mut(&mut self, label: &str, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(label);
            self.x.inspect_mut("x", ui);
            self.y.inspect_mut("y", ui);
        });
    }
}
