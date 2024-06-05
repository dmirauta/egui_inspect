use std::time::Duration;

use egui_inspect::{
    background_task::{BackgroundTask, Progress, Task},
    EframeMain, EguiInspect, InspectNumber, DPEQ,
};

#[derive(EguiInspect, Clone, DPEQ)]
enum Mode {
    Ordinary,
    Power {
        #[inspect(slider, min = 2.0, max = 5.0)]
        p: u32,
    },
}

#[derive(EguiInspect, Clone)]
struct MySummation {
    iters: usize,
    sleep_millis: u64,
    mode: Mode,
    ready: bool,
}

impl Default for MySummation {
    fn default() -> Self {
        Self {
            iters: 100,
            sleep_millis: 25,
            mode: Mode::Ordinary,
            ready: false,
        }
    }
}

impl Task for MySummation {
    type Return = u32;
    fn exec_with_expected_steps(&self) -> Option<usize> {
        // provide an expected number of iterations required if ready to start
        if self.ready {
            Some(self.iters)
        } else {
            None
        }
    }
    fn on_exec(&mut self, progress: Progress) -> Self::Return {
        (0..self.iters as u32)
            .map(|i| {
                progress.increment();
                std::thread::sleep(Duration::from_millis(self.sleep_millis));
                match self.mode {
                    Mode::Ordinary => i,
                    Mode::Power { p } => i.pow(p),
                }
            })
            .sum()
    }
}

#[derive(Default, EguiInspect, EframeMain)]
pub struct AutoProgressBarTest {
    background_task_1: BackgroundTask<MySummation>,
    background_task_2: BackgroundTask<MySummation>,
}
