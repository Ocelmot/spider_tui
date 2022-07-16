mod ui;

use ui::tui::TUI;
use crate::ui::{UIControl, UI};

use std::{io, env, path::Path};


use tokio::select;

use spider_client::{SpiderClient, Message, SpiderClientConfig};

use tracing::{info, debug};
use tracing_appender::rolling::{Rotation, RollingFileAppender};

#[tokio::main]
async fn main() -> Result<(), io::Error> {

    // command line arguments: <filename>
    // filename is name of config file, defaults to config.json
    let mut args = env::args().skip(1);
    let path_str = args.next().unwrap_or("spider_tui_config.json".to_string());
    let config_path = Path::new(&path_str);

    let config = SpiderClientConfig::from_file(config_path);
    let log_path = config.log_path.clone();
    let log_path = log_path.unwrap_or(format!("spider_tui.log"));
    
    // Setup tracing
    let file_appender = RollingFileAppender::new(Rotation::NEVER, "", log_path);
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .pretty().with_ansi(false)
        .with_writer(non_blocking)
        .init();
    info!("Starting!");

    let client = SpiderClient::from_config(config);
    let mut client_handle = client.start().await;

    let mut ui = TUI::new();


    loop {
        select! {
            msg_result = ui.recv_async() => {
                match msg_result{
                    Some(msg) => {
                        match msg{
                            ui::UIInput::Message(text) => {
                                if text == ""{
                                    continue;
                                }
                                let ch = text.chars().next().unwrap();
                                if ch == ':' {
                                    let args = &text[1..].split_whitespace().collect::<Vec<_>>();
                                    if args.len() == 0 {
                                        continue;
                                    }
                                    
                                    let response = match args[0] {
                                        "quit" => break,
                                        _ => format!("No such command: {}", &text[1..]),
                                    };
                                    ui.send_async(UIControl::Message(response)).await;
                                    
                                }else{
                                    let parts = &text.split(':').collect::<Vec<_>>();
                                    if parts.len() <= 1 {
                                        ui.send_async(UIControl::Message(format!("Invalid format, requires: <id>:<message>"))).await;
                                        continue;
                                    }
                                    if let Ok(to) = parts[0].parse::<u32>(){
                                        let body = parts[1..].join(":").as_bytes().to_vec();
                                        let msg = Message::Message{ msg_type: String::from("text"), routing: Some(to), body };
                                        ui.send_async(UIControl::Message(format!("Sending message: {:?}", msg))).await;
                                        client_handle.emit(msg).await;
                                    }else{
                                        ui.send_async(UIControl::Message(format!("Failed to parse id: {}", parts[0]))).await;
                                    }
                                }
                            },
                            ui::UIInput::Close => {
                                break;
                            }
                        }
                    },
                    None => {println!("No message"); break;},
                }
            },
            base_msg = client_handle.recv() => {
                if let Some(msg) = base_msg{
                    match msg {
                        Message::Introduction{..}=> {}, // discard verification for now
                        Message::Message { msg_type, routing, body } => {
                            let body = String::from_utf8_lossy(&body);
                            ui.send_async(UIControl::Message(format!(" -> {:?}", body))).await
                        },
                    }
                }else{
                    println!("Client connection to base broke");
                    break;
                }
            },
        };
    }

    // println!("Should start closing down here!");
    // chord.process_handle.await.expect("thread panicked");

    Ok(())
}
