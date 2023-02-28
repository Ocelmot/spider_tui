use crate::renderer::Renderer;

use super::{ModelProcessor, ModelView};


impl<R: Renderer> ModelProcessor<R>{
	pub(crate) fn render(&mut self, renderer: &mut R){

		match self.view{
			ModelView::List => {
				renderer.render_page_list(&self.page_set.get_page_vec(), self.page_set.selected_index());
			},
			ModelView::Page => {
				match self.get_current_page(){
					Some(page) => renderer.render_page(page),
					None => {
						self.view = ModelView::List;
						renderer.render_page_list(&self.page_set.get_page_vec(), self.page_set.selected_index());
					},
				}
			},
		}
	}
} 