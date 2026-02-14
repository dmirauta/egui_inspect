use crate::{utils::type_name_base, EguiInspect, DEFAULT_FRAME_STYLE};
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
                    format!("{label}: {t:.2} sec elapsed, {}/{exp_len} done", self.count)
                        .inspect("", ui);
                    ui.add(ProgressBar::new((self.count as f32) / (exp_len as f32)));
                }
                SynchedStatsOpts::Spinner { display_count } => {
                    match display_count {
                        true => {
                            format!("{label}: {t:.2} sec elapsed, {} done", self.count)
                        }
                        false => format!("{label}: {t:.2} sec elapsed"),
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
    fn begin_signal(&self) -> Option<SynchedStatsOpts>;
    fn on_exec(&mut self, progress: Progress) -> Self::Return;
}

enum BackgroundTaskState<T: Task> {
    PendingStart {
        init_params: T,
    },
    Ongoing {
        progress: Progress,
        join_handle: Option<JoinHandle<T::Return>>,
    },
    Restarting,
}

/// A struct which allows for easily running a task in the background while tracking its progress
/// in an egui ui. In the starting state it exposes the initialisation parameters for its
/// associated task, in the running/ongoing state it shows a progress bar, and in the finished
/// state it displays the result object and offers to restart.
pub struct BackgroundTask<T: Task, const COMPACT_LABELS: bool = false> {
    state: BackgroundTaskState<T>,
    pub res: Result<T::Return, String>,
}

impl<T: Task, const COMPACT_LABELS: bool> Default for BackgroundTask<T, COMPACT_LABELS> {
    fn default() -> Self {
        Self {
            state: BackgroundTaskState::PendingStart {
                init_params: Default::default(),
            },
            res: Err("".into()),
        }
    }
}
macro_rules! base_inspect {
    ($self:ident, $label: ident, $ui: ident, $state_inspect: tt) => {
        if COMPACT_LABELS {
            $state_inspect($self, "", &format!("{} progress", $label), $ui);
            base_inspect!($self, $ui); // res inspect
        } else {
            DEFAULT_FRAME_STYLE.to_frame().show($ui, |ui| {
                ui.strong(format!("{} ({})", $label, type_name_base::<T>()));
                $state_inspect($self, "params", "progress", ui);
                base_inspect!($self, ui); // res inspect
            });
        }
    };
    ($self:ident, $ui: ident) => {
        match &$self.res {
            Ok(r) => r.inspect("previous result", $ui),
            Err(e) => {
                if !e.is_empty() {
                    e.inspect("previous error", $ui)
                }
            }
        }
    };
}

impl<T: Task, const COMPACT_LABELS: bool> EguiInspect for BackgroundTask<T, COMPACT_LABELS>
where
    T::Return: EguiInspect + Send + 'static,
{
    fn inspect(&self, label: &str, ui: &mut egui::Ui) {
        let state_inspect = |s: &Self, _: &str, prog_label: &str, ui: &mut egui::Ui| {
            match &s.state {
                BackgroundTaskState::PendingStart { .. } => {
                    ui.label("Innactive task.");
                }
                BackgroundTaskState::Restarting => { /* state only briefly used inside poll_ready */
                }
                BackgroundTaskState::Ongoing { progress, .. } => progress.0.inspect(prog_label, ui),
            }
        };
        base_inspect!(self, label, ui, state_inspect);
    }

    fn inspect_mut(&mut self, label: &str, ui: &mut egui::Ui) {
        let state_inspect = |s: &mut Self,
                             param_label: &str,
                             prog_label: &str,
                             ui: &mut egui::Ui| {
            match &mut s.state {
                BackgroundTaskState::PendingStart { init_params } => {
                    init_params.inspect_mut(param_label, ui);
                    s.poll_ready();
                }
                BackgroundTaskState::Restarting => { /* state only briefly used inside poll_ready */
                }
                BackgroundTaskState::Ongoing { progress, .. } => {
                    // progress.0.inspect(&format!("{label} progress"), ui);
                    progress.0.inspect(prog_label, ui);
                    s.poll_result();
                }
            }
        };
        base_inspect!(self, label, ui, state_inspect);
    }
}

impl<T: Task, const COMPACT_LABELS: bool> BackgroundTask<T, COMPACT_LABELS>
where
    T::Return: Send,
{
    pub fn spawn(&mut self, ssopts: SynchedStatsOpts, mut init_params: T) {
        let progress = Progress(Arc::new(Mutex::new(SynchedStats::new(ssopts))));
        let _progress = progress.clone();
        let join_handle = std::thread::spawn(move || init_params.on_exec(_progress));
        self.state = BackgroundTaskState::Ongoing {
            progress,
            join_handle: Some(join_handle),
        };
    }
    fn poll_ready(&mut self) {
        let ssopts = match &self.state {
            BackgroundTaskState::PendingStart { init_params } => init_params.begin_signal(),
            _ => None,
        };
        if let Some(ssopts) = ssopts {
            if let BackgroundTaskState::PendingStart { init_params } =
                mem::replace(&mut self.state, BackgroundTaskState::Restarting)
            {
                self.spawn(ssopts, init_params);
            };
        }
    }
    fn poll_result(&mut self) {
        if let BackgroundTaskState::Ongoing { join_handle, .. } = &mut self.state {
            // NOTE: handle is only briefly None when we take the result just after its finished
            if join_handle.as_ref().unwrap().is_finished() {
                self.res = join_handle
                    .take()
                    .unwrap() // already checked is_some
                    .join()
                    .map_err(|e| format!("{e:?}"));
            } else {
                return;
            }
        }
        self.state = BackgroundTaskState::PendingStart {
            init_params: Default::default(),
        }
    }
}
