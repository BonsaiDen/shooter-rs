// External Dependencies ------------------------------------------------------
use std::net::SocketAddr;
use std::collections::VecDeque;
use std::sync::mpsc::TryRecvError;
use cobalt::{
    Config, Client, Connection, ConnectionID, Handler, ClientState, UdpSocket,
    Stats, MessageKind
};


// Network Message Types ------------------------------------------------------
pub enum Message {
    ServerConfig = 0,
    ServerState = 1,
    ServerEvents = 2,
    ClientInput = 3,
    ClientEvents = 4,
    Unknown = 255
}

impl Message {
    pub fn from_u8(id: u8) -> Message {
        match id {
            0 => Message::ServerConfig,
            1 => Message::ServerState,
            2 => Message::ServerEvents,
            3 => Message::ClientInput,
            4 => Message::ClientEvents,
            _ => Message::Unknown
        }
    }
}


// Network Stream Abstraction -------------------------------------------------
pub struct Stream {
    handler: StreamHandler,
    client: Client,
    sync_token: ClientState<UdpSocket>,
    tick_rate: u32,
    connected: bool,
    connection_time: f64,

    // TODO move stats out here?
    connection_rtt: u32,
    connection_packet_loss: f32,
    bytes_sent: u32,
    bytes_received: u32,
}

impl Stream {

    pub fn new(addr: SocketAddr) -> Stream {

        let tick_rate = 30;
        let mut handler = StreamHandler::new();
        let mut client = Client::new(Config {
            send_rate: tick_rate,
            connection_init_threshold: 250,
            .. Config::default()
        });

        let sync_token = client.connect_sync(&mut handler, addr).unwrap();

        Stream {
            handler: handler,
            client: client,
            sync_token: sync_token,
            connected: false,
            connection_time: 0.0,
            connection_rtt: 0,
            connection_packet_loss: 0.0,
            bytes_sent: 0,
            bytes_received: 0,
            tick_rate: tick_rate
        }

    }

    // Getters ----------------------------------------------------------------
    pub fn id(&self) -> ConnectionID {
        ConnectionID(0)
    }

    pub fn connected(&self) -> bool {
        self.connected
    }

    pub fn server_addr(&self) -> Result<SocketAddr, ()> {
        self.client.peer_addr().or(Err(()))
    }

    pub fn rtt(&self) -> u32 {
        self.connection_rtt
    }

    pub fn packet_loss(&self) -> f32 {
        self.connection_packet_loss
    }

    pub fn bytes_sent(&self) -> u32 {
        self.bytes_sent
    }

    pub fn bytes_received(&self) -> u32 {
        self.bytes_received
    }

    pub fn get_tick_rate(&self) -> u32 {
        self.tick_rate
    }

    pub fn set_tick_rate(&mut self, tick_rate: u32) {
        self.tick_rate = tick_rate;
        self.client.set_config(Config {
            send_rate: tick_rate,
            connection_init_threshold: 250,
            .. Config::default()

        }, &mut self.sync_token)
    }


    // Methods ----------------------------------------------------------------
    pub fn receive(&mut self) {
        self.client.receive_sync(
            &mut self.handler, &mut self.sync_token, 1000 / self.tick_rate
        );
        self.client.tick_sync(&mut self.handler, &mut self.sync_token);
    }

    pub fn send(&mut self) {
        self.client.send_sync(&mut self.handler, &mut self.sync_token);
    }

