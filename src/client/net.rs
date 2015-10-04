use std::thread;
use std::net::SocketAddr;
use std::sync::mpsc::{TryRecvError};
use cobalt::{
    Config, Client, Connection, ConnectionID, Handler, SyncToken, UdpSocket
};
use std::collections::VecDeque;

pub use cobalt::MessageKind as MessageKind;

pub struct Network {
    handler: EventHandler,
    client: Client,
    sync_token: SyncToken<UdpSocket>,
    connected: bool,
    connection_time: f64,
    connection_rtt: u32,
    connection_packet_loss: f32
}

impl Network {

    pub fn new(tick_rate: u32, addr: SocketAddr) -> Network {

        let mut handler = EventHandler::new();

        let mut client = Client::new(Config {
            send_rate: tick_rate,
            .. Config::default()
        });

        let sync_token = client.connect_sync(&mut handler, addr).unwrap();

        Network {
            handler: handler,
            client: client,
            sync_token: sync_token,
            connected: false,
            connection_time: 0.0,
            connection_rtt: 0,
            connection_packet_loss: 0.0
        }

    }

    // Getters ----------------------------------------------------------------
    pub fn connected(&mut self) -> bool {
        self.connected
    }

    pub fn server_addr(&mut self) -> Result<SocketAddr, ()> {
        self.client.peer_addr().or(Err(()))
    }

    pub fn rtt(&mut self) -> u32 {
        self.connection_rtt
    }

    pub fn packet_loss(&mut self) -> f32 {
        self.connection_packet_loss
    }


    // Methods ----------------------------------------------------------------
    pub fn receive(&mut self) {
        self.client.receive_sync(&mut self.handler, &mut self.sync_token);
        self.client.tick_sync(&mut self.handler, &mut self.sync_token);
    }

    pub fn send(&mut self) {
        self.client.send_sync(&mut self.handler, &mut self.sync_token);
    }

    pub fn message(&mut self, time: f64) -> Result<EventType, TryRecvError> {

        // Try to reconnect after 3 seconds
        if self.connection_time != 0.0 && time - self.connection_time > 3.0 {
            self.connection_time = 0.0;
            self.handler.reset();
        }

        // Internal event handling
        match self.handler.try_recv() {

            Some(event) => {

                match event {
                    EventType::ConnectionFailed(_) => {
                        println!("Connection failed, retrying in 3 seconds...");
                        self.connection_time = time;
                    },
                    EventType::Connection(_) => {
                        println!("Connection established");
                        self.connected = true;
                    },
                    EventType::ConnectionLost(_) => {
                        println!("Connection lost, reconnecting in 3 seconds...");
                        self.connection_time = time;
                        self.connected = false;
                    },
                    EventType::Tick(rtt, packet_loss) => {
                        self.connection_rtt = rtt;
                        self.connection_packet_loss = packet_loss;
                    },
                    _ => ()
                }

                Ok(event)

            },

            None => Err(TryRecvError::Empty)

        }

    }

    pub fn send_message(&mut self, kind: MessageKind, data: Vec<u8>) {
        self.handler.send(kind, data);
    }

    pub fn close(&mut self) {
        self.client.close_sync(&mut self.handler, &mut self.sync_token).unwrap();
    }

}


#[derive(Debug)]
pub enum EventType {
    Bind,
    Shutdown,
    Connect,
    Close,
    Tick(u32, f32),
    Message(ConnectionID, Vec<u8>),
    Connection(ConnectionID),
    ConnectionFailed(ConnectionID),
    ConnectionCongestionState(ConnectionID, bool),
    ConnectionLost(ConnectionID),
    PacketLost(ConnectionID, Vec<u8>)
}

pub enum Command {
    Close,
    Reset,
    Shutdown,
    Send(MessageKind, Vec<u8>),
    SendTo(ConnectionID, MessageKind, Vec<u8>)
}

pub struct EventHandler {
    events: VecDeque<EventType>,
    commands: VecDeque<Command>
}

impl EventHandler {

    pub fn new() -> EventHandler {
        EventHandler {
            events: VecDeque::new(),
            commands: VecDeque::new()
        }
    }

    pub fn try_recv(&mut self) -> Option<EventType> {
        self.events.pop_front()
    }

    pub fn send(&mut self, kind: MessageKind, data: Vec<u8>) {
        self.commands.push_back(Command::Send(kind, data));
    }

    pub fn reset(&mut self) {
        self.commands.push_back(Command::Reset);
    }

    pub fn close(&mut self) {
        self.commands.push_back(Command::Close);
    }

    pub fn send_to(&mut self, id: ConnectionID, kind: MessageKind, data: Vec<u8>) {
        self.commands.push_back(Command::SendTo(id, kind, data));
    }

    pub fn shutdown(&mut self) {
        self.commands.push_back(Command::Shutdown);
    }

}

impl Handler<Client> for EventHandler {

    fn connect(&mut self, _: &mut Client) {
        self.events.push_back(EventType::Connect);
    }

    fn tick_connection(
        &mut self,
        client: &mut Client,
        conn: &mut Connection
    ) {

        let id = conn.id();

        // Create events from received connection messages
        for msg in conn.received() {
            self.events.push_back(EventType::Message(id, msg));
        }

        // Create a tick event
        self.events.push_back(EventType::Tick(conn.rtt(), conn.packet_loss()));

        // TODO we somehow need to be able to send a outgoing packet without delay

        // Handle commands
        while let Some(cmd) = self.commands.pop_front() {
            match cmd {
                Command::Send(kind, data) => {
                    conn.send(kind, data);
                },
                Command::Reset => {
                    conn.reset();
                },
                Command::Close => {
                    client.close().unwrap();
                },
                _ => ()
            }
        }

    }

    fn close(&mut self, _: &mut Client) {
        self.events.push_back(EventType::Close);
    }

    fn connection(&mut self, _: &mut Client, conn: &mut Connection) {
        self.events.push_back(EventType::Connection(conn.id()));
    }

    fn connection_failed(&mut self, client: &mut Client, conn: &mut Connection) {
        self.events.push_back(EventType::ConnectionFailed(conn.id()));
    }

    fn connection_packet_lost(
        &mut self, _: &mut Client, conn: &mut Connection, data: &[u8]
    ) {
        self.events.push_back(EventType::PacketLost(conn.id(), data.to_vec()));
    }

    fn connection_congestion_state(&mut self, _: &mut Client, conn: &mut Connection, state: bool) {
        self.events.push_back(EventType::ConnectionCongestionState(conn.id(), state));
    }

    fn connection_lost(&mut self, _: &mut Client, conn: &mut Connection) {
        self.events.push_back(EventType::ConnectionLost(conn.id()));
    }

}

