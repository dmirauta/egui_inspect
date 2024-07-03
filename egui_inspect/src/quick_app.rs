use crate::EguiInspect;

pub struct QuickApp<I> {
    pub inner: I,
}

impl<I: EguiInspect> eframe::App for QuickApp<I> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                self.inner.inspect_mut("", ui);
            })
        });
    }
}
