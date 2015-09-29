use std::thread;
use std::net::SocketAddr;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use cobalt::{Config, Client, Connection, ConnectionID, Handler};

pub use cobalt::MessageKind as MessageKind;

pub struct Network {
    addr: SocketAddr,
    server_thread: Option<thread::JoinHandle<()>>,
    event_channel: EventChannel,
    connected: bool,
    connection_time: u64
}

impl Network {

    pub fn new(tick_rate: u32, addr: SocketAddr) -> Network {

        let mut handler = EventHandler::new();
        let channel = handler.event_channel();

        let server_thread = thread::spawn(move|| {
            let mut client = Client::new(Config {
                send_rate: tick_rate,
                .. Config::default()
            });
            client.connect(&mut handler, addr);
        });

        Network {
            addr: addr,
            server_thread: Some(server_thread),
            event_channel: channel,
            connected: false,
            connection_time: 0
        }

    }

    pub fn connected(&mut self) -> bool {
        self.connected
    }

    pub fn server_addr(&mut self) -> &SocketAddr {
        &self.addr
    }

    pub fn send(&self, kind: MessageKind, data: Vec<u8>) {
        self.event_channel.send(kind, data);
    }

    pub fn try_recv(&mut self, time: u64) -> Result<EventType, TryRecvError> {

        // Try to reconnect after 3 seconds
        if self.connection_time != 0 && time - self.connection_time > 3000 {
            self.connection_time = 0;
            self.event_channel.reset();
        }

        // Internal event handling
        match self.event_channel.try_recv() {

            Ok(event) => {

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
                    _ => ()
                }

                Ok(event)

            },

            Err(err) => Err(err)

        }

    }

}

impl Drop for Network {
    fn drop(&mut self ) {
        self.event_channel.close();
        self.server_thread.take().unwrap().join().unwrap();
    }
}

// To be moved into Cobalt ----------------------------------------------------

#[derive(Debug)]
pub enum EventType {
    Bind,
    Shutdown,
    Connect,
    Close,
    Tick,
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

pub struct EventChannel {
    receiver: Receiver<EventType>,
    sender: Sender<Command>
}

impl EventChannel {

    pub fn try_recv(&self) -> Result<EventType, TryRecvError> {
        self.receiver.try_recv()
    }

    pub fn send(&self, kind: MessageKind, data: Vec<u8>) {
        self.sender.send(Command::Send(kind, data)).unwrap();
    }

    pub fn reset(&self) {
        self.sender.send(Command::Reset).unwrap();
    }

    pub fn close(&self) {
        self.sender.send(Command::Close).unwrap();
    }

    pub fn send_to(&self, id: ConnectionID, kind: MessageKind, data: Vec<u8>) {
        self.sender.send(Command::SendTo(id, kind, data)).unwrap();
    }

    pub fn shutdown(&self) {
        self.sender.send(Command::Shutdown).unwrap();
    }

}

pub struct EventHandler {
    from_receiver: Option<Receiver<EventType>>,
    from_sender: Sender<EventType>,
    to_receiver: Receiver<Command>,
    to_sender: Option<Sender<Command>>
}

impl EventHandler {

    pub fn new() -> EventHandler {
        let (from_sender, from_receiver) = channel::<EventType>();
        let (to_sender, to_receiver) = channel::<Command>();
        EventHandler {
            from_receiver: Some(from_receiver),
            from_sender: from_sender,
            to_receiver: to_receiver,
            to_sender: Some(to_sender)
        }
    }

    pub fn event_channel(&mut self) -> EventChannel {
        EventChannel {
            receiver: self.from_receiver.take().unwrap(),
            sender: self.to_sender.take().unwrap(),
        }
    }

}

impl Handler<Client> for EventHandler {

    fn connect(&mut self, _: &mut Client) {
        self.from_sender.send(EventType::Connect).unwrap();
    }

    fn tick_connection(
        &mut self,
        client: &mut Client,
        conn: &mut Connection
    ) {

        let id = conn.id();

        // Create events from received connection messages
        for msg in conn.received() {
            self.from_sender.send(EventType::Message(id, msg)).unwrap();
        }

        // Create a tick event
        self.from_sender.send(EventType::Tick).unwrap();

        // Handle commands
        while let Ok(cmd) = self.to_receiver.try_recv() {
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
        self.from_sender.send(EventType::Close).unwrap();
    }

    fn connection(&mut self, _: &mut Client, conn: &mut Connection) {
        self.from_sender.send(EventType::Connection(conn.id())).unwrap();
    }

    fn connection_failed(&mut self, client: &mut Client, conn: &mut Connection) {
        self.from_sender.send(EventType::ConnectionFailed(conn.id())).unwrap();
    }

    fn connection_packet_lost(
        &mut self, _: &mut Client, conn: &mut Connection, data: &[u8]
    ) {
        self.from_sender.send(EventType::PacketLost(conn.id(), data.to_vec())).unwrap();
    }

    fn connection_congestion_state(&mut self, _: &mut Client, conn: &mut Connection, state: bool) {
        self.from_sender.send(EventType::ConnectionCongestionState(conn.id(), state)).unwrap();
    }

    fn connection_lost(&mut self, _: &mut Client, conn: &mut Connection) {
        self.from_sender.send(EventType::ConnectionLost(conn.id())).unwrap();
    }

}

