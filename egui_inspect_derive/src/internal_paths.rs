use crate::utils::get_path_str;
use crate::FieldAttr;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::Field;

pub(crate) fn path_is_internally_handled(path_str: &String) -> bool {
    path_str == "f32"
        || path_str == "f64"
        || path_str == "u8"
        || path_str == "i8"
        || path_str == "u16"
        || path_str == "i16"
        || path_str == "u32"
        || path_str == "i32"
        || path_str == "u64"
        || path_str == "i64"
        || path_str == "usize"
        || path_str == "isize"
        || path_str == "bool"
        || path_str == "String"
        || path_str == "str"
}

pub(crate) fn try_handle_internal_path(
    field: &Field,
    mutable: bool,
    attrs: &FieldAttr,
    loose_field: bool,
) -> Option<TokenStream> {
    let path_str = get_path_str(&field.ty);

    path_str.as_ref()?;
    let path_str = path_str.unwrap();

    if !path_is_internally_handled(&path_str) {
        return None;
    }

    match path_str.as_str() {
        "f64" | "f32" | "u8" | "i8" | "u16" | "i16" | "u32" | "i32" | "u64" | "i64" | "usize"
        | "isize" => handle_number_path(field, mutable, attrs, loose_field),
        "String" => handle_string_path(field, mutable, attrs),
        _ => None,
    }
}

fn handle_number_path(
    field: &Field,
    mutable: bool,
    attrs: &FieldAttr,
    loose_field: bool,
) -> Option<TokenStream> {
    let name = &field.ident;

    let name_str = match &attrs.name {
        Some(n) => n.clone(),
        None => name.clone().unwrap().to_string(),
    };

    let no_edit = attrs.no_edit;
    let slider = attrs.slider;
    let log_slider = attrs.log_slider;
    let min = attrs.min;
    let max = attrs.max;

    if no_edit {
        return None;
    }

    let base = if loose_field {
        quote!(#name)
    } else {
        quote!(self.#name)
    };

    if mutable && !slider && !log_slider {
        match (min, max) {
            (Some(mi), Some(ma)) => {
                return Some(quote_spanned! {field.span() => {
                    egui_inspect::InspectNumber::inspect_with_drag_value(&mut #base, &#name_str, ui, #mi, #ma);
                    }
                });
            }
            _ => {
                return Some(quote_spanned! {field.span() => {
                    #base.inspect_mut(&#name_str, ui);
                    }
                });
            }
        }
    }

    let min = min.unwrap_or(0.0);
    let max = max.unwrap_or(100.0);

    if mutable && log_slider {
        return Some(quote_spanned! {field.span() => {
        // egui_inspect::InspectNumber::inspect_with_log_slider(&mut self.#name, &#name_str, ui, #min, #max);
        #base.inspect_with_log_slider(&#name_str, ui, #min, #max);
            }
        });
    }
    if mutable && slider {
        return Some(quote_spanned! {field.span() => {
        // egui_inspect::InspectNumber::inspect_with_slider(&mut self.#name, &#name_str, ui, #min, #max);
        #base.inspect_with_slider(&#name_str, ui, #min, #max);
            }
        });
    }

    None
}

fn handle_string_path(field: &Field, mutable: bool, attrs: &FieldAttr) -> Option<TokenStream> {
    let name = &field.ident;

    let name_str = match &attrs.name {
        Some(n) => n.clone(),
        None => name.clone().unwrap().to_string(),
    };

    let multiline = attrs.multiline;
    let no_edit = attrs.no_edit;

    if no_edit {
        return None;
    }

    if mutable && multiline {
        return Some(quote_spanned! {field.span() => {
            egui_inspect::base_type_inspect::str_inspect_mut_multiline(&mut self.#name, &#name_str, ui);
            }
        });
    }
    if mutable && !multiline {
        return Some(quote_spanned! {field.span() => {
            egui_inspect::base_type_inspect::str_inspect_mut_singleline(&mut self.#name, &#name_str, ui);
            }
        });
    }

    None
}
