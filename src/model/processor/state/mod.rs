

use spider_client::message::UiPage;

use crate::renderer::Renderer;

use super::ModelProcessor;



impl<R: Renderer> ModelProcessor<R>{

	pub(crate) fn set_pages(&mut self, pages: Vec<UiPage>){
		self.page_set.clear();
		self.page_set.add_pages(pages)
		// let mut new_pages = Vec::new();
		// for page in pages{
		// 	new_pages.push(page.into());
		// }
		// self.pages = new_pages;
		// self.selected_page = 0;
	}

	pub(crate) fn upsert_page(&mut self, page: UiPage){
		self.page_set.upsert_page(page);
		// let page: ModelPage = page.into();

		// match self.pages.iter().position(|x| x.id == page.id) {
		// 	Some(index) => {
		// 		self.pages[index] = page;
		// 	},
		// 	None => {
		// 		self.pages.push(page);
		// 	},
		// }
	}

	pub(crate) fn get_current_page(&self) -> Option<&UiPage>{
		self.page_set.selected_page().map(|mgr| mgr.get_page())
		// self.pages.get(self.selected_page)
	}

	pub(crate) fn select_prev_page(&mut self){
		self.page_set.select_prev_page()
		// if self.pages.len() <= 1{
		// 	self.selected_page = 0;
		// }else{
		// 	if self.selected_page >= self.pages.len(){
		// 		self.selected_page = self.pages.len()-1
		// 	}
		// 	if self.selected_page != 0{
		// 		self.selected_page -= 1;
		// 	}
		// }
	}

	pub(crate) fn select_next_page(&mut self){
		self.page_set.select_next_page()
		// if self.pages.len() <= 1{
		// 	self.selected_page = 0;
		// }else{
		// 	self.selected_page += 1;
		// 	if self.selected_page >= self.pages.len(){
		// 		self.selected_page = self.pages.len()-1
		// 	}
		// }
	}
}