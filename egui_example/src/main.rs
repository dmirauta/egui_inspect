use egui::{Color32, Stroke, Style};
use egui_inspect::{EguiInspect, FrameStyle, InspectNumber, DEFAULT_FRAME_STYLE};

use eframe::{egui, NativeOptions};
use egui_inspect_wrap::VisualsUi;

#[derive(EguiInspect)]
#[inspect(no_border)]
struct MyApp {
    edit_style: bool,
    #[inspect(hide)]
    visuals: VisualsUi,
    #[inspect(no_edit)]
    string: String,
    #[inspect(multiline)]
    code: String,
    #[inspect(min = 12.0, max = 53.0)]
    unsigned32: u32,
    #[inspect(hide)]
    _skipped: bool,
    strings: Vec<String>,
    #[inspect(no_edit)]
    raw_string: &'static str,
    #[inspect(slider, min = -43.0, max = 125.0)]
    float64: f64,
    #[inspect(log_slider, min = -43.0, max = 125.0)]
    log_varied_float64: f64,
    #[inspect(name = "A proper field name")]
    ugly_internal_field_name: u16,
    #[inspect(name = "A tuple struct")]
    ugly_internal_field_name_2: Salut,
    #[inspect(name = "An optional struct with three floats")]
    opt_vector_struct: Option<Vector>,
    my_enum: MyEnum,
    #[inspect(no_edit)]
    my_enum_readonly: MyEnum,
    #[inspect(custom_func_mut = "custom_bool_inspect")]
    custom_bool: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            edit_style: false,
            visuals: Default::default(),
            string: "I am a single line string".to_owned(),
            code: "Hello\nI\nam\na\nmultiline\nstring".to_owned(),
            _skipped: true,
            unsigned32: 42,
            strings: vec![
                "Bonjour".to_string(),
                "Voici une liste de string".to_string(),
                "Avec plusieurs strings".to_string(),
            ],
            raw_string: "YetAnotherString",
            float64: 6.0,
            log_varied_float64: 6.0,
            ugly_internal_field_name: 16,
            ugly_internal_field_name_2: Salut(50, 123.45),
            opt_vector_struct: Some(Vector {
                x: 10.0,
                y: 20.0,
                z: 30.0,
            }),
            my_enum: MyEnum::AnOptionWithStructData {
                vec: Default::default(),
                salut: Default::default(),
            },
            my_enum_readonly: MyEnum::AnOptionWithNoData,
            custom_bool: false,
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
struct Salut(i32, f32);

#[derive(EguiInspect, PartialEq, Default)]
struct Vector {
    #[inspect(name = "X axis")]
    x: f32,
    #[inspect(name = "Y axis")]
    y: f32,
    #[inspect(name = "Z axis")]
    z: f32,
}

#[derive(EguiInspect, PartialEq)]
enum MyEnum {
    AnOptionWithNoData,
    AnOptionWithStructData {
        vec: Vector,
        #[inspect(name = "salut_field")]
        salut: Salut,
    },
}

fn custom_bool_inspect(boolean: &mut bool, label: &'static str, ui: &mut egui::Ui) {
    ui.label("Overriden inspect for the following bool");
    boolean.inspect(label, ui);
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.columns(2, |cols| {
                    let ui = &mut cols[0];
                    self.inspect_mut("Test App", ui);
                    // or readonly
                    // self.inspect("Test App", ui);

                    // more ui...
                    let salut = Salut(1, 2.0);
                    salut.inspect("label for tuple struct", ui);

                    // logic based on set values...
                    // let c = self.float64 + self.log_varied_float64;
                    // should ideally only set when changing
                    if self.edit_style {
                        self.visuals
                            .inspect_mut("visuals (egui style)", &mut cols[1]);

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
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}
