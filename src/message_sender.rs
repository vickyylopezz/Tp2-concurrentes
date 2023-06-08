use std::{
    net::{SocketAddr, UdpSocket},
    sync::Arc,
    time::Duration,
};

use crate::errors::Error;
pub const ACK: &str = "ACK";

pub struct MessageSender {}

impl MessageSender {
    pub fn send(
        socket: Arc<UdpSocket>,
        addr: SocketAddr,
        message: String,
        attempts: Option<usize>,
        timeout: Option<Duration>,
        id: u32,
    ) -> Result<(), Error> {
        let mut attempts = set_attempts(attempts);
        let timeout = set_duration(timeout);
        set_read_timeout(&socket, timeout)?;

        let mut buf: [u8; 10] = [0; 10];
        while attempts > 0 {
            attempts -= 1;
            send_message(&socket, message.clone(), addr, id)?;
            match socket.recv_from(&mut buf) {
                Ok(_) => {
                    let message = convert_to_string(buf, id)?;
                    let message_parsed = message[0..3].to_string();
                    if message_parsed != *ACK {
                        println!("[COFFEE MACHINE {}]: get {}", id, message_parsed);
                        return Err(Error::InvalidMessageFormat);
                    } else {
                        println!("[COFFEE MACHINE {}]: get ACK", id);
                    }
                }
                Err(_) => {
                    println!("[COFFEE MACHINE {}]: timeout", id);
                    continue;
                }
            };
        }

        Ok(())
    }
}

fn convert_to_string(buf: [u8; 10], id: u32) -> Result<String, Error> {
    match std::str::from_utf8(&buf) {
        Ok(msg) => Ok(msg.to_string()),
        Err(_) => {
            println!("[COFFEE MACHINE {}]: get invalid message", id);
            Err(Error::InvalidMessage)
        }
    }
}

fn send_message(
    socket: &Arc<UdpSocket>,
    message: String,
    addr: SocketAddr,
    id: u32,
) -> Result<(), Error> {
    println!("[COFFEE MACHINE {}]: send {} to {}", id, message, addr);
    match socket.send_to(message.as_bytes(), addr) {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::CantSendMessage),
    }
}

fn set_read_timeout(socket: &Arc<UdpSocket>, timeout: Duration) -> Result<(), Error> {
    match socket.set_read_timeout(Some(timeout)) {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::CantSetReadTimeout),
    }
}

fn set_duration(timeout: Option<Duration>) -> Duration {
    match timeout {
        None => Duration::from_secs(10),
        Some(d) => d,
    }
}

fn set_attempts(attempts: Option<usize>) -> usize {
    attempts.unwrap_or(1)
}
