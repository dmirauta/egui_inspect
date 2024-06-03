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

#[macro_export]
macro_rules! quick_app_from {
    ($inspectable: ty) => {
        quick_app_from!($inspectable, {
            let mut name: &str = std::any::type_name::<$inspectable>();
            if let Some(_name) = name.split("::").last() {
                name = _name;
            }
            name
        });
    };
    ($inspectable: ty, $name: expr) => {
        quick_app_from!($inspectable, $name, Default::default());
    };
    ($inspectable: ty, $name: expr, $native_opts: expr) => {
        fn main() -> eframe::Result<()> {
            eframe::run_native(
                $name,
                $native_opts,
                Box::new(|_cc| Box::<egui_inspect::quick_app::QuickApp<$inspectable>>::default()),
            )
        }
    };
}
