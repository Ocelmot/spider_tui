
pub mod update;

use spider_client::{message::Message, SelfRelation, Relation};
use update::ModelUpdate;

use std::thread::JoinHandle;
use tokio::sync::mpsc::{channel, Receiver, Sender, error::SendError};

pub(crate) mod processor;
use processor::ModelProcessor;

use crate::renderer::Renderer;

pub struct Model{
	_handle: JoinHandle<Result<(), std::io::Error>>,
	
	//pipes in/out
	model_tx: Sender<ModelUpdate>,
	model_rx: Receiver<Message>,
	

}


impl Model{
	// take piped inputs to modify rendered model
	pub async fn start<R: Renderer>(renderer: R, relation: Relation) -> Self{

		let (model_tx, mod_rx) = channel(50);
        let (mod_tx, model_rx) = channel(50);

		let processor = ModelProcessor::new(mod_rx, mod_tx, renderer, relation).await;
		let thread = processor.start();

		Self{
			_handle: thread,
			model_tx,
			model_rx,
		}
	}

	pub async fn recv(&mut self) -> Option<Message>{
		self.model_rx.recv().await
	}

	pub async fn send(&mut self, message: ModelUpdate) -> Result<(), SendError<ModelUpdate>>{
		self.model_tx.send(message).await
	}

	
}
