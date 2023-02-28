
use spider_client::message::{UiElement, Message, UiMessage};

use crate::{model::{update::ModelUpdate}, renderer::Renderer};

use super::{ModelProcessor, ModelView};



impl<R: Renderer> ModelProcessor<R>{
	pub(crate) fn update(&mut self, update: ModelUpdate){
		match update{
			ModelUpdate::Event(event) => {
				match event{
					crossterm::event::Event::Key(key) => {
						match key.code{
							crossterm::event::KeyCode::Backspace => {},
							crossterm::event::KeyCode::Enter => {
								match self.view{
									ModelView::List => self.view = ModelView::Page,
									ModelView::Page => todo!(),
								}
							},
							// crossterm::event::KeyCode::Left => todo!(),
							// crossterm::event::KeyCode::Right => todo!(),
							crossterm::event::KeyCode::Up => {
								match self.view {
									ModelView::List => self.select_prev_page(),
									ModelView::Page => {},
								}
							},
							crossterm::event::KeyCode::Down => {
								match self.view {
									ModelView::List => self.select_next_page(),
									ModelView::Page => {},
								}
							},
							// crossterm::event::KeyCode::Home => todo!(),
							// crossterm::event::KeyCode::End => todo!(),
							// crossterm::event::KeyCode::PageUp => todo!(),
							// crossterm::event::KeyCode::PageDown => todo!(),
							// crossterm::event::KeyCode::Tab => todo!(),
							// crossterm::event::KeyCode::BackTab => todo!(),
							// crossterm::event::KeyCode::Delete => todo!(),
							// crossterm::event::KeyCode::Insert => todo!(),
							// crossterm::event::KeyCode::F(_) => todo!(),
							crossterm::event::KeyCode::Char(ch) => {
								if ch == 'q' {
									match self.view{
										ModelView::List => self.exit = true,
										ModelView::Page => {},
									}
								}
								if ch == ' ' {
									// self.test_page.update_element(element);
									self.page_num += 1;
									match self.test_page.get_by_id_mut("data"){
										Some(mut element) => {
											element.set_text(format!("{}", self.page_num));
										},
										None => {
											panic!("NO SUCH ID!");
										}
									}
									let changes = self.test_page.get_changes();
									let msg = Message::Ui(UiMessage::UpdateElements(changes));
									self.sender.blocking_send(msg);
								}
								
							},
							// crossterm::event::KeyCode::Null => todo!(),
							crossterm::event::KeyCode::Esc => {
								match self.view{
									ModelView::List => {},
									ModelView::Page => self.view = ModelView::List,
								}
							},
							_ => {},
						}
					},
					crossterm::event::Event::Mouse(_) => {},
					crossterm::event::Event::Resize(_, _) => {},
				}
			},
			ModelUpdate::SetPages(pages) => {
				self.set_pages(pages);
			},
			ModelUpdate::SetPage(page) => {
				self.upsert_page(page);
			},
			ModelUpdate::UpdateElementsFor(id, elements) => {
				match self.page_set.get_page_mut(&id){
					Some(page) => {
						page.apply_changes(elements);
					},
					None => {}, // No page, skip update
				}
			},
		}
	}

}