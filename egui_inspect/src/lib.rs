//! # egui_inspect
//! This crate expose macros and traits to generate boilerplate code
//! for structs inspection and edition.
//!
//! Basic usage would be
//! ```
//! # use egui_inspect::*;
//! #[derive(EguiInspect)]
//! struct MyApp {
//!     #[inspect(no_edit)]
//!     string: String,
//!     #[inspect(multiline)]
//!     code: String,
//!     #[inspect(min = 12.0, max = 53.0)]
//!     unsigned32: u32,
//!     #[inspect(hide)]
//!     skipped: bool,
//!     #[inspect(custom_func_mut = "custom_bool_inspect")]
//!     boolean: bool,
//!     #[inspect(no_edit)]
//!     raw_string: &'static str,
//!     #[inspect(slider, min = -43.0, max = 125.0)]
//!     float64: f32,
//!     #[inspect(name = "A proper field name")]
//!     ugly_internal_field_name: u16,
//! }
//!
//! fn custom_bool_inspect(boolean: &mut bool, label: &'static str, ui: &mut egui::Ui) {
//!    ui.label("C'EST LA GIGA FONCTION CUSTOM WÉ");
//!    boolean.inspect(label, ui);
//! }
//!
//! fn main() {
//!     let app = MyApp::default();
//!     app.inspect("My App", &ui); // here `ui` would be some `&mut egui::Ui`
//! }
//! ```
//!
//! You can add attributes to structures field.
//! Currently supported attributes are defined in the struct AttributeArgs of egui_inspect_derive
//!
//! Here is a list of supported attributes.
//! It might not be up to date, it's better to check directly AttributeArgs declaration
//!
//! - `name` *(String)*: Use custom label for the given field instead of the internal field name
//! - `hide` *(bool)*: If true, doesn't generate code for the given field
//! - `no_edit` *(bool)*: If true, never call mut function for the given field (May be overridden by other params)
//! - `slider` *(bool)*: If true, use a slider when inspecting numbers (`mut` only)
//! - `min` *(f32)*: Min value for inspecting numbers (`mut` only)
//! - `max` *(f32)*: Max value for inspecting numbers (`mut` only)
//! - `multiline` *(bool)*: If true, display the text on multiple lines (`mut` only)
//! - `custom_func` *(String)*: Use custom function for non-mut inspect (Evaluate the string as a function path)
//! - `custom_func_mut` *(String)*: Use custom function for mut inspect (Evaluate the string as a function path)
//!

use egui::{Frame, Margin, Stroke};
/// See also [EguiInspect]
pub use egui_inspect_derive::*;

/// Base trait to automatically inspect structs
pub trait EguiInspect {
    fn inspect(&self, label: &str, ui: &mut egui::Ui);
    fn inspect_mut(&mut self, label: &str, ui: &mut egui::Ui);
}

pub trait InspectNumber {
    fn inspect_with_slider(&mut self, label: &str, ui: &mut egui::Ui, min: f32, max: f32);
    fn inspect_with_log_slider(&mut self, label: &str, ui: &mut egui::Ui, min: f32, max: f32);
    fn inspect_with_drag_value(&mut self, label: &str, ui: &mut egui::Ui);
}

pub trait InspectString {
    fn inspect_mut_multiline(&mut self, label: &str, ui: &mut egui::Ui);
    fn inspect_mut_singleline(&mut self, label: &str, ui: &mut egui::Ui);
}

pub struct FrameStyle {
    pub inner_margin: Margin,
    pub outer_margin: Margin,
    pub stroke: Stroke,
}

pub static DEFAULT_FRAME_STYLE: FrameStyle = FrameStyle {
    inner_margin: Margin {
        left: 5.0,
        right: 5.0,
        bottom: 5.0,
        top: 5.0,
    },
    outer_margin: Margin {
        left: 1.0,
        right: 1.0,
        bottom: 1.5,
        top: 1.5,
    },
    stroke: Stroke {
        width: 0.7,
        color: egui::Color32::GRAY,
    },
};

impl FrameStyle {
    pub fn to_frame(&self) -> Frame {
        Frame::none()
            .inner_margin(self.inner_margin)
            .outer_margin(self.outer_margin)
            .stroke(self.stroke)
    }
}

pub mod base_type_inspect;
