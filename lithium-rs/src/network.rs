// External Dependencies ------------------------------------------------------
use std::net::SocketAddr;
use std::collections::VecDeque;
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
    state: ClientState<UdpSocket>,
    tick_rate: u32
}

impl Stream {

    pub fn new(addr: SocketAddr) -> Stream {

        let tick_rate = 30;
        let mut handler = StreamHandler::new();
        let mut client = Client::new(Config {
            send_rate: tick_rate,
            connection_init_threshold: 250,
            .. Default::default()
        });

        let state = client.connect_sync(&mut handler, addr).unwrap();

        Stream {
            handler: handler,
            client: client,
            state: state,
            tick_rate: tick_rate
        }

    }

    // Getters ----------------------------------------------------------------
    pub fn id(&self) -> ConnectionID {
        ConnectionID(0)
    }

    pub fn server_addr(&self) -> Result<SocketAddr, ()> {
        self.client.peer_addr().or(Err(()))
    }

    pub fn rtt(&self) -> u32 {
        self.state.rtt()
    }

    pub fn packet_loss(&self) -> f32 {
        self.state.packet_loss()
    }

    pub fn stats(&self) -> Stats {
        self.state.stats()
    }

    pub fn bytes_sent(&self) -> u32 {
        self.state.stats().bytes_sent
    }

    pub fn bytes_received(&self) -> u32 {
        self.state.stats().bytes_received
    }

    pub fn set_tick_rate(&mut self, tick_rate: u32) {
        self.tick_rate = tick_rate;
        self.client.set_config(Config {
            send_rate: tick_rate,
            connection_init_threshold: 250,
            .. Default::default()

        }, &mut self.state)
    }


    // Methods ----------------------------------------------------------------
    pub fn receive(&mut self) {
        self.client.receive_sync(
            &mut self.handler, &mut self.state, 1000 / self.tick_rate
        );
        self.client.tick_sync(&mut self.handler, &mut self.state);
    }

    pub fn recv_message(&mut self) -> Option<StreamEvent> {
        self.handler.try_recv()
    }

    pub fn send(&mut self) {
        self.client.send_sync(&mut self.handler, &mut self.state);
    }

    pub fn send_message(&mut self, kind: MessageKind, typ: Message, data: &Vec<u8>) {
        let mut msg = [typ as u8].to_vec();
        msg.extend(data);
        self.state.send(kind, msg);
    }

    pub fn reset(&mut self) {
        self.state.reset();
    }

    pub fn destroy(&mut self) {
        self.client.close_sync(&mut self.handler, &mut self.state).unwrap();
    }

}


#[derive(Debug)]
pub enum StreamEvent {
    Connect,
    Tick,
    Close,
    Message(ConnectionID, Vec<u8>),
    Connection(ConnectionID),
    ConnectionFailed(ConnectionID),
    ConnectionCongestionState(ConnectionID, bool),
    ConnectionLost(ConnectionID),
    PacketLost(ConnectionID, Vec<u8>)
}

pub enum StreamCommand {
    Send(MessageKind, Vec<u8>)
}


// Network Stream Handler Implementation --------------------------------------
pub struct StreamHandler {
    events: VecDeque<StreamEvent>
}

impl StreamHandler {

    pub fn new() -> StreamHandler {
        StreamHandler {
            events: VecDeque::new()
        }
    }

    pub fn try_recv(&mut self) -> Option<StreamEvent> {
        self.events.pop_front()
    }

}

impl Handler<Client> for StreamHandler {

    fn connect(&mut self, _: &mut Client) {
        self.events.push_back(StreamEvent::Connect);
    }

    fn tick_connection(
        &mut self,
        _: &mut Client,
        conn: &mut Connection
    ) {

        let id = conn.id();
        for msg in conn.received() {
            self.events.push_back(StreamEvent::Message(id, msg));
        }

        self.events.push_back(StreamEvent::Tick);

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

