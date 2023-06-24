use std::{
    net::{SocketAddr, UdpSocket},
    sync::Arc,
    time::Duration,
};

use crate::{action::Action, errors::Error, message_parser::MessageParser};
pub const ACK: &str = "ACK";

pub struct MessageSender {}

impl MessageSender {
    pub fn send(
        socket: Arc<UdpSocket>,
        addr: SocketAddr,
        message: String,
        attempts: Option<usize>,
        id: u32,
    ) -> Result<(), Error> {
        let mut attempts = set_attempts(attempts);
        // set_read_timeout(&socket, timeout)?;

        // let mut buf = [0u8; 1024];
        // while attempts > 0 {
        //     attempts -= 1;
            send_message(&socket, message.clone(), addr, id)?;
            // match socket.recv_from(&mut buf) {
            //     Ok((size, _from)) => {
            //         let message = String::from_utf8_lossy(&buf[..size]);
            //         println!("[COFFEE MACHINE {}]: get {}", id, message);
            //         if let Ok(received) = MessageParser::parse(message.into_owned()) {
            //             match received {
            //                 Action::NotEnoughPoints(_) => return Err(Error::NotEnoughPoints),
            //                 Action::ClientAlreadyBlocked(_) => {
            //                     return Err(Error::ClientAlreadyBlocked)
            //                 }
            //                 Action::Ack => (),
            //                 _ => return Err(Error::InvalidMessageFormat),
            //             }
            //         }
            //     }
            //     Err(_) => {
            //         println!("[COFFEE MACHINE {}]: timeout", id);
            //         continue;
            //     }
            // };
        // }

        Ok(())
    }

    pub fn recv(socket: Arc<UdpSocket>, id: u32, timeout: Option<Duration>) -> Result<(), Error> {
        let mut buf = [0u8; 1024];
        let timeout = set_duration(timeout);
        set_read_timeout(&socket, timeout)?;
        match socket.recv_from(&mut buf) {
            Ok((size, _from)) => {
                let message = String::from_utf8_lossy(&buf[..size]);
                println!("[COFFEE MACHINE {}]: get {}", id, message);
                if let Ok(received) = MessageParser::parse(message.into_owned()) {
                    match received {
                        Action::NotEnoughPoints(_) => return Err(Error::NotEnoughPoints),
                        Action::ClientAlreadyBlocked(_) => {
                            return Err(Error::ClientAlreadyBlocked)
                        }
                        Action::Ack => return Ok(()),
                        _ => return Err(Error::InvalidMessageFormat),
                    }
                }
            }
            Err(_) => {
                println!("[COFFEE MACHINE {}]: timeout", id);
                return Err(Error::Timeout);
                // continue;
            }
        };
        Ok(())
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