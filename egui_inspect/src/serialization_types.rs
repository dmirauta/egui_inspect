// NOTE: Could be derived via [egui_inspect_wrap::shadow_struct] once that supports enums,
// though a plus side of the manual implementation is that the structure is fixed (no
// array or hashmap/object-field inserting/removing).

use chrono::Datelike;
use egui_extras::DatePickerButton;
use toml::value::Date;

impl crate::EguiInspect for toml::value::Date {
    fn inspect(&self, label: &str, ui: &mut egui::Ui) {
        format!("{self}").inspect(label, ui)
    }

    fn inspect_mut(&mut self, label: &str, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(label);
            let mut cdt = chrono::naive::NaiveDate::from_ymd_opt(
                self.year.into(),
                self.month.into(),
                self.day.into(),
            );
            if let Some(cdt) = &mut cdt {
                ui.add(DatePickerButton::new(cdt));
                // TODO: Careful casting?
                *self = Date {
                    year: cdt.year() as u16,
                    month: cdt.month() as u8,
                    day: cdt.day() as u8,
                }
            } else {
                ui.label("<date error>");
            }
        });
    }
}

impl crate::EguiInspect for toml::Value {
    fn inspect(&self, label: &str, ui: &mut egui::Ui) {
        match self {
            toml::Value::String(s) => s.inspect(label, ui),
            toml::Value::Integer(i) => i.inspect(label, ui),
            toml::Value::Float(f) => f.inspect(label, ui),
            toml::Value::Boolean(b) => b.inspect(label, ui),
            toml::Value::Datetime(dt) => format!("{dt}").inspect(label, ui),
            toml::Value::Array(arr) => {
                ui.collapsing(label, |ui| {
                    for (i, item) in arr.iter().enumerate() {
                        item.inspect(format!("{label}[{i}]").as_str(), ui);
                    }
                });
            }
            toml::Value::Table(tab) => {
                ui.collapsing(label.to_string(), |ui| {
                    for (key, item) in tab.iter() {
                        item.inspect(key.as_str(), ui);
                    }
                });
            }
        }
    }

    fn inspect_mut(&mut self, label: &str, ui: &mut egui::Ui) {
        match self {
            toml::Value::String(s) => s.inspect_mut(label, ui),
            toml::Value::Integer(i) => i.inspect_mut(label, ui),
            toml::Value::Float(f) => f.inspect_mut(label, ui),
            toml::Value::Boolean(b) => b.inspect_mut(label, ui),
            toml::Value::Datetime(dt) => {
                if let Some(d) = &mut dt.date {
                    d.inspect_mut(label, ui);
                }
                // TODO: handle time component of datetime...
            }
            toml::Value::Array(arr) => {
                ui.collapsing(label, |ui| {
                    for (i, item) in arr.iter_mut().enumerate() {
                        item.inspect_mut(format!("{label}[{i}]").as_str(), ui);
                    }
                });
            }
            toml::Value::Table(tab) => {
                ui.collapsing(label.to_string(), |ui| {
                    for (key, item) in tab.iter_mut() {
                        item.inspect_mut(key.as_str(), ui);
                    }
                });
            }
        }
    }
}
