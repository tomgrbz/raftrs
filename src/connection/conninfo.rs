use anyhow::Result;
use std::fmt::{Display, Formatter};
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::UdpSocket;

#[derive(Debug)]
pub struct ConnInfo {
    socket: UdpSocket,
    addr: SocketAddr,
    remote_sock: SocketAddr,
}

impl ConnInfo {
    pub async fn new(port: u16) -> Result<Self> {
        let socket = UdpSocket::bind(("127.0.0.1", 0)).await?;
        let addr = socket.local_addr().expect("Failed to fetch local address");

        let remote_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
        Ok(ConnInfo {
            socket,
            addr,
            remote_sock: remote_addr,
        })
    }
    pub async fn send_msg(&self, buf: &[u8]) -> std::io::Result<usize> {
        loop {
            self.socket.writable().await?;
            match self.socket.try_send_to(buf, self.remote_sock) {
                Ok(n) => {
                    return Ok(n);
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }

    pub fn recv_msg(&self, buf: &mut [u8]) -> std::io::Result<usize> {
        let (len, _) = self.socket.try_recv_from(buf)?;
        Ok(len)
    }

    pub async fn ready_to_read(&self) -> Result<()> {
        let v = self.socket.readable().await;

        if let Err(ref e) = v {
            if e.kind() == io::ErrorKind::WouldBlock {
                return Ok(());
            }
        }
        Ok(())
    }
}

impl Display for ConnInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Addr: {}", self.addr)
    }
}
