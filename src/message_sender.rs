use std::{
    net::{SocketAddr, UdpSocket},
    sync::Arc,
    time::Duration,
};

use crate::errors::Error;

pub struct MessageSender {}

impl MessageSender {
    pub fn send(
        socket: Arc<UdpSocket>,
        addr: SocketAddr,
        bytes: &[u8],
        attempts: Option<usize>,
        timeout: Option<Duration>,
    ) -> Result<(), Error> {
        let mut _attempts: usize = match attempts {
            None => 1,
            Some(a) => a,
        };

        let _timeout: Duration = match timeout {
            None => Duration::from_secs(10),
            Some(d) => d,
        };

        socket
            .set_read_timeout(Some(_timeout))
            .expect("Failed to set socket timeout");

        let mut buf: [u8; 10] = [0; 10];

        while _attempts > 0 {
            _attempts -= 1;
            match socket.send_to(bytes, addr) {
                Ok(_) => (),
                Err(_) => return Err(Error::CantSendMessage),
            };
            match socket.recv_from(&mut buf) {
                Ok(_) => (),
                Err(_) => continue,
            };
            let s = match std::str::from_utf8(&buf) {
                Ok(v) => v,
                Err(_) => panic!("Invalid UTF-8 sequence"),
            };
            if s != "ACK" {
                return Err(Error::InvalidMessageFormat);
            } else {
                return Ok(());
            }
        }

        Err(Error::CantSendMessage)
    }
}
