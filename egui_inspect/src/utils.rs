use std::any::type_name;

use egui::{text::LayoutJob, Align, FontSelection, RichText, Style};

pub fn concat_rich_text(rtv: Vec<RichText>) -> LayoutJob {
    let style = Style::default();
    let mut layout_job = LayoutJob::default();
    for rt in rtv {
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
