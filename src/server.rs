use super::*;

struct Client {
    player_id: Id,
    server_model: Arc<Mutex<Model>>,
    sender: Box<dyn geng::net::Sender<ServerMessage>>,
}

impl Drop for Client {
    fn drop(&mut self) {
        self.server_model
            .lock()
            .unwrap()
            .drop_player(self.player_id);
    }
}

impl geng::net::Receiver<ClientMessage> for Client {
    fn handle(&mut self, message: ClientMessage) {
        let mut server_model = self.server_model.lock().unwrap();
        if let ClientMessage::Ping = &message {
            self.sender.send(ServerMessage::Model(server_model.clone())); // TODO: Diff?
        } else {
            server_model.handle_message(self.player_id, message);
        }
    }
}
struct ServerApp {
    model: Arc<Mutex<Model>>,
}
impl geng::net::server::App for ServerApp {
    type Client = Client;
    type ServerMessage = ServerMessage;
    type ClientMessage = ClientMessage;
    fn connect(&mut self, mut sender: Box<dyn geng::net::Sender<ServerMessage>>) -> Client {
        let mut model = self.model.lock().unwrap();
        let player_id = model.new_player();
        sender.send(ServerMessage::PlayerId(player_id));
        sender.send(ServerMessage::Model(model.clone()));
        Client {
            server_model: self.model.clone(),
            player_id,
            sender,
        }
    }
}

pub struct Server {
    model: Arc<Mutex<Model>>,
    server: geng::net::Server<ServerApp>,
}

impl Server {
    pub fn new<T: std::net::ToSocketAddrs + Debug + Copy>(addr: T, model: Model) -> Self {
        let model = Arc::new(Mutex::new(model));
        Self {
            model: model.clone(),
            server: geng::net::Server::new(
                ServerApp {
                    model: model.clone(),
                },
                addr,
            ),
        }
    }
    pub fn handle(&self) -> geng::net::ServerHandle {
        self.server.handle()
    }
    pub fn run(self) {
        let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
        let server_thread = std::thread::spawn({
            let model = self.model;
            let running = running.clone();
            move || {
                while running.load(std::sync::atomic::Ordering::Relaxed) {
                    // TODO: smoother TPS
                    std::thread::sleep(std::time::Duration::from_millis(
                        (1000.0 / Model::TICKS_PER_SECOND) as u64,
                    ));
                    let mut model = model.lock().unwrap();
                    model.tick();
                }
            }
        });
        self.server.run();
        running.store(false, std::sync::atomic::Ordering::Relaxed);
        server_thread.join().expect("Failed to join server thread");
    }
}
