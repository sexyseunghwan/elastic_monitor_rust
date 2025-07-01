use crate::common::*;

pub trait MessageFormatter {
    fn get_telegram_format(&self) -> String;
    fn get_email_format(&self) -> HtmlContents;
}

#[derive(Debug, new)]
pub struct HtmlContents {
    pub html_form_map: HashMap<String, String>,
    pub view_page_dir: String,
}
