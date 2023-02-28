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

    //test page
    test_page: UiPageManager,
    page_num: usize,


    exit: bool,
}

impl<R: Renderer> ModelProcessor<R> {
    pub async fn new(
        receiver: Receiver<ModelUpdate>,
        sender: Sender<Message>,
        renderer: R,
        relation: Relation,
    ) -> Self {

        // Testing ui
        let id = relation.id;

        let c1 = UiElement::from_string("Value is: ");

        let mut c2 = UiElement::from_string("0");
        c2.set_id("data");

        let mut page_mgr = UiPageManager::new(id.clone(), "Remote Page...");
        let mut root = page_mgr.get_element_mut(&UiPath::root()).expect("all pages have a root");
        root.set_kind(UiElementKind::Rows);
        root.append_child(c1);
        root.append_child(c2);
        drop(root);

        page_mgr.get_changes(); //  get changes to synch, but we are going to send the whole page at first. This
                                // Could instead set the initial elements with raw and then recalculate ids

        let msg = Message::Ui(UiMessage::SetPage(page_mgr.get_page().clone()));
        sender.send(msg).await.unwrap();

        Self {
            receiver,
            sender,
            renderer: Some(renderer),

            view: ModelView::List,

            page_set: UiPageList::new(),

            test_page: page_mgr,
            page_num: 0,

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
