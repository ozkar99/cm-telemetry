mod net;

pub mod dirt;

// Event specifies a way to serialize itself from a net::Packet (slice of bytes).
pub trait Event {
    fn from_packet(packet: &net::Packet) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;
}

// Server implements a generic server that can bind and recv
// exposes the next_event method that returns an unpacked "Event"
pub struct Server<T: Event> {
    srv: net::Server,
    phantom: std::marker::PhantomData<T>, // needed to avoid "type unused" compile error
}

impl<T: Event> Server<T> {
    pub fn new() -> Result<Server<T>, std::io::Error> {
        match net::Server::new() {
            Ok(srv) => Ok(Server {
                srv,
                phantom: std::marker::PhantomData,
            }),
            Err(e) => Err(e),
        }
    }

    pub fn with_address(addr: &'static str) -> Result<Server<T>, std::io::Error> {
        match net::Server::with_addr(addr) {
            Ok(srv) => Ok(Server {
                srv,
                phantom: std::marker::PhantomData,
            }),
            Err(e) => Err(e),
        }
    }

    pub fn next_event(&self) -> Result<T, Box<dyn std::error::Error>> {
        let packet = self.srv.recv()?;
        T::from_packet(&packet)
    }
}
