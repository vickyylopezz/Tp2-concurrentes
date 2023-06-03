use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    thread,
};

use crate::orders::Order;

pub struct Server {
    pub addr: SocketAddr,
    pub socket: UdpSocket,
}

impl Server {
    pub fn new(orders: Vec<Order>) -> Server {
        let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let port = 8000;
        let addr = SocketAddr::new(ip_addr, port);

        let socket = UdpSocket::bind(addr).expect("Error when binding server socket");
        println!("[SERVER]: Listening on port 8000");

        let socket_clone = socket.try_clone().unwrap();
        thread::spawn(move || {
            Server::handle_client(socket_clone.try_clone().unwrap(), orders.len() as u32)
        });

        Server { addr, socket }
    }

    pub fn handle_client(socket: UdpSocket, num_oders: u32) {
        let mut i = 0;
        loop {
            if i >= num_oders - 1 {
                break;
            }
            let mut buf = [0u8; 1024];
            let (size, client_addr) = socket
                .recv_from(&mut buf)
                .expect("Error when receiving data");
            let message = String::from_utf8_lossy(&buf[..size]);
            println!("[SERVER]: Receive {} from {}", message, client_addr);
            i += 1;
        }
    }
}
