use spider_client::message::UiPage;


pub trait Renderer: Sync + Send + 'static{
    fn startup(&mut self);
    fn render_menu(&mut self);
    fn render_page(&mut self, page: &UiPage);
    fn render_page_list(&mut self, list: &Vec<&UiPage>, highlight_index: usize);
    fn shutdown(self);
}

pub mod tui;
