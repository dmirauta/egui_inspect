use std::ops::Range;

use crate as egui_inspect;
use crate::{utils::concat_rich_text, EguiInspect};
use derive_getters::Getters;
use egui::{self, Color32, RichText, ScrollArea};

fn substring_range(string: &str, substring: &str) -> Option<Range<usize>> {
    let start = string.find(substring);
    start.map(|si| si..si + substring.len())
}

/// Generic search ui that can be used in a wrapping free context.
/// Index passed through to on_select will only be usable for ordered data.
pub fn search_ui<'a>(
    ui: &mut egui::Ui,
    search_text: &mut String,
    entries: impl Iterator<Item = &'a String>,
    on_select: impl FnOnce(usize, &'a String) -> (),
) {
    search_text.inspect_mut("search for", ui);
    ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
        for (i, entry) in entries.enumerate() {
            if entry.contains(&*search_text) {
                let match_range =
                    substring_range(entry.as_str(), search_text.as_str()).unwrap_or(0..0);
                let prefix = &entry[..match_range.start];
                let suffix = &entry[match_range.end..];
                if ui
                    .button(concat_rich_text(vec![
                        RichText::new(prefix).color(Color32::WHITE),
                        RichText::new(search_text.clone()).color(Color32::GREEN),
                        RichText::new(suffix).color(Color32::WHITE),
                    ]))
                    .clicked()
                {
                    return on_select(i, entry);
                }
            }
        }
    });
}

/// Wrapper around Vec<String> which also holds persistent [search_ui] related data
#[derive(Getters)]
pub struct TextSearch {
    options: Vec<String>,
    input: String,
    selected: Option<usize>,
    just_clicked: bool,
    #[getter(skip)]
    pub display_selected: bool,
}

impl TextSearch {
    pub fn new(options: impl Into<Vec<String>>) -> Self {
        TextSearch {
            options: options.into(),
            input: String::new(),
            selected: None,
            just_clicked: false,
            display_selected: true,
        }
    }
}

impl EguiInspect for TextSearch {
    fn inspect(&self, _label: &str, ui: &mut egui::Ui) {
        match &self.selected {
            Some(i) => ui.label(format!("Selection: {}", self.options[*i])),
            None => ui.label(format!("Selection: <None>")),
        };
    }

    fn inspect_mut(&mut self, label: &str, ui: &mut egui::Ui) {
        self.just_clicked = false;
        if self.options.is_empty() {
            ui.label("<no available options>");
        } else {
            search_ui(ui, &mut self.input, self.options.iter(), |i, _| {
                self.selected = Some(i);
                self.just_clicked = true
            });
            if self.display_selected {
                self.inspect(label, ui)
            }
        }
    }
}

/// Wrapper around Vec<I> which also holds persistent [search_ui] related data
#[derive(EguiInspect)]
#[inspect(collapsible, no_trait_bound = "I")]
pub struct SearchSelection<I> {
    #[inspect(hide)]
    items: Vec<I>,
    #[inspect(hide)]
    str_from_i: Box<dyn Fn(&I) -> String>,
    #[inspect(name = "")]
    pub search: TextSearch,
}

impl<I> SearchSelection<I> {
    pub fn new(items: impl Into<Vec<I>>, str_from_i: impl 'static + Fn(&I) -> String) -> Self {
        let mut new = Self {
            items: items.into(),
            str_from_i: Box::new(str_from_i),
            search: TextSearch::new(vec![]),
        };
        new.reset_search_text();
        new
    }

    pub fn reset_search_text(&mut self) {
        let options = Vec::from_iter(self.items.iter().map(&self.str_from_i));
        self.search = TextSearch::new(options);
    }

    pub fn items(&self) -> &Vec<I> {
        &self.items
    }

    pub fn mut_items_with(&mut self, items_mut_fn: impl Fn(&mut Vec<I>) -> ()) {
        items_mut_fn(&mut self.items);
        let old_search = self.search.input.clone();
        self.reset_search_text();
        self.search.input = old_search;
    }

    pub fn get_selected_ref(&self) -> Option<&I> {
        self.search.selected.as_ref().map(|i| &self.items[*i])
    }

    /// may need to [Self::reset_search_text] on calling this, if mutation affects label
    pub fn get_selected_mut(&mut self) -> Option<&mut I> {
        self.search.selected.as_ref().map(|i| &mut self.items[*i])
    }
}

impl<I: Clone> SearchSelection<I> {
    #[allow(dead_code)]
    pub fn get_selected(&self) -> Option<I> {
        self.get_selected_ref().map(|i| i.clone())
    }
}
