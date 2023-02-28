
// converter from crossbeam events to async stream

use std::thread::spawn;

use crossterm::event::{self, Event};
use tokio::sync::mpsc::{Receiver, channel};




pub fn get_event_stream() -> Receiver<Event> {
	let (stream_tx, stream_rx) = channel(50);

	let _ = spawn(move || -> Result<(), std::io::Error> {
		loop{
			let e = event::read();
			match e {
				Ok(e) => {
					match stream_tx.blocking_send(e){
						Ok(_) => {},
						Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, e)),
					}
				},
				Err(e) => return Err(e),
			}
		}
		
	});

	stream_rx
}