use std::ops::Range;

use crate as egui_inspect;
use crate::{utils::concat_rich_text, EguiInspect};
use derive_getters::Getters;
use egui::text::LayoutJob;
use egui::{self, Color32, RichText, ScrollArea};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

fn substring_range(string: &str, substring: &str) -> Option<Range<usize>> {
    let start = string.find(substring);
    start.map(|si| si..si + substring.len())
}

pub fn non_contiguous_highlight(
    text: &str,
    higlight_idxs: &[usize],
    highlight_color: Color32,
    text_color: Color32,
) -> LayoutJob {
    let rti = text.chars().enumerate().map(|(i, c)| {
        let rt = RichText::new(c);
        match higlight_idxs.contains(&i) {
            true => rt.color(highlight_color),
            false => rt.color(text_color),
        }
    });
    concat_rich_text(rti)
}

pub trait SearchMethod {
    /// outputs score (how good is this match?) and matched character indices
    fn match_idxs(search_text: &str, queery_text: &str) -> Option<(i64, Vec<usize>)>;
}

pub struct BasicSearch;

impl SearchMethod for BasicSearch {
    fn match_idxs(search_text: &str, queery_text: &str) -> Option<(i64, Vec<usize>)> {
        substring_range(search_text, queery_text).map(|r| (-(r.start as i64), r.collect()))
    }
}

pub struct FuzzySearch;

impl SearchMethod for FuzzySearch {
    fn match_idxs(search_text: &str, queery_text: &str) -> Option<(i64, Vec<usize>)> {
        SkimMatcherV2::default().fuzzy_indices(search_text, queery_text)
    }
}

pub struct Match {
    idx: usize,
    char_idxs: Vec<usize>,
    score: i64,
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
    #[getter(skip)]
    pub fuzzy: bool,
    #[getter(skip)]
    cache: Vec<Match>,
}

impl TextSearch {
    pub fn new(options: impl Into<Vec<String>>, fuzzy: bool) -> Self {
        TextSearch {
            options: options.into(),
            input: String::new(),
            selected: None,
            just_clicked: false,
            display_selected: true,
            fuzzy,
            cache: vec![],
        }
    }
}

impl EguiInspect for TextSearch {
    fn inspect(&self, _label: &str, ui: &mut egui::Ui) {
        match &self.selected {
            Some(i) => ui.label(format!("Selection: {}", self.options[*i])),
            None => ui.label("Selection: <None>"),
        };
    }

    fn inspect_mut(&mut self, label: &str, ui: &mut egui::Ui) {
        self.just_clicked = false;
        if self.options.is_empty() {
            ui.label("<no available options>");
        } else {
            let match_idxs = match self.fuzzy {
                true => FuzzySearch::match_idxs,
                false => BasicSearch::match_idxs,
            };
            ui.horizontal(|ui| {
                ui.label("search for:");
                if ui.text_edit_singleline(&mut self.input).changed() {
                    self.cache = self
                        .options
                        .iter()
                        .enumerate()
                        .filter_map(|(idx, entry)| {
                            match_idxs(entry, &self.input).map(|(score, char_idxs)| Match {
                                idx,
                                char_idxs,
                                score,
                            })
                        })
                        .collect();
                    self.cache.sort_by_key(|t| t.score);
                }
            });
            if self.input.is_empty() && self.cache.len() != self.options.len() {
                self.cache = (0..self.options.len())
                    .map(|idx| Match {
                        idx,
                        char_idxs: vec![],
                        score: 0,
                    })
                    .collect();
            }
            ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
                for Match { idx, char_idxs, .. } in self.cache.iter().rev() {
                    let entry = &self.options[*idx];
                    if ui
                        .button(non_contiguous_highlight(
                            entry,
                            char_idxs,
                            Color32::GREEN,
                            Color32::WHITE,
                        ))
                        .clicked()
                    {
                        self.selected = Some(*idx);
                        self.just_clicked = true
                    }
                }
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
            search: TextSearch::new(vec![], false),
        };
        new.reset_search_text();
        new
    }

    pub fn reset_search_text(&mut self) {
        let options = Vec::from_iter(self.items.iter().map(&self.str_from_i));
        self.search = TextSearch::new(options, false);
    }

    pub fn items(&self) -> &Vec<I> {
        &self.items
    }

    pub fn mut_items_with(&mut self, items_mut_fn: impl Fn(&mut Vec<I>)) {
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
        self.get_selected_ref().cloned()
    }
}
