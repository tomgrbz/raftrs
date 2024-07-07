use std::fmt::{Display, Formatter};
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;
use anyhow::Result;
pub struct ConnInfo {
    socket: UdpSocket,
    addr: SocketAddr,
}

impl ConnInfo {
    pub fn new(port: u16) -> Self {
        let socket =
            UdpSocket::bind(("localhost", port)).expect("Unable to create udp socket on localhost");
        let addr = socket.local_addr().expect("Failed to fetch local address");
        ConnInfo { socket, addr }
    }
    pub fn send_msg(&self, buf: &[u8]) -> Result<usize, std::io::Error> {
        self.socket.send_to(buf, self.addr)
    }

    pub fn recv_msg(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr), std::io::Error> {
        self.socket.recv_from(buf)
    }
}

impl Display for ConnInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Addr: {}", self.addr)
    }
}
