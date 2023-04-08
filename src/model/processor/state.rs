

use spider_client::message::{UiPage, UiPageManager};

use crate::renderer::Renderer;

use super::{ModelProcessor, PageState};





impl<R: Renderer> ModelProcessor<R>{

	pub(crate) fn set_pages(&mut self, pages: Vec<UiPage>){
		self.page_set.clear();
		self.page_set.add_pages(pages)
	}

	pub(crate) fn upsert_page(&mut self, page: UiPage){
		self.page_set.upsert_page(page);
	}

	pub(crate) fn get_current_page(&self) -> Option<&UiPage>{
		self.page_set.selected_page().map(|mgr| mgr.get_page())
	}

	pub(crate) fn get_current_mgr(&self) -> Option<&UiPageManager>{
		self.page_set.selected_page()
	}

	pub(crate) fn get_current_mgr_state(&mut self) -> Option<(&UiPageManager, &PageState)>{
		match self.page_set.selected_page() {
			Some(mgr) => {
				let id = mgr.get_page().id();
				if !self.page_states.contains_key(id) {
					self.page_states.insert(id.clone(), PageState::default());
				}
				let state = self.page_states.get(id).expect("any missing state should have been inserted");
				Some((mgr, state))
			},
			None => None,
		}
	}

	pub(crate) fn get_current_mgr_state_mut(&mut self) -> Option<(&mut UiPageManager, &mut PageState)>{
		match self.page_set.selected_page_mut() {
			Some(mgr) => {
				let id = mgr.get_page().id();
				if !self.page_states.contains_key(id) {
					self.page_states.insert(id.clone(), PageState::default());
				}
				let state = self.page_states.get_mut(id).expect("any missing state should have been inserted");
				Some((mgr, state))
			},
			None => None,
		}
	}

	pub(crate) fn select_prev_page(&mut self){
		self.page_set.select_prev_page()
	}

	pub(crate) fn select_next_page(&mut self){
		self.page_set.select_next_page()
	}
}