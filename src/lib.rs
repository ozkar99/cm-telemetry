mod net;

pub mod dirt;
pub mod f1;

/// TelemetryPacket is an alias for a vector of bytes
pub type TelemetryPacket = [u8];

/// TelemetryEvent specifies a way to serialize itself from a Packet
pub trait TelemetryEvent {
    fn from_packet(packet: &TelemetryPacket) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;
}

/// TelemetryServer implements a generic server that can bind and recv packets
/// exposes the next_event method that returns an unpacked "TelemetryEvent"
pub struct TelemetryServer<T: TelemetryEvent> {
    srv: net::Server,
    phantom: std::marker::PhantomData<T>, // needed to avoid "type unused" compile error
}

impl<T: TelemetryEvent> TelemetryServer<T> {
    /// new initializes a Server with the given address
    pub fn new(address: &str) -> Result<TelemetryServer<T>, std::io::Error> {
        let srv = net::Server::new(address)?;
        Ok(TelemetryServer {
            srv,
            phantom: std::marker::PhantomData,
        })
    }

    /// next will call recv on the inner UDP server (this blocks)
    /// and will call from_packet from the given T
    pub fn next(&self) -> Result<T, Box<dyn std::error::Error>> {
        let packet = self.srv.recv()?;
        T::from_packet(&packet)
    }
}
