//! Provides some facilities for wrapping around external types to visualise them, and implements
//! wrapping for [`egui::Visuals`] to easily preview styling changes live

use egui::{
    epaint::Shadow,
    style::{Selection, WidgetVisuals, Widgets},
    Color32, Rounding, Stroke, Visuals,
};
use egui_inspect::EguiInspect;

/// defines struct in second arg to shadow that in first, a field can be of its OriginalStruct, or
/// Into<OriginalStruct>
#[macro_export]
macro_rules! shadow_struct {
    ($shadowed: ident, $shadow: ident, $($field: ident: $type: ty),*) => {
        #[derive(EguiInspect, Clone)]
        #[inspect(collapsible)]
        pub struct $shadow {
            $(pub $field: $type,)*
        }

        impl From<$shadowed> for $shadow {
            fn from($shadowed {$($field,)*}: $shadowed) -> Self {
                $shadow {
                    $($field: $field.into(),)*
                }
            }
        }

        impl From<$shadow> for $shadowed {
            fn from($shadow {$($field,)*}: $shadow) -> Self {
                $shadowed {
                    $($field: $field.into(),)*
                }
            }
        }
    };
}

/// similar to shadow_struct!(...), but need not shadow all fields
#[macro_export]
macro_rules! shadow_struct_w_default {
    ($shadowed: ident, $shadow: ident, $($field: ident: $type: ty),*) => {
        #[derive(EguiInspect, Clone)]
        #[inspect(collapsible)]
        pub struct $shadow {
            $(pub $field: $type,)*
        }

        impl From<$shadowed> for $shadow {
            fn from($shadowed {$($field,)* ..}: $shadowed) -> Self {
                $shadow {
                    $($field: $field.into(),)*
                }
            }
        }

        impl From<$shadow> for $shadowed {
            fn from($shadow {$($field,)*}: $shadow) -> Self {
                $shadowed {
                    $($field: $field.into(),)*
                    ..Default::default()
                }
            }
        }

        impl Default for $shadow {
            fn default() -> Self {
                $shadowed::default().into()
            }
        }

    };
}

shadow_struct!(Rounding, RoundingUi, nw: f32, ne: f32, sw: f32, se: f32);
shadow_struct!(Shadow, ShadowUi, extrusion: f32, color: Color32);
shadow_struct!(Selection, SelectionUi, bg_fill: Color32, stroke: Stroke);
shadow_struct!(WidgetVisuals, WidgetVisualsUi, bg_fill: Color32, weak_bg_fill: Color32, bg_stroke: Stroke, rounding: RoundingUi, fg_stroke: Stroke, expansion: f32);
shadow_struct!(Widgets, WidgetsUi, noninteractive: WidgetVisualsUi, inactive: WidgetVisualsUi, hovered: WidgetVisualsUi, active: WidgetVisualsUi, open: WidgetVisualsUi);

// VisualsUi wraps around [`egui::Visuals`]
shadow_struct_w_default!(Visuals, VisualsUi, dark_mode: bool,
    override_text_color: Option<Color32>,
    widgets: WidgetsUi,
    selection: SelectionUi,
    hyperlink_color: Color32,
    faint_bg_color: Color32,
    extreme_bg_color: Color32,
    code_bg_color: Color32,
    warn_fg_color: Color32,
    error_fg_color: Color32,
    window_rounding: RoundingUi,
    window_shadow: ShadowUi,
    window_fill: Color32,
    window_stroke: Stroke,
    menu_rounding: RoundingUi,
    panel_fill: Color32,
    popup_shadow: ShadowUi,
    resize_corner_size: f32,
    text_cursor: Stroke,
    text_cursor_preview: bool,
    clip_rect_margin: f32,
    button_frame: bool,
    collapsing_header_frame: bool,
    indent_has_left_vline: bool,
    striped: bool,
    slider_trailing_fill: bool,
    image_loading_spinners: bool);
