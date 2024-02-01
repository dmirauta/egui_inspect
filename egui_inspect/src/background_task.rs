use std::{
    sync::{Arc, Mutex},
    thread::JoinHandle,
    time::{Duration, Instant},
};

use crate::EguiInspect;
use egui::ProgressBar;

#[derive(Clone)]
pub struct SynchedStats {
    count: usize,
    expected_len: usize,
    start: Instant,
    elapsed: Duration,
}

impl SynchedStats {
    fn new(expected_len: usize) -> Self {
        Self {
            count: 0,
            expected_len,
            start: Instant::now(),
            elapsed: Default::default(),
        }
    }
    pub fn tick(&mut self) {
        self.count += 1;
        self.elapsed = self.start.elapsed();
    }
}

impl EguiInspect for SynchedStats {
    fn inspect(&self, label: &str, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let t = (self.elapsed.as_millis() as f32) / 1000.0;
            // TODO: better and/or custom formatting
            format!(
                "{label} progress: {}/{} done, {t:.2} sec elapsed",
                self.count, self.expected_len
            )
            .inspect("", ui);
            ui.add(ProgressBar::new(
                (self.count as f32) / (self.expected_len as f32),
            ));
            ui.ctx().request_repaint();
        });
    }

    fn inspect_mut(&mut self, label: &str, ui: &mut egui::Ui) {
        self.inspect(label, ui);
    }
}

pub type Progress = Arc<Mutex<SynchedStats>>;

pub trait ProgressFunc<R>: Fn(Progress) -> R + Send + 'static {}

pub trait Task<R>: Default + EguiInspect + Clone + Send + 'static {
    fn exec_with_expected_steps(&self) -> Option<usize>;
    fn on_exec(&mut self, progress: Progress) -> R;
}

pub enum BackgroundTask<R, T: Task<R>> {
    Starting {
        task: T,
    },
    Ongoing {
        progress: Progress,
        join_handle: Option<JoinHandle<R>>,
    },
    Finished {
        result: Result<R, String>,
        task: T,
    },
}

impl<R, I: Task<R>> Default for BackgroundTask<R, I> {
    fn default() -> Self {
        Self::Starting {
            task: Default::default(),
        }
    }
}

impl<R, T> EguiInspect for BackgroundTask<R, T>
where
    R: EguiInspect + Send + 'static,
    T: Task<R>,
{
    fn inspect(&self, label: &str, ui: &mut egui::Ui) {
        match self {
            BackgroundTask::Starting { .. } => {
                ui.label("Innactive task.");
            }
            BackgroundTask::Ongoing { progress, .. } => progress.inspect(label, ui),
            BackgroundTask::Finished { result, .. } => match result {
                Ok(r) => {
                    r.inspect(format!("{label} result").as_str(), ui);
                }
                Err(e) => e.inspect(format!("{label} error").as_str(), ui),
            },
        }
    }

    fn inspect_mut(&mut self, label: &str, ui: &mut egui::Ui) {
        let mut poll_ready = false;
        let mut poll_res = false;
        match self {
            BackgroundTask::Starting { task } => {
                task.inspect_mut(format!("{label} init parameters").as_str(), ui);
                poll_ready = true;
            }
            BackgroundTask::Ongoing { progress, .. } => {
                progress.inspect(label, ui);
                poll_res = true;
            }
            BackgroundTask::Finished { result, task } => {
                match result {
                    Ok(r) => {
                        r.inspect(format!("{label} result").as_str(), ui);
                    }
                    Err(e) => e.inspect(format!("{label} error").as_str(), ui),
                }

                task.inspect_mut(
                    format!("{label} init parameters (start again)").as_str(),
                    ui,
                );
                poll_ready = true;
            }
        }
        // state transitions
        if poll_ready {
            self.poll_ready();
        }
        if poll_res {
            self.poll_result();
        }
    }
}

impl<R, T> BackgroundTask<R, T>
where
    R: Send + 'static,
    T: Task<R>,
{
    pub fn spawn(expected_steps: usize, mut task: T) -> Self {
        let progress = Arc::new(Mutex::new(SynchedStats::new(expected_steps)));
        let _progress = progress.clone();
        let join_handle = std::thread::spawn(move || task.on_exec(_progress));
        Self::Ongoing {
            progress,
            join_handle: Some(join_handle),
        }
    }
    fn poll_ready(&mut self) {
        let task = match self {
            BackgroundTask::Starting { task } => Some(task),
            BackgroundTask::Finished { task, .. } => Some(task),
            _ => None,
        };
        if let Some(task) = task {
            if let Some(expected_steps) = task.exec_with_expected_steps() {
                // TODO: remove clone
                *self = Self::spawn(expected_steps, task.clone());
            }
        }
    }
    fn poll_result(&mut self) {
        let mut res = Err(String::new());
        if let BackgroundTask::Ongoing { join_handle, .. } = self {
            if join_handle.is_some() {
                if join_handle.as_ref().unwrap().is_finished() {
                    res = join_handle
                        .take()
                        .unwrap() // already checked is_some
                        .join()
                        .map_err(|e| format!("{e:?}"));
                }
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
