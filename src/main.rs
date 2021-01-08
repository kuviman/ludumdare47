use geng::prelude::*;

use ::noise::{NoiseFn as _, Seedable as _};

mod app;
mod model;
#[cfg(not(target_arch = "wasm32"))]
mod server;
pub mod util;

use app::App;
use model::{Id, Model};
#[cfg(not(target_arch = "wasm32"))]
use server::Server;

pub type ClientMessage = model::Message;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ServerMessage {
    PlayerId(Id),
    PackList(Vec<String>),
    UpdateClientView(model::ClientView),
    UpdateTiles(HashMap<Vec2<i64>, model::Tile>),
    UnloadArea(AABB<i64>),
}

type Connection = geng::net::client::Connection<ServerMessage, ClientMessage>;

#[derive(StructOpt)]
struct Opt {
    #[structopt(long)]
    no_server: bool,
    #[structopt(long)]
    no_client: bool,
    #[structopt(long)]
    addr: Option<String>,
    #[structopt(long)]
    log_level: Option<log::LevelFilter>,
}

fn main() {
    if let Some(dir) = std::env::var_os("CARGO_MANIFEST_DIR") {
        std::env::set_current_dir(std::path::Path::new(&dir).join("static")).unwrap();
    }

    geng::setup_panic_handler();
    let opt: Opt = StructOpt::from_args();
    let addr = opt
        .addr
        .as_ref()
        .map(|s| s.as_str())
        .or(option_env!("DEFAULT_ADDR"))
        .unwrap_or("127.0.0.1:7878");

    logger::init_with_level(opt.log_level.unwrap_or(log::LevelFilter::Info)).unwrap();

    #[cfg(not(target_arch = "wasm32"))]
    let (server, server_handle) = if !opt.no_server {
        let server = Server::new(
            addr,
            Model::create("new_world").unwrap_or(Model::load("new_world").unwrap()),
        );
        let server_handle = server.handle();
        if std::env::var_os("CARGO_MANIFEST_DIR").is_none() {
            ctrlc::set_handler({
                let server_handle = server_handle.clone();
                move || {
                    server_handle.shutdown();
                }
            })
            .unwrap();
        }
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
                        let assets: app::Assets = geng::LoadAsset::load(&geng, ".")
                            .await
                            .expect("Failed to load assets");
                        let connection = geng::net::client::connect(&addr).await;
                        let (message, connection) = connection.into_future().await;
                        let player_id = match message {
                            Some(ServerMessage::PlayerId(id)) => id,
                            _ => unreachable!(),
                        };
                        let (message, connection) = connection.into_future().await;
                        let pack_list = match message {
                            Some(ServerMessage::PackList(pack_list)) => pack_list,
                            _ => unreachable!(),
                        };
                        let resource_pack = app::ResourcePack::load_all(geng.clone(), pack_list)
                            .await
                            .expect("Failed to load resource packs");
                        let (message, connection) = connection.into_future().await;
                        let view = match message {
                            Some(ServerMessage::UpdateClientView(view)) => view,
                            _ => unreachable!(),
                        };
                        App::new(
                            &geng,
                            assets,
                            &Rc::new(resource_pack),
                            player_id,
                            view,
                            connection,
                        )
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
