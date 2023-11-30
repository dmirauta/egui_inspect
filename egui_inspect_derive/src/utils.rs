use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::Type::{Path, Reference};
use syn::{Field, Type};

use crate::FieldAttr;

pub fn get_path_str(type_path: &Type) -> Option<String> {
    match type_path {
        Path(type_path) => {
            let ident = type_path.path.get_ident();
            if let Some(name) = ident {
                return Some(name.to_string());
            }
            return None;
        }
        Reference(type_ref) => get_path_str(&*type_ref.elem),
        _ => Some("".to_string()),
    }
}

pub(crate) fn get_default_function_call(
    field: &Field,
    mutable: bool,
    attrs: &FieldAttr,
    loose_field: bool,
) -> TokenStream {
    let name = &field.ident;

    let name_str = match &attrs.name {
        Some(n) => n.clone(),
        None => name.clone().unwrap().to_string(),
    };

    let base = if loose_field {
        quote!(#name)
    } else {
        if mutable {
            quote!(&mut self.#name)
        } else {
            quote!(&self.#name)
        }
    };

    return if mutable {
        quote_spanned! {field.span() => {
            egui_inspect::EguiInspect::inspect_mut(#base, &#name_str, ui);
            }
        }
    } else {
        quote_spanned! {field.span() => {
            egui_inspect::EguiInspect::inspect(#base, &#name_str, ui);
            }
        }
    };
}
