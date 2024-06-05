use crate::EguiInspect;

#[derive(Default)]
pub struct QuickApp<I: Default> {
    inner: I,
}

impl<I: Default + EguiInspect> eframe::App for QuickApp<I> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                self.inner.inspect_mut("", ui);
            })
        });
    }
}
