use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, parse_quote, Data, DataEnum, DeriveInput, Field, Fields, FieldsNamed, FieldsUnnamed, GenericParam, Generics, Index, Variant
};

use darling::{FromDeriveInput, FromField, FromMeta};

mod internal_paths;
mod utils;

// TODO: post_inspect would be more useful as an EguiInspect attribute, or on both?
#[derive(Clone, Debug, Default, FromField, FromDeriveInput)]
#[darling(attributes(eframe_main), default)]
struct EframeMainAttr {
    /// The title of the application (defaults to struct name)
    title: Option<String>,
    /// How to generate `eframe::NativeOptions` (defaults to `NativeOptions::default()`)
    options: Option<String>,
    /// How to initialise the Application object (defaults to `Self::Default()`)
    init: Option<String>,
    /// Code to run after the `Self::egui_inspect` call within the generated `Eframe::App::update`
    post_inspect: Option<String>,
    /// If false, manually impement the `Eframe::App` trait
    no_eframe_app_derive: bool,
}

// TODO: run docstring example as test to ensure it does not regress
/// Generates a simple, boilerplate [eframe::App] and its main,  for structs which already define
/// how to display themselves through  the [egui_inspect::EguiInspect] trait.
///
/// Minimal example:
/// ```
/// use egui_inspect::{EguiInspect, EframeMain};
///
/// #[derive(Default, EguiInspect, EframeMain)]
/// #[inspect(no_border)]
/// #[eframe_main(title = "My Application")]
/// struct MyApp {}
/// ```
/// In this case creating an empty application window.
///
/// Longer example:
/// ```
///use egui_inspect::{egui, EframeMain, EguiInspect};
///
///#[derive(Default, Debug, EguiInspect)]
///struct Options {
///    add_input: bool,
///    double: bool,
///}
///
///#[derive(Default, Debug, EguiInspect, EframeMain)]
///#[eframe_main(post_inspect = "self.post_inspect(ui);")]
///struct ReflectMe {
///    #[inspect(slider, min = 10.0, max = 20.0)]
///    input: u16,
///    #[inspect(hide)]
///    internal: f32,
///    #[inspect(name = "Some options")]
///    options: Options,
///}
///
///impl ReflectMe {
///    /// Runs after the egui_inspect call that is used by the generated Eframe::App::update
///    fn post_inspect(&mut self, ui: &mut egui::Ui) {
///        if ui.button("ready").clicked() {
///            if self.options.add_input {
///                self.internal += self.input as f32;
///            }
///            if self.options.double {
///                self.internal *= 2.0;
///            }
///            println!("self: {self:?}");
///        }
///    }
///}
/// ```
///
/// The options accepted by `#[eframe_main()]` consist of the fields of:
/// ```
///struct EframeMainAttr {
///    /// The title of the application (defaults to struct name)
///    title: Option<String>,
///    /// How to generate `eframe::NativeOptions` (defaults to `NativeOptions::default()`)
///    options: Option<String>,
///    /// How to initialise the Application object (defaults to `Self::Default()`)
///    init: Option<String>,
///    /// Code to run after the `Self::egui_inspect` call within the generated `Eframe::App::update`
///    post_inspect: Option<String>,
///    /// If false, manually impement the `Eframe::App` trait
///    no_eframe_app_derive: bool,
///}
///```
///
/// When compiling for WASM, one may need to append the following to their Cargo.toml:
///
/// ```toml
/// [target.'cfg(target_arch = "wasm32")'.dependencies]
/// wasm-bindgen-futures = "0.4"
///
/// [profile.release]
/// opt-level = 2 # fast and small wasm
///
/// # Optimize all dependencies even in debug builds:
/// [profile.dev.package."*"]
/// opt-level = 2
/// ```
///
/// see https://github.com/emilk/eframe_template for more on this.
#[proc_macro_derive(EframeMain, attributes(eframe_main))]
pub fn derive_eframe_main(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident.clone();
    let attr = EframeMainAttr::from_derive_input(&input).unwrap();
    let title = attr.title.unwrap_or(ident.to_string());
    // TODO: Accept expressions/tokens rather than strings
    let options: TokenStream = attr.options.unwrap_or("Default::default()".to_string()).parse().unwrap();
    let init: TokenStream = attr.init.unwrap_or(format!("{ident}::default()")).parse().unwrap();
    let post_inspect: TokenStream = attr.post_inspect.unwrap_or(String::new()).parse().unwrap();
    let eframe_app_derive = match attr.no_eframe_app_derive {
        true => quote!{
        },
        false => quote! {
            impl egui_inspect::eframe::App for #ident {
                fn update(&mut self, ctx: &egui_inspect::egui::Context, _frame: &mut egui_inspect::eframe::Frame) {
                    egui_inspect::egui::CentralPanel::default().show(ctx, |ui| {
                        egui_inspect::egui::ScrollArea::both().show(ui, |ui| {
                            self.inspect_mut("", ui);
                            #post_inspect
                        })
                    });
                }
            }
        },
    };

    quote! {
        #eframe_app_derive

        #[cfg(not(target_arch = "wasm32"))]
        fn main() -> egui_inspect::eframe::Result<()> {
            egui_inspect::eframe::run_native(
                #title,
                #options,
                Box::new(|_cc| Ok(Box::new(#init))),
            )
        }

        #[cfg(target_arch = "wasm32")]
        fn main() {
            use wasm_bindgen_futures::wasm_bindgen::JsCast;
            wasm_bindgen_futures::spawn_local(async {
                let document = egui_inspect::eframe::web_sys::window()
                    .expect("No window")
                    .document()
                    .expect("No document");

                let canvas = document
                    .get_element_by_id("the_canvas_id")
                    .expect("Failed to find the_canvas_id")
                    .dyn_into::<egui_inspect::eframe::web_sys::HtmlCanvasElement>()
                    .expect("the_canvas_id was not a HtmlCanvasElement");

                let start_result = egui_inspect::eframe::WebRunner::new()
                    .start(
                        canvas,
                        #options,
                        Box::new(|_cc| Ok(Box::new(#init))),
                    )
                    .await;

                // Remove the loading text and spinner:
                if let Some(loading_text) = document.get_element_by_id("loading_text") {
                    match start_result {
                        Ok(_) => {
                            loading_text.remove();
                        }
                        Err(e) => {
                            loading_text.set_inner_html(
                                "<p> The app has crashed. See the developer console for details. </p>",
                            );
                            panic!("Failed to start eframe: {e:?}");
                        }
                    }
                }
            });
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
    /// Use a button for a bool field (will only be set to true during the frame that it is pressed)
    button: bool,
}

#[derive(Clone, Debug, Default, FromDeriveInput)]
#[darling(attributes(inspect), default)]
struct DeriveAttr {
    /// Surround in visual border
    no_border: bool,
    collapsible: bool,
    /// layout field inspects horizontally rather than vertically
    horiz: bool,
    frame_style: Option<String>,
    on_hover_text: Option<String>,
    /// If a parameter is only used in hidden fields, it does not need to be EguiInspect.
    no_trait_bound: Option<String>,
    // TODO: Multiple exempt parameters, ideally automatically detected.
}

// TODO: keep structs in sync after changes, or just reference them by tag and use jump to
// definition via an lsp for help?
/// Generate [egui_inspect::EguiInspect] impl recursively, based on field impls.
///
/// Example:
/// ```
///use egui_inspect::{egui, EframeMain, EguiInspect};
///
///#[derive(EguiInspect)]
///struct Options {
///    add_input: bool,
///    double: bool,
///}
///
///#[derive(EguiInspect)]
///#[inspect(collapsible)]
///struct ReflectMe {
///    #[inspect(slider, min = 10.0, max = 20.0)]
///    input: u16,
///    #[inspect(hide)]
///    internal: f32,
///    #[inspect(name = "Some options")]
///    options: Options,
///}
/// ```
///
/// The structure may be annotated with options defined by the fields of
/// ```
///struct DeriveAttr {
///    /// Surround in visual border
///    no_border: bool,
///    collapsible: bool,
///    /// layout field inspects horizontally rather than vertically
///    horiz: bool,
///    frame_style: Option<String>,
///    on_hover_text: Option<String>,
///    /// If a parameter is only used in hidden fields, it does not need to be EguiInspect.
///    no_trait_bound: Option<String>,
///}
/// ```
///
/// And its fields by the fields of:
/// ```
///struct FieldAttr {
///    /// Name of the field to be displayed on UI labels
///    name: Option<String>,
///    /// Doesn't generate code for the given field
///    hide: bool,
///    /// Doesn't call mut function for the given field (May be overridden by other params)
///    no_edit: bool,
///    /// Use slider function for numbers
///    slider: bool,
///    /// Use logarithmic slider function for numbers
///    log_slider: bool,
///    /// Min value for numbers
///    min: Option<f32>,
///    /// Max value for numbers
///    max: Option<f32>,
///    /// Display mut text on multiple line
///    multiline: bool,
///    /// Use custom function for non-mut inspect
///    custom_func: Option<String>,
///    /// Use custom function for mut inspect
///    custom_func_mut: Option<String>,
///}
/// ```
#[proc_macro_derive(EguiInspect, attributes(inspect))]
pub fn derive_egui_inspect(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let attr = DeriveAttr::from_derive_input(&input).unwrap();

    let name = input.ident;

    let mut ignore_list = vec![];
    if let Some(s) = &attr.no_trait_bound {
        ignore_list.push(s.as_str());
    }
    let generics = add_trait_bounds(input.generics, ignore_list);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let inspect = inspect_data(&input.data, &name, false, &attr);
    let inspect_mut = inspect_data(&input.data, &name, true, &attr);

    quote! {
        impl #impl_generics egui_inspect::EguiInspect for #name #ty_generics #where_clause {
            fn inspect(&self, label: &str, ui: &mut egui_inspect::egui::Ui) {
                #inspect
            }
            fn inspect_mut(&mut self, label: &str, ui: &mut egui_inspect::egui::Ui) {
                #inspect_mut
            }
        }
    }
    .into()
}

fn add_trait_bounds(mut generics: Generics, ignore_list: Vec<&str>) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(type_param) = param {
            if !ignore_list.contains(&type_param.ident.to_string().as_str()) {
                type_param
                    .bounds
                    .push(parse_quote!(egui_inspect::EguiInspect));
            }
        }
    }
    generics
}

fn inspect_data(data: &Data, _struct_name: &Ident, mutable: bool, attr: &DeriveAttr) -> TokenStream {
    let mut inner = match data {
        Data::Struct(data) => handle_fields(&data.fields, mutable),
        Data::Enum(data_enum) => handle_enum(data_enum, _struct_name, mutable),
        Data::Union(_) => unimplemented!("Unions are not yet supported"),
    };

    inner = if attr.collapsible {
        quote!(
            ui.collapsing(label, |ui| {
                #inner
            });
        )
    } else {
        quote!(
            if label!="" {
                ui.strong(label);
            }
            #inner
        )
    };

    if !attr.no_border {
        let style_path_str = attr
            .frame_style
            .clone()
            .unwrap_or("egui_inspect::DEFAULT_FRAME_STYLE".to_string());
        let style_path: TokenStream = style_path_str.parse().unwrap();
        inner = quote! {
            #style_path
             .to_frame()
             .show(ui, |ui| {
                #inner
            });
        }
    };

    if attr.horiz {
        inner = quote! {
            ui.horizontal(|ui|{
                #inner
            });
        }
    }

    // TODO: Avoid double frame? (with border)
    if let Some(on_hover_text) = attr.on_hover_text.clone() {
        inner = quote!(
            egui_inspect::egui::Frame::none()
                .show(ui, |ui| {
                   #inner
                }).response.on_hover_text_at_pointer(#on_hover_text);
        );
    }

    inner
}

fn handle_enum(data_enum: &DataEnum, struct_name: &Ident, mutable: bool) -> TokenStream {
    let variants: Vec<_> = data_enum.variants.iter().collect();
    let name_arms = variants.iter().map(|v| variant_name_arm(v, struct_name));

    let reflect_variant_name = quote!(
        let current_variant = match self {
            #(#name_arms,)*
        };
    );

    let combo_opts = variants.iter().map(|v| variant_combo(v, struct_name));
    let inspect_arms = variants
        .iter()
        .map(|v| variant_inspect_arm(v, struct_name, mutable));

    let variant_or_option = if mutable {
        quote! {
            ui.horizontal(|ui| {
                egui_inspect::egui::ComboBox::new(format!("{self:p}").as_str(), "")
                    .selected_text(current_variant)
                    .show_ui(ui, |ui| {
                        #(#combo_opts;)*
                    });
            });
        }
    } else {
        quote! {
            ui.label(current_variant);
        }
    };

    quote!(
        #reflect_variant_name

        #variant_or_option

        match self {
            #(#inspect_arms),*
        };
    )
}

fn variant_name_arm(variant: &Variant, struct_name: &Ident) -> TokenStream {
    let ident = &variant.ident;
    match &variant.fields {
        Fields::Named(_) => {
            quote!(#struct_name::#ident {..} => stringify!(#ident))
        }
        Fields::Unnamed(_) => {
            quote!(#struct_name::#ident (..) => stringify!(#ident))
        }
        Fields::Unit => {
            quote!(#struct_name::#ident => stringify!(#ident))
        }
    }
}

fn variant_combo(variant: &Variant, struct_name: &Ident) -> TokenStream {
    let ident = &variant.ident;
    // TODO: Replace with handle_fields,
    // which would need to take this ident as the base for fields instead of "self".
    match &variant.fields {
        Fields::Named(fields) => {
            let defaults = fields.named.iter().map(|f| {
                let ident = &f.ident;
                quote!( #ident: Default::default() )
            });
            quote!(ui.selectable_value(self, 
                                       #struct_name::#ident { #(#defaults),* }, 
                                       stringify!(#ident)))
        }
        Fields::Unnamed(fields) => {
            let defaults = fields.unnamed.iter().map(|_| quote!(Default::default()));
            quote!(ui.selectable_value(self, #struct_name::#ident ( #(#defaults),* ), stringify!(#ident)))
        }
        Fields::Unit => {
            quote!(ui.selectable_value(self, #struct_name::#ident, stringify!(#ident)))
        }
    }
}

fn variant_inspect_arm(variant: &Variant, struct_name: &Ident, mutable: bool) -> TokenStream {
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
                .map(|f| handle_named_field(f, mutable, true));
            quote!(#struct_name::#ident { #(#field_idents),* } => { #(#inspect_fields;)* })
        }
        Fields::Unnamed(fields) => {
            let field_idents: Vec<_> = (0..fields.unnamed.len()).map(|i| Ident::new(format!("unnamed_{i}").as_str(), Span::call_site())).collect();
            let inspect_fields = field_idents.iter().map(|id| {
                if mutable {
                    quote! {#id.inspect_mut("", ui)}
                } else {
                    quote! {#id.inspect("", ui)}
                }
            });
            quote!(#struct_name::#ident (#(#field_idents),*) => { #(#inspect_fields;)* })
        }
        Fields::Unit => {
            quote!(#struct_name::#ident => () )
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
    let field_inspects = fields
        .named
        .iter()
        .map(|f| handle_named_field(f, mutable, false));
    quote! {
        #(#field_inspects)*
    }
}

fn handle_unnamed_fields(fields: &FieldsUnnamed, mutable: bool) -> TokenStream {
    let mut field_inspects = Vec::new();
    for (i, _) in fields.unnamed.iter().enumerate() {
        let tuple_index = Index::from(i);
        let name = format!("Field {i}");
        let ref_str = if mutable { quote!(&mut) } else { quote!(&) };
        field_inspects.push(
            quote! { egui_inspect::EguiInspect::inspect(#ref_str self.#tuple_index, #name, ui);},
        );
    }

    let result = quote! {
        #(#field_inspects)*
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
