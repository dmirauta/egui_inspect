use egui_inspect::{
    background_task::{BackgroundTask, Progress, SynchedStatsOpts, Task},
    EframeMain, EguiInspect, DPEQ,
};
use std::time::Duration;

#[derive(EguiInspect, Clone, DPEQ, Default)]
enum Mode {
    #[default]
    Ordinary,
    Power {
        #[inspect(slider, min = 2.0, max = 5.0)]
        p: u32,
    },
}

#[derive(EguiInspect, better_default::Default)]
struct MySummation {
    #[default(100)]
    iters: usize,
    #[default(25)]
    sleep_millis: u64,
    mode: Mode,
    #[inspect(button)]
    begin: bool,
}

impl Task for MySummation {
    type Return = u32;
    /// provide a SynchedStatsOpts object when ready to begin
    fn exec_with_expected_steps(&self) -> Option<SynchedStatsOpts> {
        self.begin
            .then_some(SynchedStatsOpts::HasExpectedLen(self.iters))
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

#[derive(EguiInspect, better_default::Default)]
struct Sleep5 {
    #[inspect(button)]
    begin: bool,
}

impl Task for Sleep5 {
    type Return = ();
    fn exec_with_expected_steps(&self) -> Option<SynchedStatsOpts> {
        self.begin.then_some(Default::default())
    }
    fn on_exec(&mut self, _: Progress) -> Self::Return {
        std::thread::sleep(Duration::from_secs(5));
    }
}

#[derive(Default, EguiInspect, EframeMain)]
pub struct AutoProgressBarTest {
    background_task_1: BackgroundTask<MySummation>,
    background_task_2: BackgroundTask<MySummation>,
    background_task_3: BackgroundTask<Sleep5>,
}
