use crate::{utils::type_name_base, EguiInspect};
use egui::ProgressBar;
use std::{
    mem,
    sync::{Arc, Mutex},
    thread::JoinHandle,
    time::Instant,
};

#[derive(Clone)]
pub enum SynchedStatsOpts {
    HasExpectedLen(usize),
    Spinner { display_count: bool },
}

impl Default for SynchedStatsOpts {
    fn default() -> Self {
        SynchedStatsOpts::Spinner {
            display_count: false,
        }
    }
}

#[derive(Clone)]
pub struct SynchedStats {
    count: usize,
    start: Instant,
    opts: SynchedStatsOpts,
}

impl SynchedStats {
    fn new(opts: SynchedStatsOpts) -> Self {
        Self {
            count: 0,
            start: Instant::now(),
            opts,
        }
    }
    pub fn tick(&mut self) {
        self.count += 1;
    }
}

impl EguiInspect for SynchedStats {
    fn inspect(&self, label: &str, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let elapsed = self.start.elapsed();
            let t = (elapsed.as_millis() as f32) / 1000.0;
            match self.opts {
                SynchedStatsOpts::HasExpectedLen(exp_len) => {
                    // TODO: better and/or custom formatting
                    format!(
                        "{label} progress: {t:.2} sec elapsed, {}/{exp_len} done",
                        self.count
                    )
                    .inspect("", ui);
                    ui.add(ProgressBar::new((self.count as f32) / (exp_len as f32)));
                }
                SynchedStatsOpts::Spinner { display_count } => {
                    match display_count {
                        true => {
                            format!("{label} progress: {t:.2} sec elapsed, {} done", self.count)
                        }
                        false => format!("{label} progress: {t:.2} sec elapsed"),
                    }
                    .inspect("", ui);

                    ui.spinner();
                }
            }
            ui.ctx().request_repaint();
        });
    }

    fn inspect_mut(&mut self, label: &str, ui: &mut egui::Ui) {
        self.inspect(label, ui);
    }
}

#[derive(Clone)]
pub struct Progress(Arc<Mutex<SynchedStats>>);

impl Progress {
    pub fn increment(&self) {
        if let Ok(mut mtx) = self.0.lock() {
            mtx.tick();
        }
    }
}

pub trait Task: Default + EguiInspect + Send + 'static {
    type Return;
    fn exec_with_expected_steps(&self) -> Option<SynchedStatsOpts>;
    fn on_exec(&mut self, progress: Progress) -> Self::Return;
}

/// A struct which allows for easily running a task in the background while tracking its progress
/// in an egui ui. In the starting state it exposes the initialisation parameters for its
/// associated task, in the running/ongoing state it shows a progress bar, and in the finished
/// state it displays the result object and offers to restart.
pub enum BackgroundTask<T: Task> {
    Starting {
        task: T,
    },
    Ongoing {
        progress: Progress,
        join_handle: Option<JoinHandle<T::Return>>,
    },
    Finished {
        result: Result<T::Return, String>,
        task: T,
    },
    Restarting,
}

impl<T: Task> Default for BackgroundTask<T> {
    fn default() -> Self {
        Self::Starting {
            task: Default::default(),
        }
    }
}

impl<T: Task> EguiInspect for BackgroundTask<T>
where
    T::Return: EguiInspect + Send + 'static,
{
    fn inspect(&self, label: &str, ui: &mut egui::Ui) {
        match self {
            BackgroundTask::Starting { .. } => {
                ui.label("Innactive task.");
            }
            BackgroundTask::Restarting => {
                ui.label("Restarting...");
            }
            BackgroundTask::Ongoing { progress, .. } => progress.0.inspect(label, ui),
            BackgroundTask::Finished { result, .. } => match result {
                Ok(r) => {
                    r.inspect(format!("{label} result").as_str(), ui);
                }
                Err(e) => e.inspect(format!("{label} error").as_str(), ui),
            },
        }
    }

    fn inspect_mut(&mut self, label: &str, ui: &mut egui::Ui) {
        let mut params_base_label = format!("{} parameters", type_name_base::<T>());
        if !label.is_empty() {
            params_base_label += format!(" ({label})").as_str();
        }

        match self {
            BackgroundTask::Starting { task } => {
                task.inspect_mut(params_base_label.as_str(), ui);
                self.poll_ready();
            }
            BackgroundTask::Restarting => {
                ui.label("Restarting...");
            }
            BackgroundTask::Ongoing { progress, .. } => {
                progress.0.inspect(label, ui);
                self.poll_result();
            }
            BackgroundTask::Finished { result, task } => {
                match result {
                    Ok(r) => {
                        r.inspect(format!("{label} result").as_str(), ui);
                    }
                    Err(e) => e.inspect(format!("{label} error").as_str(), ui),
                }

                task.inspect_mut(format!("{params_base_label} (start again)").as_str(), ui);
                self.poll_ready();
            }
        }
    }
}

impl<T: Task> BackgroundTask<T>
where
    T::Return: Send,
{
    pub fn spawn(ssopts: SynchedStatsOpts, mut task: T) -> Self {
        let progress = Progress(Arc::new(Mutex::new(SynchedStats::new(ssopts))));
        let _progress = progress.clone();
        let join_handle = std::thread::spawn(move || task.on_exec(_progress));
        Self::Ongoing {
            progress,
            join_handle: Some(join_handle),
        }
    }
    fn poll_ready(&mut self) {
        let ssopts = match self {
            BackgroundTask::Starting { task } => task.exec_with_expected_steps(),
            BackgroundTask::Finished { task, .. } => task.exec_with_expected_steps(),
            _ => None,
        };
        if let Some(ssopts) = ssopts {
            match mem::replace(self, BackgroundTask::Restarting) {
                BackgroundTask::Starting { task } | BackgroundTask::Finished { task, .. } => {
                    *self = Self::spawn(ssopts, task);
                }
                _ => {}
            };
        }
    }
    fn poll_result(&mut self) {
        let mut res = Err(String::new());
        if let BackgroundTask::Ongoing { join_handle, .. } = self {
            if join_handle.is_some() && join_handle.as_ref().unwrap().is_finished() {
                res = join_handle
                    .take()
                    .unwrap() // already checked is_some
                    .join()
                    .map_err(|e| format!("{e:?}"));
            }
        }
        if let Err(es) = &res {
            if es.is_empty() {
                return;
            }
        }
        *self = BackgroundTask::Finished {
            result: res,
            task: Default::default(),
        }
    }
}
