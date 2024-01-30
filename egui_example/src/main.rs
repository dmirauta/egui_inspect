use std::collections::{BTreeMap, HashMap};

use egui::{Color32, Stroke, Style};
use egui_inspect::{EguiInspect, FrameStyle, InspectNumber, DEFAULT_FRAME_STYLE};

use eframe::{egui, NativeOptions};
use egui_inspect_wrap::VisualsUi;
use egui_plot::{Line, Plot};

#[derive(EguiInspect)]
struct Primitives {
    #[inspect(no_edit)]
    string: String,
    #[inspect(multiline)]
    code: String,
    #[inspect(min = 12.0, max = 53.0)]
    unsigned32: u32,
    #[inspect(hide)]
    _skipped: bool,
    #[inspect(custom_func_mut = "custom_bool_inspect")]
    custom_bool: bool,
    raw_string: &'static str,
    #[inspect(slider, min = -43.0, max = 125.0)]
    float64: f64,
    #[inspect(log_slider, min = -43.0, max = 125.0)]
    log_varied_float64: f64,
}

impl Default for Primitives {
    fn default() -> Self {
        Self {
            string: "I am a single line string".to_owned(),
            code: "Hello\nI\nam\na\nmultiline\nstring".to_owned(),
            _skipped: true,
            custom_bool: true,
            unsigned32: 42,
            raw_string: "YetAnotherString",
            float64: 6.0,
            log_varied_float64: 6.0,
        }
    }
}

#[derive(EguiInspect)]
struct Containers {
    #[inspect(name = "vector")]
    an_ugly_internal_name: Vec<[f64; 2]>,
    string_map: HashMap<String, Custom>,
    ordered_string_map: BTreeMap<String, u32>,
}

impl Default for Containers {
    fn default() -> Self {
        let mut string_map = HashMap::new();
        string_map.insert("one thing".to_string(), Custom(50, 123.45));
        string_map.insert("another".to_string(), Custom(200, 13.45));

        let mut ordered_string_map = BTreeMap::new();
        for (i, key) in [
            "Bonjour".to_string(),
            "Voici une liste de string".to_string(),
            "Avec plusieurs strings".to_string(),
        ]
        .into_iter()
        .enumerate()
        {
            ordered_string_map.insert(key, (i as u32) * 5);
        }

        Self {
            an_ugly_internal_name: vec![[1.0, 28.0], [2.0, 15.0], [4.0, 20.0], [8.0, 3.0]],
            string_map,
            ordered_string_map,
        }
    }
}

#[derive(EguiInspect, Default)]
#[inspect(no_border)]
struct MyApp {
    edit_style: bool,
    #[inspect(hide)]
    visuals: VisualsUi,
    some_primitives: Primitives,
    containers: Containers,
    fancy_enum: MyEnum,
}

impl MyApp {
    fn new() -> Self {
        Self {
            fancy_enum: MyEnum::VariantWithStructData {
                a_plot: APlot {
                    stroke: Stroke {
                        width: 5.0,
                        color: Color32::from_rgba_unmultiplied(41, 91, 37, 55),
                    },
                    xy: vec![],
                },
                optional_data: Some(0),
            },
            ..Default::default()
        }
    }
}

static CUSTOM_BOX: FrameStyle = FrameStyle {
    stroke: Stroke {
        width: 2.0,
        color: Color32::RED,
    },
    ..DEFAULT_FRAME_STYLE
};

#[derive(EguiInspect, PartialEq, Default)]
#[inspect(
    style = "crate::CUSTOM_BOX",
    collapsible,
    on_hover_text = "show when hovering"
)]
struct Custom(i32, f32);

#[derive(Default, PartialEq)]
struct APlot {
    stroke: Stroke,
    xy: Vec<[f64; 2]>,
}

impl EguiInspect for APlot {
    fn inspect(&self, _label: &str, _ui: &mut egui::Ui) {
        todo!();
    }

    fn inspect_mut(&mut self, label: &str, ui: &mut egui::Ui) {
        ui.label(label);
        self.stroke.inspect_mut("stroke", ui);
        Plot::new(label).height(200.0).show(ui, |pui| {
            pui.line(Line::new(self.xy.clone()).stroke(self.stroke))
        });
    }
}

#[derive(EguiInspect, PartialEq, Default)]
enum MyEnum {
    #[default]
    PlainVariant,
    DifferentPlainVariant,
    VariantWithStructData {
        #[inspect(name = "Mirroring data in containers.vector, try editing it!")]
        a_plot: APlot,
        optional_data: Option<usize>,
    },
}

fn custom_bool_inspect(boolean: &mut bool, label: &'static str, ui: &mut egui::Ui) {
    ui.label("Overriden inspect for the following bool");
    boolean.inspect(label, ui);
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let MyEnum::VariantWithStructData { a_plot, .. } = &mut self.fancy_enum {
            a_plot.xy = self.containers.an_ugly_internal_name.clone();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.columns(2, |cols| {
                    let ui = &mut cols[0];
                    // display own derived ui mutably
                    self.inspect_mut("", ui);
                    // or readonly
                    // self.inspect("Test App", ui);

                    ui.menu_button("inspected label", |ui| {
                        "A simple label, which demands required space.".inspect("", ui);
                    });
                    ui.menu_button("ui.label", |ui| {
                        ui.label("A label with potentially odd kerning.");
                    });

                    // conditionally showing other ui based on interactions
                    if self.edit_style {
                        self.visuals
                            .inspect_mut("visuals (egui style)", &mut cols[1]);

                        // should ideally only set when changing
                        ctx.set_style(Style {
                            visuals: self.visuals.clone().into(),
                            ..Default::default()
                        })
                    }
                });
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "My egui App",
        NativeOptions::default(),
        Box::new(|_cc| Box::new(MyApp::new())),
    )
}
