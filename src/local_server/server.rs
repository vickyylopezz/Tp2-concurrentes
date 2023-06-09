use std::{
    net::{SocketAddr, UdpSocket},
    time::Duration,
};

use crate::{
    action::Action, errors::Error, local_server::leader_election::LeaderElection,
    message_parser::MessageParser,
};

pub fn id_to_dataaddr(id: usize) -> SocketAddr {
    let port = (2234 + id) as u16;
    SocketAddr::from(([127, 0, 0, 1], port))
}

pub struct Server {
    pub addr: SocketAddr,
    pub socket: UdpSocket,
    pub shop_id: u32,
    pub shops_amount: u32,
}

impl Server {
    pub fn new(shop_id: u32, shops_amount: u32) -> Server {
        let addr = id_to_dataaddr(shop_id as usize);
        let socket = UdpSocket::bind(addr).expect("Error when binding server socket");
        println!(
            "[SERVER OF SHOP {}]: listening on port {}",
            shop_id,
            addr.port()
        );

        Server {
            addr,
            socket,
            shop_id,
            shops_amount,
        }
    }

    pub fn run(self) -> Result<(), Error> {
        let shop_leader = LeaderElection::new(self.shop_id as usize, self.shops_amount);

        loop {
            let mut buf = [0u8; 1024];

            if shop_leader.am_i_leader()? {
                println!("[SERVER FROM SHOP {}]: im leader", self.shop_id);
                let _ = self.socket.set_read_timeout(Some(Duration::new(3, 0)));
                match self.socket.recv_from(&mut buf) {
                    Ok((size, from)) => {
                        let message = String::from_utf8_lossy(&buf[..size]);
                        println!("[SERVER FROM SHOP {}]: get {}", self.shop_id, message);
                        if let Ok(msg) = MessageParser::parse(message.into_owned()) {
                            println!("[SERVER FROM SHOP {}]: send ACK to {}", self.shop_id, from);
                            self.socket.send_to("ACK".as_bytes(), from).unwrap();
                            match msg {
                                Action::Block(_) => {
                                    println!("[SERVER FROM SHOP {}]: to do BLOCK", self.shop_id)
                                }
                                Action::CompleteOrder(_, _, _) => println!(
                                    "[SERVER FROM SHOP {}]: to do COMPLETE ORDER",
                                    self.shop_id
                                ),
                                Action::FailOrder(_) => println!(
                                    "[SERVER FROM SHOP {}]: to do FAIL ORDER",
                                    self.shop_id
                                ),
                            }
                        }
                    }
                    Err(_) => continue,
                }
            } else {
                match self.socket.recv_from(&mut buf) {
                    Ok((size, from)) => {
                        let message = String::from_utf8_lossy(&buf[..size]);
                        println!("[SERVER FROM SHOP {}]: get {}", self.shop_id, message);
                        if let Ok(msg) = MessageParser::parse(message.into_owned()) {
                            println!("[SERVER FROM SHOP {}]: send ACK to {}", self.shop_id, from);
                            self.socket.send_to("ACK".as_bytes(), from).unwrap();
                            match msg {
                                Action::Block(_) => {
                                    println!("[SERVER FROM SHOP {}]: to do BLOCK", self.shop_id)
                                }
                                Action::CompleteOrder(_, _, _) => println!(
                                    "[SERVER FROM SHOP {}]: to do COMPLETE ORDER",
                                    self.shop_id
                                ),
                                Action::FailOrder(_) => println!(
                                    "[SERVER FROM SHOP {}]: to do FAIL ORDER",
                                    self.shop_id
                                ),
                            }
                        }
                    }
                    Err(_) => continue,
                }
            }
        }
    }
}
