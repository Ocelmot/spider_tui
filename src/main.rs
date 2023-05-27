mod renderer;
mod config;
mod model;
mod event_stream;

// use console_subscriber;

use model::{Model, update::ModelUpdate};
use crate::config::SpiderTuiConfig;

use std::{io, env, path::{Path, PathBuf}};

use tokio::select;

use spider_client::{
    SpiderClient,
    message::{
        Message,
        UiMessage,
        UiPage,
        UiElement, UiElementKind
    },
    SpiderId2048,
    SelfRelation,
    Role,
    Relation,
    AddressStrategy
};

use tracing::{info, debug};
use tracing_appender::rolling::{Rotation, RollingFileAppender};

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    console_subscriber::init();

    // command line arguments: <filename>
    // filename is name of config file, defaults to config.json
    let mut args = env::args().skip(1);
    let path_str = args.next().unwrap_or("spider_tui_config.json".to_string());
    let config_path = Path::new(&path_str);

    let config = SpiderTuiConfig::from_file(config_path);
    let log_path = config.log_path.clone();
    
    // Setup tracing
    // let file_appender = RollingFileAppender::new(Rotation::NEVER, "", log_path);
    // let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    // tracing_subscriber::fmt()
    //     .pretty().with_ansi(false)
    //     .with_writer(non_blocking)
    //     .init();
    info!("Starting!");

    let client_path = PathBuf::from(config.state_data_path);
    let mut client = if client_path.exists(){
        SpiderClient::from_file(&client_path)
    }else{
        let mut client = SpiderClient::new();
        client.set_state_path(&client_path);
        client.add_strat(AddressStrategy::Addr(String::from("localhost:1930")));
        client.save();
        client
    };
    
    if !client.has_host_relation(){
        let path = PathBuf::from(&config.keyfile_path);        

        let data = match std::fs::read_to_string(&path){
            Ok(str) => str,
            Err(_) => String::from("[]"),
        };
		let id:SpiderId2048 = serde_json::from_str(&data).expect("Failed to deserialize spiderid");
        let host = Relation { id, role: Role::Peer };
        client.set_host_relation(host);
        client.save();
    }

    client.connect().await;

    let renderer = renderer::tui::TUI::new();
    let model = Model::start(renderer, client.self_relation()).await;


    


    // connect client and keyboard inputs to model, connect model outputs to base
    splice_client_keyboard_model(client, model).await;


    Ok(())
}



async fn splice_client_keyboard_model(mut client: SpiderClient, mut model: Model){

    let mut events = event_stream::get_event_stream();
    loop {

        select! {
            // keypresses to model
            event = events.recv() =>{
                match event{
                    Some(event) => {
                        // println!("terminal event: {:?}", event);
                        let update = ModelUpdate::Event(event);
                        if let Err(_) = model.send(update).await{
                            debug!("Failed to send to model");
                            break;// encountered error
                        }                    
                    },
                    None => break, // inputs have failed, quit
                }

            }
            // client messages to model
            from_client = client.recv() => {
                match from_client{
                    Some(from_client) => {
                        // println!("from client: {:?}", from_client);
                        let update = message_to_update(from_client);
                        if let Some(update) = update {
                            model.send(update).await;
                        }
                    },
                    None => todo!("need to update the model about the client's disconnection and attempt a reconnection"),
                }
            }
            // model messages to client
            from_model = model.recv() => {
                match from_model{
                    Some(from_model) => {
                        // println!("Message from model: {:?}", from_model);
                        client.send(from_model).await
                    },
                    None => break, // model has quit
                }
            }
        }
    }

}



fn message_to_update(msg: Message) -> Option<ModelUpdate> {
    match msg {
        Message::Ui(ui) => {
            match ui {
                UiMessage::Subscribe => None,
                UiMessage::Pages(page_list) => Some(ModelUpdate::SetPages(page_list)),
                UiMessage::GetPage(_) => None,
                UiMessage::Page(page) => Some(ModelUpdate::SetPage(page)),
                UiMessage::UpdateElementsFor(id, updates) => Some(ModelUpdate::UpdateElementsFor(id, updates)),
                UiMessage::Dataset(path, dataset) => Some(ModelUpdate::UpdateDataset(path, dataset)),
                UiMessage::InputFor(_, _, _, _) => None,

                UiMessage::SetPage(_) => None,
                UiMessage::ClearPage => None,
                UiMessage::UpdateElements(_) => None,
                UiMessage::Input(_, _, _) => None,
            }
        },
        Message::Dataset(_) => None,
        Message::Event (_) => None,
    }
}

