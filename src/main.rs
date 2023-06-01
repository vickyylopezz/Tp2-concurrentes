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

use error::LeaderError;
use rand::{thread_rng, Rng};
mod error;

const TIMEOUT: Duration = Duration::from_secs(5);

pub fn id_to_ctrladdr(id: usize) -> SocketAddr {
    let port = (1234 + id) as u16;
    SocketAddr::from(([127, 0, 0, 1], port))
}

pub fn id_to_dataaddr(id: usize) -> SocketAddr {
    let port = (2234 + id) as u16;
    SocketAddr::from(([127, 0, 0, 1], port))
}

fn id_missing() -> i32 {
    println!("Number of shop must be specified");
    return -1;
}

struct LeaderElection {
    id: usize,
    socket: UdpSocket,
    leader_id: Arc<(Mutex<Option<usize>>, Condvar)>,
    got_ack: Arc<(Mutex<Option<usize>>, Condvar)>,
    stop: Arc<(Mutex<bool>, Condvar)>,
    shops_amount: i32,
}

impl LeaderElection {
    fn new(id: usize, shops_amount: i32) -> LeaderElection {
        let mut ret = LeaderElection {
            id,
            socket: UdpSocket::bind(id_to_ctrladdr(id)).unwrap(),
            leader_id: Arc::new((Mutex::new(Some(id)), Condvar::new())),
            got_ack: Arc::new((Mutex::new(None), Condvar::new())),
            stop: Arc::new((Mutex::new(false), Condvar::new())),
            shops_amount,
        };
        println!("Entre Leader Election");
        let mut clone = ret.clone();
        thread::spawn(move || clone.responder());

        //Find the leader
        ret.find_new();
        ret
    }

    fn am_i_leader(&self) -> bool {
        self.get_leader_id() == self.id
    }

    fn get_leader_id(&self) -> usize {
        self.leader_id
            .1
            .wait_while(self.leader_id.0.lock().unwrap(), |leader_id| {
                leader_id.is_none()
            })
            .unwrap()
            .unwrap()
    }

    fn next(&self, id: usize) -> usize {
        (id + 1) % self.shops_amount as usize
    }

    fn find_new(&mut self) {
        println!("Entro find_new");
        if *self.stop.0.lock().unwrap() {
            return;
        }
        println!("[{}] buscando lider", self.id);
        *self.leader_id.0.lock().unwrap() = None;

        //Send Election to all the nodes
        match self.safe_send_next(&self.ids_to_msg(b'E', &[self.id]), self.id) {
            Ok(_) => {
                self.leader_id
                    .1
                    .wait_while(self.leader_id.0.lock().unwrap(), |leader_id| {
                        leader_id.is_none()
                    });
            }
            Err(_) => *self.leader_id.0.lock().unwrap() = Some(self.id),
        }
        println!("Salí safe_send_next");
    }

    fn ids_to_msg(&self, header: u8, ids: &[usize]) -> Vec<u8> {
        let mut msg = vec![header];
        msg.extend_from_slice(&ids.len().to_le_bytes());
        for id in ids {
            msg.extend_from_slice(&id.to_le_bytes());
        }
        msg
    }

    fn safe_send_next(&self, msg: &[u8], id: usize) -> Result<(), LeaderError> {
        let next_id = self.next(id);
        println!("Next id: {}", next_id);
        if next_id == self.id {
            println!("[{}] enviando {} a {}", self.id, msg[0] as char, next_id);
            return Err(LeaderError::Timeout);
        }
        *self.got_ack.0.lock().unwrap() = None;
        self.socket.send_to(msg, id_to_ctrladdr(next_id));
        let got_ack =
            self.got_ack
                .1
                .wait_timeout_while(self.got_ack.0.lock().unwrap(), TIMEOUT, |got_it| {
                    got_it.is_none() || got_it.unwrap() != next_id
                });
        if got_ack.unwrap().1.timed_out() {
            println!("Entre timeout");
            match self.safe_send_next(msg, next_id) {
                Ok(_) => (),
                Err(_) => return Err(LeaderError::Timeout),
            }
        }

        Ok(())
    }

