use std::any::type_name;

use crate::EguiInspect;

pub trait IntoApp: Default {
    fn name() -> &'static str {
        let mut name: &str = type_name::<Self>().into();
        if let Some(_name) = name.split("::").last() {
            name = _name;
        }
        name
    }
    fn eframe_native_opts() -> eframe::NativeOptions {
        Default::default()
    }
    fn create(_cc: &eframe::CreationContext) -> Self {
        Default::default()
    }
}

pub struct QuickApp<I: Default> {
    inner: I,
}

impl<I: Default + EguiInspect> eframe::App for QuickApp<I> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.inner.inspect_mut("", ui);
        });
    }
}

impl<I: Default + EguiInspect + IntoApp + 'static> QuickApp<I> {
    pub fn run() -> eframe::Result<()> {
        eframe::run_native(
            I::name(),
            I::eframe_native_opts(),
            Box::new(|cc| {
                Box::new(QuickApp {
                    inner: I::create(cc),
                })
            }),
        )
    }
}
