use std::time::Duration;

use egui_inspect::{
    background_task::{BackgroundTask, Progress, Task},
    quick_app::{IntoApp, QuickApp},
    EguiInspect,
};

#[derive(EguiInspect, Clone, PartialEq)]
enum Mode {
    Ordinary,
    Squares,
}

#[derive(EguiInspect, Clone)]
struct ParamPick {
    iters: usize,
    sleep_millis: u64,
    mode: Mode,
    ready: bool,
}

impl Default for ParamPick {
    fn default() -> Self {
        Self {
            iters: 100,
            sleep_millis: 25,
            mode: Mode::Ordinary,
            ready: false,
        }
    }
}

impl Task<usize> for ParamPick {
    fn exec_with_expected_steps(&self) -> Option<usize> {
        if self.ready {
            Some(self.iters)
        } else {
            None
        }
    }
    fn on_exec(&mut self, progress: Progress) -> usize {
        (0..self.iters)
            .map(|i| {
                if let Ok(mut mtx) = progress.lock() {
                    mtx.tick();
                }
                std::thread::sleep(Duration::from_millis(self.sleep_millis));
                match self.mode {
                    Mode::Ordinary => i,
                    Mode::Squares => i * i,
                }
            })
            .sum()
    }
}

#[derive(Default, EguiInspect)]
pub struct AutoProgressBarTest {
    background_task_1: BackgroundTask<usize, ParamPick>,
    background_task_2: BackgroundTask<usize, ParamPick>,
}

impl IntoApp for AutoProgressBarTest {}

fn main() -> eframe::Result<()> {
    QuickApp::<AutoProgressBarTest>::run()
}
