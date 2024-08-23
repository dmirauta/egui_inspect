use std::collections::{BTreeMap, HashMap};

use egui_inspect::egui::{self, Color32, Stroke, Style};
use egui_inspect::search_select::SearchSelection;
use egui_inspect::{EframeMain, EguiInspect, FrameStyle, InspectNumber, DEFAULT_FRAME_STYLE};

use egui_inspect_wrap::VisualsUi;
use egui_plot::{Line, Plot};

#[derive(EguiInspect)]
#[inspect(collapsible)]
struct Primitives {
    #[inspect(no_edit)]
    string: String,
    #[inspect(multiline)]
    code: String,
    unsigned32: u32,
    #[inspect(hide)]
    _skipped: bool,
    #[inspect(custom_func_mut = "custom_bool_inspect")]
    custom_bool: bool,
    raw_string: &'static str,
    #[inspect(slider = false, min = 10.0, max = 125.0)]
    usize: usize,
    #[inspect(slider, min = -43.0, max = 125.0)]
    isize: isize,
    #[inspect(log_slider, min = -43.0, max = 125.0)]
    log_varied_float64: f64,
}

fn custom_bool_inspect(boolean: &mut bool, label: &'static str, ui: &mut egui::Ui) {
    ui.label("Overriden inspect for the following bool");
    boolean.inspect(label, ui);
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
            usize: 20,
            isize: 6,
            log_varied_float64: 6.0,
        }
    }
}

#[derive(EguiInspect)]
#[inspect(collapsible)]
struct Containers {
    #[inspect(name = "vector")]
    an_ugly_internal_name: Vec<[f64; 2]>,
    string_map: HashMap<String, Custom>,
    ordered_string_map: BTreeMap<String, u32>,
    a_wrapped_searchable_vec: SearchSelection<Custom>,
}

impl Default for Containers {
    fn default() -> Self {
        let string_map = [
            ("one thing".into(), Custom(50, 123.45)),
            ("another".into(), Custom(200, 13.45)),
        ]
        .into_iter()
        .collect();

        let ordered_string_map = [
            "Bonjour".into(),
            "Voici une liste de string".into(),
            "Avec plusieurs strings".into(),
        ]
        .into_iter()
        .enumerate()
        .map(|(i, key)| (key, (i as u32) * 5))
        .collect();

        let an_ugly_internal_name = vec![[1.0, 28.0], [2.0, 15.0], [4.0, 20.0], [8.0, 3.0]];
        let a_wrapped_searchable_vec: Vec<_> = an_ugly_internal_name
            .iter()
            .map(|[i, j]| Custom(*i as i32, *j as f32))
            .collect();

        Self {
            an_ugly_internal_name,
            string_map,
            ordered_string_map,
            a_wrapped_searchable_vec: SearchSelection::new(a_wrapped_searchable_vec, |c| {
                format!("{c:?}")
            }),
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

#[derive(EguiInspect, PartialEq, Default, Debug)]
#[inspect(
    frame_style = "crate::CUSTOM_BOX",
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
    fn inspect(&self, label: &str, ui: &mut egui::Ui) {
        ui.label(label);
        Plot::new(label).height(200.0).show(ui, |pui| {
            pui.line(Line::new(self.xy.clone()).stroke(self.stroke))
        });
    }

    fn inspect_mut(&mut self, label: &str, ui: &mut egui::Ui) {
        self.stroke.inspect_mut("stroke", ui);
        self.inspect(label, ui);
    }
}

#[derive(EguiInspect, PartialEq, Default)]
#[inspect(collapsible)]
enum MyEnum {
    #[default]
    PlainVariant,
    UnnamedFieldVariant(usize, String),
    VariantWithStructData {
        #[inspect(name = "Mirroring data in containers.vector, try editing it!")]
        a_plot: APlot,
        optional_data: Option<usize>,
    },
}

#[derive(EguiInspect, Default, EframeMain)]
#[inspect(no_border)]
#[eframe_main(title = "My egui App", init = "MyApp::new()", no_eframe_app_derive)]
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

impl egui_inspect::eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut egui_inspect::eframe::Frame) {
        if let MyEnum::VariantWithStructData { a_plot, .. } = &mut self.fancy_enum {
            a_plot.xy.clone_from(&self.containers.an_ugly_internal_name);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.columns(2, |cols| {
                    // display own derived ui mutably
                    self.inspect_mut("", &mut cols[0]);

                    // conditionally showing other ui based on interactions
                    if self.edit_style {
                        self.visuals
                            .inspect_mut("visuals (egui style)", &mut cols[1]);

                        // TODO: should ideally only set when changing
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
