use std::{
    mem,
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
    fn exec_with_expected_steps(&self) -> Option<usize>;
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
            BackgroundTask::Restarting { .. } => {
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
        match self {
            BackgroundTask::Starting { task } => {
                task.inspect_mut(format!("{label} init parameters").as_str(), ui);
                self.poll_ready();
            }
            BackgroundTask::Restarting { .. } => {
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

                task.inspect_mut(
                    format!("{label} init parameters (start again)").as_str(),
                    ui,
                );
                self.poll_ready();
            }
        }
    }
}

impl<T: Task> BackgroundTask<T>
where
    T::Return: Send,
{
    pub fn spawn(expected_steps: usize, mut task: T) -> Self {
        let progress = Progress(Arc::new(Mutex::new(SynchedStats::new(expected_steps))));
        let _progress = progress.clone();
        let join_handle = std::thread::spawn(move || task.on_exec(_progress));
        Self::Ongoing {
            progress,
            join_handle: Some(join_handle),
        }
    }
    fn poll_ready(&mut self) {
        let expected_steps = match self {
            BackgroundTask::Starting { task } => task.exec_with_expected_steps(),
            BackgroundTask::Finished { task, .. } => task.exec_with_expected_steps(),
            _ => None,
        };
        if let Some(expected_steps) = expected_steps {
            match mem::replace(self, BackgroundTask::Restarting) {
                BackgroundTask::Starting { task } | BackgroundTask::Finished { task, .. } => {
                    *self = Self::spawn(expected_steps, task);
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
