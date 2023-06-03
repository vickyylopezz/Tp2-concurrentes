use std::{
    mem::size_of,
    net::{SocketAddr, UdpSocket},
    sync::{Arc, Condvar, Mutex},
    thread, vec,
};

use crate::constants::TIMEOUT;
use crate::errors;
use errors::Error;

pub fn id_to_ctrladdr(id: usize) -> SocketAddr {
    let port = (1234 + id) as u16;
    SocketAddr::from(([127, 0, 0, 1], port))
}

pub struct LeaderElection {
    id: usize,
    socket: UdpSocket,
    leader_id: Arc<(Mutex<Option<usize>>, Condvar)>,
    got_ack: Arc<(Mutex<Option<usize>>, Condvar)>,
    stop: Arc<(Mutex<bool>, Condvar)>,
    shops_amount: i32,
}

impl LeaderElection {
    pub fn new(id: usize, shops_amount: i32) -> LeaderElection {
        let mut ret = LeaderElection {
            id,
            socket: UdpSocket::bind(id_to_ctrladdr(id)).unwrap(),
            leader_id: Arc::new((Mutex::new(Some(id)), Condvar::new())),
            got_ack: Arc::new((Mutex::new(None), Condvar::new())),
            stop: Arc::new((Mutex::new(false), Condvar::new())),
            shops_amount,
        };
        let mut clone = ret.clone();
        thread::spawn(move || clone.responder());

        //Find the leader
        ret.find_new();
        ret
    }

    pub fn am_i_leader(&self) -> bool {
        self.get_leader_id() == self.id
    }

    pub fn get_leader_id(&self) -> usize {
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

    pub fn find_new(&mut self) {
        if let Ok(stop_lock) = self.stop.0.lock() {
            if *stop_lock {
                return;
            }
        }
        println!("[{}] buscando lider", self.id);
        if let Ok(mut leader_id_lock) = self.leader_id.0.lock() {
            *leader_id_lock = None
        }

        //Send Election to all the nodes
        match self.safe_send_next(&self.ids_to_msg(b'E', &[self.id]), self.id) {
            Ok(_) => {
                let (leader_id_lock, leader_id_cvar) = &*self.leader_id;
                if leader_id_cvar
                    .wait_while(leader_id_lock.lock().unwrap(), |leader_id| {
                        leader_id.is_none()
                    }).is_ok()
                {}
            }
            Err(_) => {
                //Si ningun nodo contesta, se autoproclama lider
                if let Ok(mut leader_id_lock) = self.leader_id.0.lock() {
                    *leader_id_lock = Some(self.id)
                }
            }
        }
    }

    fn ids_to_msg(&self, header: u8, ids: &[usize]) -> Vec<u8> {
        let mut msg = vec![header];
        msg.extend_from_slice(&ids.len().to_le_bytes());
        for id in ids {
            msg.extend_from_slice(&id.to_le_bytes());
        }
        msg
    }

    fn safe_send_next(&self, msg: &[u8], id: usize) -> Result<(), Error> {
        let next_id = self.next(id);
        if next_id == self.id {
            println!("[{}] enviando {} a {}", self.id, msg[0] as char, next_id);
            return Err(Error::Timeout);
        }
        if let Ok(mut got_ack_lock) = self.got_ack.0.lock() {
            *got_ack_lock = None
        }
        let _ = self.socket.send_to(msg, id_to_ctrladdr(next_id));
        let got_ack =
            self.got_ack
                .1
                .wait_timeout_while(self.got_ack.0.lock().unwrap(), TIMEOUT, |got_it| {
                    got_it.is_none() || got_it.unwrap() != next_id
                });
        if got_ack.unwrap().1.timed_out() {
            match self.safe_send_next(msg, next_id) {
                Ok(_) => (),
                Err(_) => return Err(Error::Timeout),
            }
        }

        Ok(())
    }

    fn responder(&mut self) {
        while !*self.stop.0.lock().unwrap() {
            let vec_capacity =
                1 + size_of::<usize>() + (self.shops_amount as usize + 1) * size_of::<usize>();
            let mut buf = Vec::with_capacity(vec_capacity);
            //let mut buf = [0; 1 + size_of::<usize>() + (self.shops_amount as usize+1) * size_of::<usize>()];
            for _i in 0..vec_capacity {
                buf.push(0);
            }
            let (_size, from) = self.socket.recv_from(&mut buf).unwrap();
            let (msg_type, mut ids) = self.parse_message(&buf);
            match msg_type {
                b'A' => {
                    println!("[{}] recibí ACK de {}", self.id, from);
                    if let Ok(mut got_ack_lock) = self.got_ack.0.lock() {
                        *got_ack_lock = Some(ids[0]);
                    }
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
                    if let Ok(mut leader_id_lock) = self.leader_id.0.lock() {
                        *leader_id_lock = Some(ids[0]);
                    }
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

        let count = usize::from_le_bytes(buf[1..1 + size_of::<usize>()].try_into().unwrap());

        let mut pos = 1 + size_of::<usize>();
        for _id in 0..count {
            ids.push(usize::from_le_bytes(
                buf[pos..pos + size_of::<usize>()].try_into().unwrap(),
            ));
            pos += size_of::<usize>();
        }

        (buf[0], ids)
    }

    fn _stop(&mut self) {
        let (stop_lock, stop_cvar) = &*self.stop;
        if let Ok(mut stop_lock) = stop_lock.lock() {
            *stop_lock = true;
        }
        if stop_cvar.wait_while(stop_lock.lock().unwrap(), |should_stop| *should_stop).is_ok() {
        }
    }

    fn clone(&self) -> LeaderElection {
        LeaderElection {
            id: self.id,
            socket: self.socket.try_clone().unwrap(),
            leader_id: self.leader_id.clone(),
            got_ack: self.got_ack.clone(),
            stop: self.stop.clone(),
            shops_amount: self.shops_amount,
        }
    }
}
