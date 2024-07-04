impl crate::EguiInspect for egui::Color32 {
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

impl crate::EguiInspect for egui::Stroke {
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

impl crate::EguiInspect for egui::Vec2 {
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