    pub fn message(&mut self, time: f64) -> Result<StreamEvent, TryRecvError> {

        // Try to reconnect after 3 seconds
        if self.connection_time != 0.0 && time - self.connection_time > 3.0 {
            self.connection_time = 0.0;
            self.handler.reset();
        }

        // Internal event handling
        match self.handler.try_recv() {

            Some(event) => {

                match event {
                    StreamEvent::ConnectionFailed(_) => {
                        // TODO forward and handle in game code
                        println!("Connection failed, retrying in 3 seconds...");
                        self.connection_time = time;
                    },
                    StreamEvent::Connection(_) => {
                        // TODO forward and handle in game code
                        println!("Connection established");
                        self.connected = true;
                    },
                    StreamEvent::ConnectionLost(_) => {
                        // TODO forward and handle in game code
                        println!("Connection lost, reconnecting in 3 seconds...");
                        self.connection_time = time;
                        self.connected = false;
                    },
                    StreamEvent::Tick(rtt, packet_loss, stats) => {
                        self.connection_rtt = rtt;
                        self.connection_packet_loss = packet_loss;
                        self.bytes_sent = stats.bytes_sent;
                        self.bytes_received = stats.bytes_received;
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

    pub fn destroy(&mut self) {
        self.client.close_sync(&mut self.handler, &mut self.sync_token).unwrap();
    }

}




#[derive(Debug)]
pub enum StreamEvent {
    Connect,
    Close,
    Tick(u32, f32, Stats),
    Message(ConnectionID, Vec<u8>),
    Connection(ConnectionID),
    ConnectionFailed(ConnectionID),
    ConnectionCongestionState(ConnectionID, bool),
    ConnectionLost(ConnectionID),
    PacketLost(ConnectionID, Vec<u8>)
}

pub enum StreamCommand {
    Reset,
    Send(MessageKind, Vec<u8>)
}


// Network Stream Handler Implementation --------------------------------------
pub struct StreamHandler {
    events: VecDeque<StreamEvent>,
    commands: VecDeque<StreamCommand>
}

impl StreamHandler {

    pub fn new() -> StreamHandler {
        StreamHandler {
            events: VecDeque::new(),
            commands: VecDeque::new()
        }
    }

    pub fn try_recv(&mut self) -> Option<StreamEvent> {
        self.events.pop_front()
    }

    pub fn send(&mut self, kind: MessageKind, data: Vec<u8>) {
        self.commands.push_back(StreamCommand::Send(kind, data));
    }

    pub fn reset(&mut self) {
        self.commands.push_back(StreamCommand::Reset);
    }

}

impl Handler<Client> for StreamHandler {

    fn connect(&mut self, _: &mut Client) {
        self.events.push_back(StreamEvent::Connect);
    }

    fn tick_connection(
        &mut self,
        client: &mut Client,
        conn: &mut Connection
    ) {

        let id = conn.id();

        // Create events from received connection messages
        for msg in conn.received() {
            self.events.push_back(StreamEvent::Message(id, msg));
        }

        // Create a tick event
        self.events.push_back(
            StreamEvent::Tick(conn.rtt(), conn.packet_loss(), client.stats())
        );

        // TODO we somehow need to be able to send a outgoing packet without delay
        // we currently have a one tick delay (?) is this still correct?

        // Handle commands
        while let Some(cmd) = self.commands.pop_front() {
            match cmd {
                StreamCommand::Send(kind, data) => {
                    conn.send(kind, data);
                },
                StreamCommand::Reset => {
                    conn.reset();
                }
            }
        }

    }

    fn close(&mut self, _: &mut Client) {
        self.events.push_back(StreamEvent::Close);
    }

    fn connection(&mut self, _: &mut Client, conn: &mut Connection) {
        self.events.push_back(StreamEvent::Connection(conn.id()));
    }

    fn connection_failed(&mut self, _: &mut Client, conn: &mut Connection) {
        self.events.push_back(StreamEvent::ConnectionFailed(conn.id()));
    }

    fn connection_packet_lost(
        &mut self, _: &mut Client, conn: &mut Connection, data: &[u8]
    ) {
        self.events.push_back(StreamEvent::PacketLost(conn.id(), data.to_vec()));
    }

    fn connection_congestion_state(&mut self, _: &mut Client, conn: &mut Connection, state: bool) {
        self.events.push_back(StreamEvent::ConnectionCongestionState(conn.id(), state));
    }

    fn connection_lost(&mut self, _: &mut Client, conn: &mut Connection) {
        self.events.push_back(StreamEvent::ConnectionLost(conn.id()));
    }

}

