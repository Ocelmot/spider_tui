use std::collections::HashMap;

use spider_client::message::{UiPage, DatasetData, AbsoluteDatasetPath};

use crate::model::processor::PageState;


pub trait Renderer: Sync + Send + 'static{
    fn startup(&mut self);
    fn render_menu(&mut self);
    fn render_page(&mut self, page: &UiPage, state: &PageState, data_map: &HashMap<AbsoluteDatasetPath, Vec<DatasetData>>);
    fn render_page_list(&mut self, list: &Vec<&UiPage>, highlight_index: usize);
    fn shutdown(self);
}

pub mod tui;
