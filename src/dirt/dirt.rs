use std::error::Error;

use crate::net::Server;
use crate::dirt::event::Event;

pub struct Dirt {
  srv: Server
}

impl Dirt {
  pub fn new() -> Result<Dirt, std::io::Error> {
    return match Server::new() {
      Ok(srv) => Ok(Dirt{srv: srv}),
      Err(e) => Err(e),
    }
  }

  pub fn with_address(addr: &'static str) -> Result<Dirt, std::io::Error> {
    return match Server::with_addr(addr) {
      Ok(srv) => Ok(Dirt{srv: srv}),
      Err(e) => Err(e),
    }
  }

  pub fn next_event(&self) -> Result<Event, Box<dyn Error>> {
    let packet = self.srv.recv()?;
    return Event::from_packet(packet);
  }
}


