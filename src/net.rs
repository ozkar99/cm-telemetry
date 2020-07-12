use std::net::UdpSocket;

const DEFAULT_ADDRESS: &str = "127.0.0.1:20777";
const MAX_PACKET_SIZE: usize = 2048;

pub struct Server {
    srv: UdpSocket,
}

pub type Packet = Vec<u8>;

impl Server {
    pub fn new() -> Result<Server, std::io::Error> {
        return match UdpSocket::bind(DEFAULT_ADDRESS) {
            Ok(udp_server) => Ok(Server { srv: udp_server }),
            Err(e) => Err(e),
        };
    }

    pub fn with_addr(addr: &'static str) -> Result<Server, std::io::Error> {
        return match UdpSocket::bind(addr) {
            Ok(udp_server) => Ok(Server { srv: udp_server }),
            Err(e) => Err(e),
        };
    }

    pub fn recv(&self) -> Result<Packet, std::io::Error> {
        let mut buf = [0; MAX_PACKET_SIZE];
        return match self.srv.recv_from(&mut buf) {
            Ok((number, _)) => Ok(buf[..number].to_vec()),
            Err(e) => Err(e),
        };
    }
}
