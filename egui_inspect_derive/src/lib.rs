use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, parse_quote, Data, DataEnum, DeriveInput, Field, Fields, FieldsNamed,
    FieldsUnnamed, GenericParam, Generics, Index, Variant,
};

use darling::{FromDeriveInput, FromField, FromMeta};

mod internal_paths;
mod utils;

#[derive(Clone, Debug, Default, FromField, FromDeriveInput)]
#[darling(attributes(eframe_main), default)]
struct EframeMainAttr {
    title: Option<String>,
    options: Option<String>,
    init: Option<String>,
}

/// Generates a simple, boilerplate [eframe::App] and its main,  for structs which already define
/// how to display themselves through  the [egui_inspect::EguiInspect] trait.
#[proc_macro_derive(EframeMain, attributes(eframe_main))]
pub fn derive_eframe_main(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident.clone();
    let attr = EframeMainAttr::from_derive_input(&input).unwrap();
    let title = attr.title.unwrap_or(ident.to_string());
    // TODO: Accept expressions/tokens rather than strings
    let options: TokenStream = attr.options.unwrap_or("Default::default()".to_string()).parse().unwrap();
    let init: TokenStream = attr.init.unwrap_or(format!("{ident}::default()")).parse().unwrap();

    quote! {
        fn main() -> eframe::Result<()> {
            eframe::run_native(
                #title,
                #options,
                Box::new(|_cc| Ok(Box::new(egui_inspect::quick_app::QuickApp {inner: #init}))),
            )
        }
    }
    .into()
}

/// Derives a impl for PartialEq that only considers an enums discriminant (variants) for
/// EguiInspect derivation (dropdowns) for enums.
#[proc_macro_derive(DPEQ, attributes(__dpeq__))]
pub fn derive_dpeq(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident.clone();

    quote! {
        impl PartialEq for #ident {
            fn eq(&self, other: &Self) -> bool {
                core::mem::discriminant(self) == core::mem::discriminant(other)
            }
        }
    }
    .into()
}

#[derive(Debug, FromField, Default)]
#[darling(attributes(inspect), default)]
struct FieldAttr {
    /// Name of the field to be displayed on UI labels
    name: Option<String>,
    /// Doesn't generate code for the given field
    hide: bool,
    /// Doesn't call mut function for the given field (May be overridden by other params)
    no_edit: bool,
    /// Use slider function for numbers
    slider: bool,
    /// Use logarithmic slider function for numbers
    log_slider: bool,
    /// Min value for numbers
    min: Option<f32>,
    /// Max value for numbers
    max: Option<f32>,
    /// Display mut text on multiple line
    multiline: bool,
    /// Use custom function for non-mut inspect
    custom_func: Option<String>,
    /// Use custom function for mut inspect
    custom_func_mut: Option<String>,
}

#[derive(Clone, Debug, Default, FromDeriveInput)]
#[darling(attributes(inspect), default)]
struct DeriveAttr {
    /// Surround in visual border
    no_border: bool,
    collapsible: bool,
    style: Option<String>,
    on_hover_text: Option<String>,
}

/// Generate [egui_inspect::EguiInspect] impl recursively, based on field impls.
#[proc_macro_derive(EguiInspect, attributes(inspect))]
pub fn derive_egui_inspect(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let attr = DeriveAttr::from_derive_input(&input).unwrap();

    let name = input.ident;

    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let inspect = wrap_in_box_optionally(inspect_data(&input.data, &name, false, attr.clone()), attr.clone());

    let inspect_mut = wrap_in_box_optionally(inspect_data(&input.data, &name, true, attr.clone()), attr.clone());

    quote! {
        impl #impl_generics egui_inspect::EguiInspect for #name #ty_generics #where_clause {
            fn inspect(&self, label: &str, ui: &mut egui::Ui) {
                #inspect
            }
            fn inspect_mut(&mut self, label: &str, ui: &mut egui::Ui) {
                #inspect_mut
            }
        }
    }
    .into()
}

fn wrap_in_box_optionally(inner: TokenStream, attr: DeriveAttr) -> TokenStream {
    if attr.no_border {
        inner
    } else {
        let style_path_str = attr
            .style
            .unwrap_or("egui_inspect::DEFAULT_FRAME_STYLE".to_string());
        let style_path: TokenStream = style_path_str.parse().unwrap();
        quote! {
            #style_path
             .to_frame()
             .show(ui, |ui| {
                #inner
            });
        }
    }
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param
                .bounds
                .push(parse_quote!(egui_inspect::EguiInspect));
        }
    }
    generics
}

