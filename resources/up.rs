use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
};

fn main() {
    let socket = UdpSocket::bind("127.0.0.1:5556").expect("Error when binding server socket");
    let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let args: Vec<String> = env::args().collect();
    let addr = SocketAddr::new(ip, 3234 + args[1].parse::<u16>().unwrap());
    socket
        .send_to("UP".as_bytes(), addr)
        .expect("Error sending message to server");
}
