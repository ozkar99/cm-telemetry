use crate::Packet;
use std::net::UdpSocket;

const MAX_PACKET_SIZE: usize = 2048;

pub struct Server {
    srv: UdpSocket,
}

impl Server {
    pub fn new(addr: &'static str) -> Result<Server, std::io::Error> {
        match UdpSocket::bind(addr) {
            Ok(udp_server) => Ok(Server { srv: udp_server }),
            Err(e) => Err(e),
        }
    }

    pub fn recv(&self) -> Result<Packet, std::io::Error> {
        let mut buf = [0; MAX_PACKET_SIZE];
        match self.srv.recv_from(&mut buf) {
            Ok((number, _)) => Ok(buf[..number].to_vec()),
            Err(e) => Err(e),
        }
    }
}
