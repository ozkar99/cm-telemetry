pub mod dirt;
mod net;

/// Packet is an alias for a vector of bytes
pub type Packet = Vec<u8>;

/// Event specifies a way to serialize itself from a Packet
pub trait Event {
    fn from_packet(packet: &Packet) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;
}

/// Server implements a generic server that can bind and recv packets
/// exposes the next_event method that returns an unpacked "Event"
pub struct Server<T: Event> {
    srv: net::Server,
    phantom: std::marker::PhantomData<T>, // needed to avoid "type unused" compile error
}

impl<T: Event> Server<T> {
    /// new initializes a Server with the given address
    pub fn new(address: &'static str) -> Result<Server<T>, std::io::Error> {
        let srv = net::Server::new(address)?;
        Ok(Server {
            srv,
            phantom: std::marker::PhantomData,
        })
    }

    /// next_event will call recv on the inner UDP server (this blocks)
    /// and will call from_packet from the given T
    pub fn next_event(&self) -> Result<T, Box<dyn std::error::Error>> {
        let packet = self.srv.recv()?;
        T::from_packet(&packet)
    }
}
