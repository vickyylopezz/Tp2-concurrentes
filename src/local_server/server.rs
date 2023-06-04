use std::{
    net::{SocketAddr, UdpSocket},
    time::Duration,
};

use crate::{errors::Error, local_server::leader_election::LeaderElection};

pub fn id_to_dataaddr(id: usize) -> SocketAddr {
    let port = (2234 + id) as u16;
    SocketAddr::from(([127, 0, 0, 1], port))
}

pub struct Server {
    pub addr: SocketAddr,
    pub socket: UdpSocket,
    pub shop_id: i32,
    pub shops_amount: i32,
}

impl Server {
    pub fn new(shop_id: i32, shops_amount: i32) -> Server {
        let addr = id_to_dataaddr(shop_id as usize);
        let socket = UdpSocket::bind(addr).expect("Error when binding server socket");
        println!(
            "[SERVER OF SHOP {}]: LISTENING ON PORT {}",
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
                println!("[SERVER FROM SHOP {}]: IM LEADER", self.shop_id);
                let _ = self.socket.set_read_timeout(Some(Duration::new(3, 0)));
                match self.socket.recv_from(&mut buf) {
                    Ok((size, _from)) => {
                        // Handle coffee machine message
                        let message = String::from_utf8_lossy(&buf[..size]);
                        println!(
                            "[SERVER FROM SHOP {}]: RECEIVE {} FROM COFFEE MACHINE",
                            self.shop_id, message
                        );
                    }
                    Err(_) => continue,
                }
            } else {
                match self.socket.recv_from(&mut buf) {
                    Ok((size, _from)) => {
                        // Handle coffee machine message
                        let message = String::from_utf8_lossy(&buf[..size]);
                        println!(
                            "[SERVER FROM SHOP {}]: RECEIVE {} FROM COFFEE MACHINE",
                            self.shop_id, message
                        );
                    }
                    Err(_) => continue,
                }
            }
        }
    }
}
