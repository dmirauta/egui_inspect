use std::{cell::RefCell, collections::VecDeque, io::stdout, time::SystemTime};

use chrono::{DateTime, Local};
use egui::{Color32, RichText, ScrollArea};
pub use log;
use log::{info, warn};

use crate::{
    utils::{concat_rich_text, type_name_base},
    EguiInspect,
};

thread_local! {
    static GUI_LOG_DATA: RefCell<GuiLogData> = Default::default();
}

struct GuiLogData {
    items: VecDeque<(SystemTime, RichText, u32)>,
    max_logs_shown: usize,
    height: f32,
}

impl GuiLogData {
    fn push(&mut self, item: RichText) {
        #[allow(unused_assignments)]
        let mut creation_time = SystemTime::UNIX_EPOCH;

        // TODO: cannot use SystemTime::now() in WASM, is there another time source?
        #[cfg(not(target_arch = "wasm32"))]
        {
            creation_time = SystemTime::now();
        }

        if let Some(true) = self
            .items
            .back()
            .map(|(_, brt, _)| brt.text() == item.text())
        {
            let (_, _, count) = self.items.back_mut().unwrap();
            *count += 1; // TODO: items of different severity/color but the same characters will still combine
        } else {
            self.items.push_back((creation_time, item, 1));
        }
        if self.items.len() > self.max_logs_shown {
            self.items.pop_front();
        }
    }
}

impl Default for GuiLogData {
    fn default() -> Self {
        Self {
            items: VecDeque::new(),
            max_logs_shown: 20,
            height: 200.0,
        }
    }
}

impl EguiInspect for GuiLogData {
    fn inspect(&self, label: &str, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(label);
            if self.items.is_empty() {
                ui.label(RichText::new("Nothing yet.").color(Color32::WHITE));
            } else {
                ScrollArea::vertical()
                    .max_height(self.height)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            for (ct, rt, count) in self.items.iter() {
                                let dt: DateTime<Local> = (*ct).into();
                                let mut rtv = vec![
                                    RichText::new(format!("[{}] ", dt.format("%H:%M:%S")))
                                        .color(Color32::WHITE),
                                    rt.clone(),
                                ];
                                if *count > 1 {
                                    rtv.push(RichText::new(format!(" (x{count})")));
                                }
                                ui.label(concat_rich_text(rtv));
                            }
                        });
                    });
            }
        });
    }

    fn inspect_mut(&mut self, _label: &str, _ui: &mut egui::Ui) {
        todo!()
    }
}

pub fn set_log_ui_max_entries(max_logs_shown: usize) {
    GUI_LOG_DATA.with_borrow_mut(|f| f.max_logs_shown = max_logs_shown);
}

pub fn set_log_ui_height(height: f32) {
    GUI_LOG_DATA.with_borrow_mut(|f| f.height = height);
}

pub struct GuiLogger;

impl log::Log for GuiLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let color = match record.level() {
                log::Level::Error => Color32::RED,
                log::Level::Warn => Color32::GOLD,
                log::Level::Info => Color32::WHITE,
                log::Level::Debug => Color32::WHITE,
                log::Level::Trace => Color32::WHITE,
            };
            let rt = RichText::new(record.args().to_string()).color(color);
            GUI_LOG_DATA.with_borrow_mut(|f| {
                f.push(rt);
            });
        }
    }

    fn flush(&self) {}
}

pub enum FileLogOption {
    #[cfg(not(target_arch = "wasm32"))]
    FullPath {
        log_path: std::path::PathBuf,
    },
    #[cfg(not(target_arch = "wasm32"))]
    DefaultTempDir {
        log_name: String,
    },
    NoFileLog,
}

/// quickstart for logging everywhere
pub fn setup_mixed_logger_with_extra(opt: FileLogOption, extra: Option<Box<dyn log::Log>>) {
    let boxed_gui_log: Box<dyn log::Log> = Box::new(GuiLogger);
    let mut text_loggers = fern::Dispatch::new().format(|out, message, record| {
        #[allow(unused_assignments)]
        let mut prefix = String::new();

        #[cfg(not(target_arch = "wasm32"))]
        {
            let ct: DateTime<Local> = SystemTime::now().into();
            let fct = ct.format("%H:%M:%S");
            prefix = match record.line() {
                Some(line) => format!("[{fct} {} {} {line}]", record.level(), record.target()),
                None => format!("[{fct} {} {}]", record.level(), record.target()),
            };
        }

        #[cfg(target_arch = "wasm32")]
        {
            prefix = match record.line() {
                Some(line) => format!("[{} {} {line}]", record.level(), record.target()),
                None => format!("[{} {}]", record.level(), record.target()),
            };
        }

        out.finish(format_args!("{prefix} {message}",))
    });

    let mut file_log_success = true;
    let mut log_path_str: Option<String> = None;
    #[cfg(not(target_arch = "wasm32"))]
    {
        text_loggers = text_loggers.chain(stdout());

        let log_path = match opt {
            FileLogOption::FullPath { log_path } => Some(log_path),
            FileLogOption::DefaultTempDir { log_name } => Some(std::env::temp_dir().join(log_name)),
            FileLogOption::NoFileLog => None,
        };
        if let Some(log_path) = &log_path {
            log_path_str = Some(log_path.to_str().unwrap().to_string());
            if let Ok(file_log) = fern::log_file(log_path) {
                text_loggers = text_loggers.chain(file_log);
            } else {
                file_log_success = false;
            }
        }
    }

    let mut log_builder = fern::Dispatch::new()
        .level(log::LevelFilter::Info)
        .chain(boxed_gui_log)
        .chain(text_loggers);

    if let Some(logger) = extra {
        log_builder = log_builder.chain(logger);
    }

    match log_builder.apply() {
        Ok(_) => {
            if !file_log_success {
                warn!("Failed to setup logfile.");
            } else if let Some(log_path) = log_path_str {
                info!("Writing log messages to {log_path:?}.");
            }
        }
        Err(e) => eprintln!("Failed to build combined logger: {e}"),
    }
}

pub fn setup_mixed_logger(opt: FileLogOption) {
    setup_mixed_logger_with_extra(opt, None);
}

/// Displays a view into the stored GUI logs ([GuiLogData]) when inspected.
/// [GuiLogData] may be altered with egui_inspect::logging::set_log_ui_* methods.
#[derive(Default)]
pub struct LogsView;

impl EguiInspect for LogsView {
    fn inspect(&self, label: &str, ui: &mut egui::Ui) {
        GUI_LOG_DATA.with_borrow(|f| f.inspect(label, ui));
    }

    fn inspect_mut(&mut self, label: &str, ui: &mut egui::Ui) {
        GUI_LOG_DATA.with_borrow(|f| f.inspect(label, ui));
    }
}

pub fn default_mixed_logger<T>() {
    #[cfg(not(target_arch = "wasm32"))]
    setup_mixed_logger(FileLogOption::DefaultTempDir {
        log_name: format!("{}_log", type_name_base::<T>()),
    });
    #[cfg(target_arch = "wasm32")]
    setup_mixed_logger_with_extra(
        FileLogOption::NoFileLog,
        Some(Box::new(eframe::WebLogger::new(log::LevelFilter::Debug))),
    );
}

/// attach log initialisation to quick EframeMain app definition
pub fn init_with_mixed_log<T: Default>() -> T {
    default_mixed_logger::<T>();
    Default::default()
}
