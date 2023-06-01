use std::{
    env,
    mem::size_of,
    net::{SocketAddr, UdpSocket},
    process,
    sync::{Arc, Condvar, Mutex},
    thread,
    time::Duration,
    vec,
};

use rand::{thread_rng, Rng};
use tp2::{
    constants::TIMEOUT,
    leader_election::{self, LeaderElection},
};

pub fn id_to_dataaddr(id: usize) -> SocketAddr {
    let port = (2234 + id) as u16;
    SocketAddr::from(([127, 0, 0, 1], port))
}

fn id_missing() -> i32 {
    println!("Number of shop must be specified");
    return -1;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        process::exit(id_missing());
    }
    println!("Sucursal numero {} corriendo", args[1]);
    println!("Cantidad de sucursales: {}", args[2]);
    //Shop running
    shop_member(
        args[1].parse::<usize>().unwrap(),
        args[2].parse::<i32>().unwrap(),
    );
}

fn shop_member(id: usize, shops_amount: i32) {
    println!("[{}] inicio", id);
    //Leader election
    let mut shop_leader = LeaderElection::new(id, shops_amount);
    let socket = UdpSocket::bind(id_to_dataaddr(id)).unwrap();
    let mut buf = [0; 4];
    //Local server logic goes here
    loop {
        if shop_leader.am_i_leader() {
            println!("[{}] soy Lider", id);
            // if thread_rng().gen_range(0, 100) >= 90 {
            //     println!("[{}] me tomo vacaciones", id);
            //     break;
            // }
            socket.set_read_timeout(Some(Duration::new(3, 0)));
            match socket.recv_from(&mut buf) {
                Ok((_, from)) => {
                    socket.send_to("PONG".as_bytes(), from).unwrap();
                    println!("Mande PONG a: {}", from);
                }
                Err(_) => continue,
            }
        } else {
            let leader_id = shop_leader.get_leader_id();
            println!("[{}] pido trabajo al Lider {}", id, leader_id);
            socket
                .send_to("PING".as_bytes(), id_to_dataaddr(leader_id))
                .unwrap();
            socket.set_read_timeout(Some(TIMEOUT)).unwrap();
            if let Ok((size, from)) = socket.recv_from(&mut buf) {
                println!("[{}] trabajando", id);
                thread::sleep(Duration::from_millis(thread_rng().gen_range(1000, 3000)));
            } else {
                // por simplicidad consideramos que cualquier error necesita un lider nuevo
                shop_leader.find_new()
            }
        }
    }

    //shop_leader.stop();

    //thread::sleep(Duration::from_secs(2));
}
