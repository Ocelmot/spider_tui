
use spider_client::message::{UiMessage, UiInput, Message, UiElementKind};

use crate::{model::{update::ModelUpdate}, renderer::Renderer};

use super::{ModelProcessor, ModelView, page_state::SelectDirection};



impl<R: Renderer> ModelProcessor<R>{
	pub(crate) fn update(&mut self, update: ModelUpdate){
		match update{
			ModelUpdate::Event(event) => {
				match event{
					crossterm::event::Event::Key(key) => {
						if let crossterm::event::KeyEventKind::Release = key.kind {
							return; // Dont respond to key-up events
						}
						match key.code{
							crossterm::event::KeyCode::Backspace => {
								if let Some((mgr, state)) = self.get_current_mgr_state_mut(){
									let input = state.get_selected_uncommited_input_mut();
									match input{
										Some(input) => {
											input.pop();
										},
										None => {},
									}
								}
							},
							crossterm::event::KeyCode::Enter => {
								match self.view{
									ModelView::List => self.view = ModelView::Page,
									ModelView::Page => {
										// send input message!
										match self.get_current_mgr_state_mut(){
											Some((mgr, state)) => {
												match state.get_selected_id() {
													Some(id) => {
														match mgr.get_by_id(id){
															Some(elem) => {
																let selected_datum = state.get_selected_datum();
																let mut elem_kind = elem.kind().clone();
																elem_kind = elem_kind.resolve(&selected_datum.as_ref());

																match elem_kind{
																	UiElementKind::TextEntry => {
																		let page_id = mgr.get_page().id().clone();
																		let id = id.clone();
																		if let Some(text) = state.take_selected_uncommited_input_mut(){
																			let dataset_ids = state.get_selected_datasets().clone();
																			let msg = Message::Ui(UiMessage::InputFor(page_id, id, dataset_ids, UiInput::Text(text)));
																			self.sender.blocking_send(msg);
																		}
																	},
																	UiElementKind::Button => {
																		let page_id = mgr.get_page().id().clone();
																		let dataset_ids = state.get_selected_datasets().clone();
																		let msg = Message::Ui(UiMessage::InputFor(page_id, id.to_string(), dataset_ids, UiInput::Click));
																		self.sender.blocking_send(msg);
																	},
																	_ => {}
																}
															},
															None => {}, // non-existant element cant update
														}
													},
													None => {},
												}
											},
											None => {},
										}
									},
								}
							},
							crossterm::event::KeyCode::Left => {
								match self.view {
									ModelView::List => {},
									ModelView::Page => {
										if let Some((mgr, state, data_map)) = self.get_context(){
											state.select_next(mgr, data_map, SelectDirection::Left);
										}
									},
								}
							},
							crossterm::event::KeyCode::Right => {
								match self.view {
									ModelView::List => {},
									ModelView::Page => {
										if let Some((mgr, state, data_map)) = self.get_context(){
											state.select_next(mgr, data_map, SelectDirection::Right);
										}
									},
								}
							},
							crossterm::event::KeyCode::Up => {
								match self.view {
									ModelView::List => self.select_prev_page(),
									ModelView::Page => {
										if let Some((mgr, state, data_map)) = self.get_context(){
											state.select_next(mgr, data_map, SelectDirection::Up);
										}
									},
								}
							},
							crossterm::event::KeyCode::Down => {
								match self.view {
									ModelView::List => self.select_next_page(),
									ModelView::Page => {
										if let Some((mgr, state, data_map)) = self.get_context(){
											state.select_next(mgr, data_map, SelectDirection::Down);
										}
									},
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
								
								// append character to currently selected input
								if let ModelView::Page = self.view {
									if let Some((mgr, state)) = self.get_current_mgr_state_mut(){
										let input = state.get_selected_uncommited_input_mut();
										match input{
											Some(input) => {
												input.push(ch);
											},
											None => {
												state.set_selected_uncommited_input(String::from(ch));
											},
										}
									}
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
					
					crossterm::event::Event::Paste(str) => {
						// append paste to currently selected input
						if let ModelView::Page = self.view {
							if let Some((_, state)) = self.get_current_mgr_state_mut(){
								let input = state.get_selected_uncommited_input_mut();
								match input{
									Some(input) => {
										input.push_str(&str);
									},
									None => {
										state.set_selected_uncommited_input(str);
									},
								}
							}
						}
					},
					crossterm::event::Event::Mouse(_) => {},
					crossterm::event::Event::Resize(_, _) => {},
					crossterm::event::Event::FocusGained => {},
					crossterm::event::Event::FocusLost => {},
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
    		ModelUpdate::UpdateDataset(path, dataset) => {
				self.datasets.insert(path, dataset);
			},
		}
	}

}