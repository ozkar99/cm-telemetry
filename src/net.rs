use std::net::UdpSocket;

#[cfg(feature = "async")]
use tokio::net::UdpSocket as AsyncUdpSocket;

const MAX_PACKET_SIZE: usize = 2048;

pub struct Server {
    srv: UdpSocket,
}

impl Server {
    pub fn new(addr: &str) -> Result<Server, std::io::Error> {
        match UdpSocket::bind(addr) {
            Ok(udp_server) => Ok(Server { srv: udp_server }),
            Err(e) => Err(e),
        }
    }

    pub fn recv(&self) -> Result<Vec<u8>, std::io::Error> {
        let mut buf = [0; MAX_PACKET_SIZE];
        let (number, _) = self.srv.recv_from(&mut buf)?;
        Ok(buf[..number].to_vec())
    }
}

#[cfg(feature = "async")]
pub struct AsyncServer {
    srv: AsyncUdpSocket,
}

#[cfg(feature = "async")]
impl AsyncServer {
    pub async fn new(addr: &str) -> Result<AsyncServer, std::io::Error> {
        match AsyncUdpSocket::bind(addr).await {
            Ok(udp_server) => Ok(AsyncServer { srv: udp_server }),
            Err(e) => Err(e),
        }
    }

    pub async fn recv(&self) -> Result<Vec<u8>, std::io::Error> {
        let mut buf = [0; MAX_PACKET_SIZE];
        let (number, _) = self.srv.recv_from(&mut buf).await?;
        Ok(buf[..number].to_vec())
    }
}
