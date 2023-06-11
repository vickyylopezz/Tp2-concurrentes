use std::{
    net::{SocketAddr, UdpSocket},
    time::Duration,
};

use rand::Rng;

use crate::{
    action::Action, errors::Error, local_server::leader_election::LeaderElection,
    message_parser::MessageParser, payment_method::Method,
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
                            match msg {
                                Action::Block(client_id) => {
                                    self.handle_block_message(from, client_id);
                                }
                                Action::CompleteOrder(client_id, _, method) => match method {
                                    Method::Cash => {
                                        self.handle_cash(from);
                                    }
                                    Method::Points => {
                                        self.handle_points(client_id, from);
                                    }
                                },
                                _ => return Err(Error::InvalidMessage),
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
                        if MessageParser::parse(message.into_owned()).is_ok() {
                            println!("[SERVER FROM SHOP {}]: send ACK to {}", self.shop_id, from);
                            self.socket.send_to("ACK".as_bytes(), from).unwrap();
                        }
                    }
                    Err(_) => continue,
                }
            }
        }
    }

    fn handle_block_message(&self, from: SocketAddr, client_id: u32) {
        let mut rng = rand::thread_rng();
        let num: u32 = rng.gen_range(0..=1);
        println!("[SERVER FROM SHOP {}]: NUM {}", self.shop_id, num);
        if num == 1 {
            println!("[SERVER FROM SHOP {}]: send ACK to {}", self.shop_id, from);
            self.socket.send_to("ACK".as_bytes(), from).unwrap();
        } else {
            println!(
                "[SERVER FROM SHOP {}]: send alreadyBlocked to {}",
                self.shop_id, from
            );
            let message = format!("alreadyBlocked {}", client_id);
            self.socket.send_to(message.as_bytes(), from).unwrap();
        };
    }

    fn handle_cash(&self, from: SocketAddr) {
        println!("[SERVER FROM SHOP {}]: send ACK to {}", self.shop_id, from);
        self.socket.send_to("ACK".as_bytes(), from).unwrap();
    }

    fn handle_points(&self, client_id: u32, from: SocketAddr) {
        let message = format!("notEnough {}", client_id);
        println!(
            "[SERVER FROM SHOP {}]: send NOT ENOUGH to {}",
            self.shop_id, from
        );
        self.socket.send_to(message.as_bytes(), from).unwrap();
    }
}
