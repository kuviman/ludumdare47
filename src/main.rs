use geng::prelude::*;

mod app;
mod model;
#[cfg(not(target_arch = "wasm32"))]
mod server;

use app::App;
use model::{Id, Model};
#[cfg(not(target_arch = "wasm32"))]
use server::Server;

pub type ClientMessage = model::Message;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ServerMessage {
    PlayerId(Id),
    Model(Model),
}

type Connection = geng::net::client::Connection<ServerMessage, ClientMessage>;

#[derive(StructOpt)]
struct Opt {
    #[structopt(long)]
    config: Option<std::path::PathBuf>,
    #[structopt(long)]
    no_server: bool,
    #[structopt(long)]
    no_client: bool,
    #[structopt(long)]
    addr: Option<String>,
}

fn main() {
    logger::init().unwrap();
    let opt: Opt = StructOpt::from_args();
    let addr = opt
        .addr
        .as_ref()
        .map(|s| s.as_str())
        .or(option_env!("DEFAULT_ADDR"))
        .unwrap_or("127.0.0.1:7878");

    #[cfg(not(target_arch = "wasm32"))]
    let (server, server_handle) = if !opt.no_server {
        let config = opt
            .config
            .as_ref()
            .map(|path| -> anyhow::Result<model::Config> {
                Ok(serde_json::from_reader(std::io::BufReader::new(
                    std::fs::File::open(path)?,
                ))?)
            })
            .map(|result| result.expect("Failed to load config"))
            .unwrap_or_default();

        let server = Server::new(addr, Model::new(config));
        let server_handle = server.handle();
        ctrlc::set_handler({
            let server_handle = server_handle.clone();
            move || {
                server_handle.shutdown();
            }
        })
        .unwrap();
        (Some(server), Some(server_handle))
    } else {
        (None, None)
    };

    #[cfg(not(target_arch = "wasm32"))]
    let server_thread = if let Some(server) = server {
        if opt.no_client {
            server.run();
            None
        } else {
            let thread = std::thread::spawn(move || server.run());
            // std::thread::sleep(std::time::Duration::from_millis(500));
            Some(thread)
        }
    } else {
        None
    };

    if !opt.no_client {
        let geng = Rc::new(Geng::new(geng::ContextOptions {
            title: "LudumDare 47".to_owned(),
            ..default()
        }));
        let geng = &geng;
        geng::run(
            geng.clone(),
            geng::LoadingScreen::new(
                geng,
                geng::EmptyLoadingScreen,
                {
                    let geng = geng.clone();
                    let addr = format!("{}://{}", option_env!("WSS").unwrap_or("ws"), addr);
                    async move {
                        let connection = geng::net::client::connect(&addr).await;
                        let (message, connection) = connection.into_future().await;
                        let player_id = match message {
                            Some(ServerMessage::PlayerId(id)) => id,
                            _ => unreachable!(),
                        };
                        let (message, connection) = connection.into_future().await;
                        let model = match message {
                            Some(ServerMessage::Model(model)) => model,
                            _ => unreachable!(),
                        };
                        App::new(&geng, player_id, model, connection)
                    }
                },
                |app| app,
            ),
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(server_thread) = server_thread {
            if !opt.no_client {
                server_handle.unwrap().shutdown();
            }
            info!("Waiting for server to shut down");
            server_thread.join().unwrap();
        }
    }
}