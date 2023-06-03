use std::{
    net::{SocketAddr, UdpSocket},
    thread, time::Duration,
};

use rand::{thread_rng, Rng};

use crate::{local_server::leader_election::LeaderElection, constants::TIMEOUT};

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

        println!("[SERVER]: Listening on port {}", addr.port());

        // match socket.try_clone() {
        //     Ok(socket_clone) => {
        //         thread::spawn(move || Server::handle_client(socket_clone, orders.len() as u32));
        //     }
        //     Err(_) => return Err(Error::CantCloneSocket),
        // }

        Server { addr, socket , shop_id, shops_amount}
    }

    pub fn handle_client(self) {
        let mut shop_leader = LeaderElection::new(self.shop_id as usize, self.shops_amount);

        loop {
            let mut buf = [0u8; 1024];

            if shop_leader.am_i_leader() {
                println!("[{}] soy Lider", self.shop_id);
                // if thread_rng().gen_range(0, 100) >= 90 {
                //     println!("[{}] me tomo vacaciones", id);
                //     break;
                // }
                self.socket.set_read_timeout(Some(Duration::new(3, 0)));
                match self.socket.recv_from(&mut buf) {
                    Ok((_, from)) => {
                        self.socket.send_to("PONG".as_bytes(), from).unwrap();
                        println!("Mande PONG a: {}", from);
                    }
                    Err(_) => continue,
                }
            } else {
                let leader_id = shop_leader.get_leader_id();
                println!("[{}] pido trabajo al Lider {}", self.shop_id, leader_id);
                self.socket
                    .send_to("PING".as_bytes(), id_to_dataaddr(leader_id))
                    .unwrap();
                self.socket.set_read_timeout(Some(TIMEOUT)).unwrap();
                if let Ok((size, from)) = self.socket.recv_from(&mut buf) {
                    println!("[{}] trabajando", self.shop_id);
                    thread::sleep(Duration::from_millis(thread_rng().gen_range(1000, 3000)));
                } else {
                    // por simplicidad consideramos que cualquier error necesita un lider nuevo
                    shop_leader.find_new()
                }
                // let (size, _) = self.socket
                //     .recv_from(&mut buf)
                //     .expect("Error when receiving data");
                // let message = String::from_utf8_lossy(&buf[..size]);
                // println!("[SERVER]: Receive {}", message);
            }



            
        }
    }
}

// fn shop_member(id: usize, shops_amount: i32) {
//     println!("[{}] inicio", id);
//     //Leader election
//     let mut buf = [0; 4];
//     //Local server logic goes here
//     loop {
        
//     }

//     //shop_leader.stop();

//     //thread::sleep(Duration::from_secs(2));
// }