fn inspect_data(data: &Data, _struct_name: &Ident, mutable: bool, attr: DeriveAttr) -> TokenStream {
    let t = match *data {
        Data::Struct(ref data) => handle_fields(&data.fields, mutable),
        Data::Enum(ref data_enum) => handle_enum(data_enum, _struct_name, mutable),
        Data::Union(_) => unimplemented!("Unions are not yet supported"),
    };
    let t = if attr.collapsible {
        quote!(ui.collapsing(label, |ui| {
                #t
        });)
    } else {
        quote!( if label!="" {
                   ui.strong(label);
                }
                #t
              )
    };
    if let Some(on_hover_text) = attr.on_hover_text {
        quote!(
        ::egui::Frame::none()
             .show(ui, |ui| {
                #t
            }).response.on_hover_text_at_pointer(#on_hover_text);)
    } else {
        t
    }
}

fn handle_enum(data_enum: &DataEnum, _struct_name: &Ident, mutable: bool) -> TokenStream {
    let variants: Vec<_> = data_enum.variants.iter().collect();
    let name_arms = variants.iter().map(|v| variant_name_arm(v, _struct_name));
    let reflect_variant_name = quote!(
        let current_variant = match self {
            #(#name_arms,)*
        };
    );
    if mutable {
        let combo_opts = variants.iter().map(|v| variant_combo(v, _struct_name));
        let inspect_arms = variants
            .iter()
            .map(|v| variant_inspect_arm(v, _struct_name));
        quote!(
            #reflect_variant_name
            ui.horizontal(|ui| {
                ::egui::ComboBox::new(format!("{self:p}").as_str(), "")
                    .selected_text(current_variant)
                    .show_ui(ui, |ui| {
                        #(#combo_opts;)*
                    });
            });
            match self {
                #(#inspect_arms),*
            };
        )
    } else {
        quote!(
            #reflect_variant_name
            ui.label(current_variant);
            // TODO: readonly held data inspect
        )
    }
}

fn variant_name_arm(variant: &Variant, _struct_name: &Ident) -> TokenStream {
    let ident = &variant.ident;
    match &variant.fields {
        Fields::Named(_) => {
            quote!(#_struct_name::#ident {..} => stringify!(#ident))
        }
        Fields::Unnamed(_) => {
            quote!(#_struct_name::#ident (..) => stringify!(#ident))
        }
        Fields::Unit => {
            quote!(#_struct_name::#ident => stringify!(#ident))
        }
    }
}

fn variant_combo(variant: &Variant, _struct_name: &Ident) -> TokenStream {
    let ident = &variant.ident;
    // TODO: Replace with handle_fields,
    // which would need to take this ident as the base for fields instead of "self".
    match &variant.fields {
        Fields::Named(fields) => {
            let defaults = fields.named.iter().map(|f| {
                let ident = f.ident.clone();
                quote!( #ident: Default::default() )
            });
            quote!(ui.selectable_value(self, 
                                       #_struct_name::#ident { #(#defaults),* }, 
                                       stringify!(#ident)))
        }
        Fields::Unnamed(fields) => {
            let defaults = fields.unnamed.iter().map(|_| quote!(Default::default()));
            quote!(ui.selectable_value(self, #_struct_name::#ident ( #(#defaults),* ), stringify!(#ident)))
        }
        Fields::Unit => {
            quote!(ui.selectable_value(self, #_struct_name::#ident, stringify!(#ident)))
        }
    }
}

fn variant_inspect_arm(variant: &Variant, _struct_name: &Ident) -> TokenStream {
    let ident = &variant.ident;
    match &variant.fields {
        Fields::Named(fields) => {
            let field_idents: Vec<_> = fields
                .clone()
                .named
                .iter()
                .map(|f| {
                    let ident = f.ident.clone();
                    quote!( #ident )
                })
                .collect();
            // TODO: properly refer to trait
            let inspect_fields = fields
                .named
                .iter()
                .map(|f| handle_named_field(f, true, true));
            quote!(#_struct_name::#ident { #(#field_idents),* } => { #(#inspect_fields;)* })
        }
        Fields::Unnamed(_) => {
            unimplemented!("TODO: unnamed")
        }
        Fields::Unit => {
            quote!(#_struct_name::#ident => () )
        }
    }
}

fn handle_fields(fields: &Fields, mutable: bool) -> TokenStream {
    match fields {
        Fields::Named(ref fields) => handle_named_fields(fields, mutable),
        Fields::Unnamed(ref fields) => handle_unnamed_fields(fields, mutable),
        // Empty implementation for unit fields (needed in plain enum variant for instance)
        Fields::Unit => quote!(),
    }
}

fn handle_named_field(f: &Field, mutable: bool, loose: bool) -> TokenStream {
    let attr = FieldAttr::from_field(f).expect("Could not get attributes from field");

    if attr.hide {
        return quote!();
    }

    let mutable = mutable && !attr.no_edit;

    if let Some(ts) = handle_custom_func(f, mutable, &attr) {
        return ts;
    }

    if let Some(ts) = internal_paths::try_handle_internal_path(f, mutable, &attr, loose) {
        return ts;
    }

    utils::get_default_function_call(f, mutable, &attr, loose)
}

fn handle_named_fields(fields: &FieldsNamed, mutable: bool) -> TokenStream {
    let recurse = fields
        .named
        .iter()
        .map(|f| handle_named_field(f, mutable, false));
    quote! {
        #(#recurse)*
    }
}

fn handle_unnamed_fields(fields: &FieldsUnnamed, mutable: bool) -> TokenStream {
    let mut recurse = Vec::new();
    for (i, _) in fields.unnamed.iter().enumerate() {
        let tuple_index = Index::from(i);
        let name = format!("Field {i}");
        let ref_str = if mutable { quote!(&mut) } else { quote!(&) };
        recurse.push(
            quote! { egui_inspect::EguiInspect::inspect(#ref_str self.#tuple_index, #name, ui);},
        );
    }

    let result = quote! {
        #(#recurse)*
    };
    result
}

fn handle_custom_func(field: &Field, mutable: bool, attrs: &FieldAttr) -> Option<TokenStream> {
    let name = &field.ident;

    let name_str = match &attrs.name {
        Some(n) => n.clone(),
        None => name.clone().unwrap().to_string(),
    };

    if let Some(custom_func_mut) = attrs.custom_func_mut.as_ref() {
        if mutable && !attrs.no_edit  {
            let ident = syn::Path::from_string(custom_func_mut)
                .unwrap_or_else(|_| panic!("Could not find function: {}", custom_func_mut));
            return Some(quote_spanned! { field.span() => {
                    #ident(&mut self.#name, &#name_str, ui);
                }
            });
        }
    }


    if let Some(custom_func) = attrs.custom_func.as_ref() {
        // TODO: Applicable conditions?
        let ident = syn::Path::from_string(custom_func)
            .unwrap_or_else(|_| panic!("Could not find function: {}", custom_func));
        return Some(quote_spanned! { field.span() => {
                #ident(&self.#name, &#name_str, ui);
            }
        });
    }

    None
}