    fn responder(&mut self) {
        println!("Entre responder");
        while !*self.stop.0.lock().unwrap() {
            let vec_capacity =
                1 + size_of::<usize>() + (self.shops_amount as usize + 1) * size_of::<usize>();
            let mut buf = Vec::with_capacity(vec_capacity);
            //let mut buf = [0; 1 + size_of::<usize>() + (self.shops_amount as usize+1) * size_of::<usize>()];
            for _i in 0..vec_capacity {
                buf.push(0);
            }
            println!("Intento recibir");
            let (size, from) = self.socket.recv_from(&mut buf).unwrap();
            let (msg_type, mut ids) = self.parse_message(&buf);
            println!("Recibí");
            match msg_type {
                b'A' => {
                    println!("[{}] recibí ACK de {}", self.id, from);
                    *self.got_ack.0.lock().unwrap() = Some(ids[0]);
                    self.got_ack.1.notify_all();
                }
                b'E' => {
                    println!("[{}] recibí Election de {}, ids {:?}", self.id, from, ids);
                    self.socket
                        .send_to(&self.ids_to_msg(b'A', &[self.id]), from)
                        .unwrap();
                    if ids.contains(&self.id) {
                        // dio toda la vuelta, cambiar a COORDINATOR
                        let winner = *ids.iter().max().unwrap();
                        self.socket
                            .send_to(&self.ids_to_msg(b'C', &[winner, self.id]), from)
                            .unwrap();
                    } else {
                        ids.push(self.id);
                        let msg = self.ids_to_msg(b'E', &ids);
                        let clone = self.clone();
                        thread::spawn(move || clone.safe_send_next(&msg, clone.id));
                    }
                }
                b'C' => {
                    println!(
                        "[{}] recibí nuevo coordinador de {}, ids {:?}",
                        self.id, from, ids
                    );
                    *self.leader_id.0.lock().unwrap() = Some(ids[0]);
                    self.leader_id.1.notify_all();
                    self.socket
                        .send_to(&self.ids_to_msg(b'A', &[self.id]), from)
                        .unwrap();
                    if !ids[1..].contains(&self.id) {
                        ids.push(self.id);
                        let msg = self.ids_to_msg(b'C', &ids);
                        let clone = self.clone();
                        thread::spawn(move || clone.safe_send_next(&msg, clone.id));
                    }
                }
                _ => {
                    println!("[{}] ??? {:?}", self.id, ids);
                }
            }
        }
        *self.stop.0.lock().unwrap() = false;
        self.stop.1.notify_all();
    }

    fn parse_message(&self, buf: &[u8]) -> (u8, Vec<usize>) {
        let mut ids = vec![];

        let mut count = usize::from_le_bytes(buf[1..1 + size_of::<usize>()].try_into().unwrap());

        let mut pos = 1 + size_of::<usize>();
        for id in 0..count {
            ids.push(usize::from_le_bytes(
                buf[pos..pos + size_of::<usize>()].try_into().unwrap(),
            ));
            pos += size_of::<usize>();
        }

        (buf[0], ids)
    }

    fn stop(&mut self) {
        *self.stop.0.lock().unwrap() = true;
        self.stop
            .1
            .wait_while(self.stop.0.lock().unwrap(), |should_stop| *should_stop);
    }

    fn clone(&self) -> LeaderElection {
        LeaderElection {
            id: self.id,
            socket: self.socket.try_clone().unwrap(),
            leader_id: self.leader_id.clone(),
            got_ack: self.got_ack.clone(),
            stop: self.stop.clone(),
            shops_amount: self.shops_amount.clone(),
        }
    }
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
    let mut socket = UdpSocket::bind(id_to_dataaddr(id)).unwrap();
    let mut buf = [0; 4];

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

    shop_leader.stop();

    thread::sleep(Duration::from_secs(2));
}
