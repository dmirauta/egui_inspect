use better_default::Default;
use egui_inspect::{
    logging::{
        init_with_mixed_log,
        log::{error, info, warn},
        LogsView,
    },
    EframeMain, EguiInspect, DPEQ,
};

#[derive(EguiInspect, DPEQ, Default)]
enum BasicLogSeverity {
    #[default]
    Info,
    Warn,
    Error,
}

#[derive(Default)]
struct LogEmitter {
    severity: BasicLogSeverity,
    message: String,
}

impl EguiInspect for LogEmitter {
    fn inspect_mut(&mut self, _label: &str, ui: &mut egui_inspect::egui::Ui) {
        ui.separator();
        ui.label("Insert log entries:");
        ui.horizontal(|ui| {
            self.severity.inspect_mut("severity", ui);
            self.message.inspect_mut("", ui);
            if ui.button("Log").clicked() {
                match &self.severity {
                    BasicLogSeverity::Info => info!("{}", self.message),
                    BasicLogSeverity::Warn => warn!("{}", self.message),
                    BasicLogSeverity::Error => error!("{}", self.message),
                }
            }
        });
    }
}

#[derive(EframeMain, EguiInspect, Default)]
#[eframe_main(init = "init_with_mixed_log::<LogTest>()")]
pub struct LogTest {
    #[inspect(name = "Feedback:")]
    feedback: LogsView,
    emitter: LogEmitter,
}
