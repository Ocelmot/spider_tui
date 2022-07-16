use std::{io, thread::{self, JoinHandle, spawn}, time::{self, Duration}, vec};
use async_trait::async_trait;
use crossterm::{event::{Event, self, EnableMouseCapture, DisableMouseCapture}, terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}, execute};
use tokio::sync::mpsc::{Sender, Receiver, channel, error::TryRecvError};
use tui::{
    Terminal,
    widgets::{Block, Borders, Paragraph},
    layout::{Layout, Direction, Constraint},
    text::{Span, Spans}
};
use tui::backend::CrosstermBackend;

use super::{UI, UIControl, UIInput};

pub struct TUI{
    thread: Option<JoinHandle<Result<(), std::io::Error>>>,
    to_ui: Sender<UIControl>,
    from_ui: Receiver<UIInput>,


}

impl TUI{
    pub fn new() -> TUI{
        let (to_ui, mut ui_recv) = channel(10);
        let (ui_tx, from_ui) = channel(10);

        let thread = spawn(move || -> Result<(), std::io::Error> {

            
            let mut stdout = io::stdout();
            execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
            enable_raw_mode()?;
            let backend = CrosstermBackend::new(stdout);
            let mut terminal = Terminal::new(backend)?;

            let title = String::from("Spider Associate Client");
            let mut msgs = Vec::new();
            let mut input_buffer = String::new();
            let result = 'main_loop: loop {
                match ui_recv.try_recv() {
                    Ok(ctrl) => {
                        match ctrl{
                            UIControl::Message(s) => {
                                msgs.push(Spans::from(vec!(Span::raw(s))));
                            }
                        }
                    },
                    Err(e) => {
                        match e {
                            TryRecvError::Empty => {},
                            TryRecvError::Disconnected => {
                                println!("========= disconnected UI! =============");
                                break Ok(()); // Can get no more commands, finish
                            },
                        }
                    },
                }

                terminal.draw(|f|{
                    let block = Block::default()
                        .title(title.as_str())
                        .borders(Borders::ALL);
                    f.render_widget(block, f.size());

                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(1)
                        .constraints([Constraint::Min(0), Constraint::Length(2)].as_ref())
                        .split(f.size());
                    let messages = Paragraph::new(msgs.clone());

                    f.render_widget(messages, chunks[0]);

                    let input_span = Span::raw(input_buffer.clone());
                    f.set_cursor(chunks[1].x + input_span.width() as u16, chunks[1].y + 1);
                    let input = Paragraph::new(vec![Spans::from(vec!(input_span))])
                    .block(Block::default().borders(Borders::TOP));
                    f.render_widget(input, chunks[1]);

                    

                })?;

                while let Ok(true) = event::poll(Duration::from_millis(1)){
                    if let Event::Key(key) = event::read()?{
                        match key.code{
                            crossterm::event::KeyCode::Backspace => {input_buffer.pop();},
                            crossterm::event::KeyCode::Enter => {
                                ui_tx.blocking_send(UIInput::Message(input_buffer));
                                input_buffer = String::new();
                            },
                            // crossterm::event::KeyCode::Left => todo!(),
                            // crossterm::event::KeyCode::Right => todo!(),
                            // crossterm::event::KeyCode::Up => todo!(),
                            // crossterm::event::KeyCode::Down => todo!(),
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
                                input_buffer.push(ch)
                            },
                            // crossterm::event::KeyCode::Null => todo!(),
                            crossterm::event::KeyCode::Esc => {
                                ui_tx.blocking_send(UIInput::Close);
                                break 'main_loop Ok(());
                            },
                            _ => {},
                        }
                    }
                }
                
                thread::sleep(time::Duration::from_millis(50));
            };

            // cleanup
            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                DisableMouseCapture,
                LeaveAlternateScreen,
            )?;
            // terminal.show_cursor()?;
            result
        });

        TUI {thread: Some(thread), to_ui, from_ui}
    }
}

#[async_trait]
impl UI for TUI{

    fn send(&self, msg: UIControl){
        match self.to_ui.blocking_send(msg) {
            Ok(_) => {},
            Err(_) => panic!("Failed to send to ui thread"),
        }
    }

    async fn send_async(&self, msg: UIControl) {
        match self.to_ui.send(msg).await {
            Ok(_) => {},
            Err(_) => panic!("Failed to send to ui thread"),
        }
    }

    fn recv(&mut self) -> Option<UIInput> {
        self.from_ui.blocking_recv()
    }

    async fn recv_async(&mut self) -> Option<UIInput> {
        self.from_ui.recv().await
    }

    fn shutdown(&mut self) {
        match self.thread.take() {
            Some(handle) => {
                handle.join();
            },
            None => {},
        }
    }

}