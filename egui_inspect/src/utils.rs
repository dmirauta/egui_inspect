use std::any::type_name;

use egui::{text::LayoutJob, Align, FontSelection, RichText, Style};

pub fn concat_rich_text(rtv: impl IntoIterator<Item = RichText>) -> LayoutJob {
    let style = Style::default();
    let mut layout_job = LayoutJob::default();
    for rt in rtv.into_iter() {
        rt.append_to(
            &mut layout_job,
            &style,
            FontSelection::Default,
            Align::Center,
        );
    }
    layout_job
}

pub fn type_name_base<T>() -> &'static str {
    let mut name: &str = type_name::<T>();
    if let Some(_name) = name.split("::").last() {
        name = _name;
    }
    name
}

#[test]
fn concat_rich_text_accepts_vec() {
    concat_rich_text(vec![
        RichText::new('a'),
        RichText::new('b'),
        RichText::new('c'),
    ]);
}

#[test]
fn concat_rich_text_accepts_itr() {
    concat_rich_text(['a', 'b', 'c'].map(RichText::new));
}
