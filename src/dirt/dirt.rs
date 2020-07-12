use std::error::Error;

use crate::dirt::event::Event;
use crate::net::Server;

// Please not that DirtRally2 needs extradata="3"!
pub struct DirtRally2 {
    srv: Server,
}

impl DirtRally2 {
    pub fn new() -> Result<DirtRally2, std::io::Error> {
        return match Server::new() {
            Ok(srv) => Ok(DirtRally2 { srv: srv }),
            Err(e) => Err(e),
        };
    }

    pub fn with_address(addr: &'static str) -> Result<DirtRally2, std::io::Error> {
        return match Server::with_addr(addr) {
            Ok(srv) => Ok(DirtRally2 { srv: srv }),
            Err(e) => Err(e),
        };
    }

    pub fn next_event(&self) -> Result<Event, Box<dyn Error>> {
        let packet = self.srv.recv()?;
        if packet.len() < 255 {
            return Err(Box::from("Packet size is less than 256 bytes, please set extradata=3 on hardware_settings_config.xml"));
        }
        return Event::from_packet(packet);
    }
}
