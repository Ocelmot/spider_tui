use spider_client::{
    message::{Message, UiElement, UiElementKind, UiMessage, UiPath, UiPageList, UiPageManager, AbsoluteDatasetPath, DatasetData},
    Relation, SpiderId2048,
};

use crate::renderer::Renderer;

mod page_state;
pub use self::page_state::PageState;

use super::update::ModelUpdate;

use std::{thread::{spawn, JoinHandle}, collections::HashMap};
use tokio::sync::mpsc::{Receiver, Sender};

mod update;

pub(crate) mod state;

enum ModelView {
    List,
    Page,
}

pub struct ModelProcessor<R: Renderer> {
    receiver: Receiver<ModelUpdate>,
    sender: Sender<Message>,
    renderer: Option<R>,

    // view
    view: ModelView,

    // page rendering
    page_set: UiPageList,
    page_states: HashMap<SpiderId2048, PageState>,

    // Datasets
    datasets: HashMap<AbsoluteDatasetPath, Vec<DatasetData>>,

    exit: bool,
}

impl<R: Renderer> ModelProcessor<R> {
    pub async fn new(
        receiver: Receiver<ModelUpdate>,
        sender: Sender<Message>,
        renderer: R,
        relation: Relation,
    ) -> Self {


        Self {
            receiver,
            sender,
            renderer: Some(renderer),

            view: ModelView::List,

            page_set: UiPageList::new(),
            page_states: HashMap::new(),

            datasets: HashMap::new(),

            exit: false,
        }
    }

    pub fn start(mut self) -> JoinHandle<Result<(), std::io::Error>> {
        spawn(move || -> Result<(), std::io::Error> {
            let mut renderer = match self.renderer.take() {
                Some(renderer) => renderer,
                None => return Ok(()),
            };

            self.sender
                .blocking_send(Message::Ui(UiMessage::Subscribe))
                .unwrap();

            renderer.startup();

            loop {
                let update = match self.receiver.blocking_recv() {
                    Some(update) => update,
                    None => break, // No more messages, exit the model/renderer
                };
                self.update(update);

                self.render(&mut renderer);

                if self.exit {
                    break;
                }
            }

            renderer.shutdown();

            Ok(())
        })
    }

    pub(crate) fn render(&mut self, renderer: &mut R){

		match self.view{
			ModelView::List => {
				renderer.render_page_list(&self.page_set.get_page_vec(), self.page_set.selected_index());
			},
			ModelView::Page => {
				match self.get_context(){
					Some((mgr, state, data_map)) => renderer.render_page(mgr.get_page(), state, data_map),
					None => {
						self.view = ModelView::List;
						renderer.render_page_list(&self.page_set.get_page_vec(), self.page_set.selected_index());
					},
				}
			},
		}
	}

}
