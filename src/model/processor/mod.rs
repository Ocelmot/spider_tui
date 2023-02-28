use spider_client::{
    message::{Message, UiElement, UiElementKind, UiMessage, UiPath, UiPageList, UiPageManager},
    Relation,
};

use crate::renderer::Renderer;

use super::update::ModelUpdate;

use std::thread::{spawn, JoinHandle};
use tokio::sync::mpsc::{Receiver, Sender};

mod render;
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
            self.sender
                .blocking_send(Message::Ui(UiMessage::GetPages))
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
}
