use std::{
    mem::size_of,
    net::{SocketAddr, UdpSocket},
    sync::{Arc, Condvar, Mutex},
    thread, vec,
};

use crate::constants::TIMEOUT;
use crate::errors;
use errors::Error;

/// Returns socket address of leader node
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
    shops_amount: u32,
}

impl LeaderElection {
    pub fn new(id: usize, shops_amount: u32) -> LeaderElection {
        let mut leader = LeaderElection {
            id,
            socket: UdpSocket::bind(id_to_ctrladdr(id)).expect("Error when binding server socket"),
            leader_id: Arc::new((Mutex::new(Some(id)), Condvar::new())),
            got_ack: Arc::new((Mutex::new(None), Condvar::new())),
            stop: Arc::new((Mutex::new(false), Condvar::new())),
            shops_amount,
        };
        let mut clone = leader.clone_leader_election();
        thread::spawn(move || clone.run());

        // Find new leader
        leader.find_new();
        leader
    }

    pub fn am_i_leader(&self) -> Result<bool, Error> {
        match self.get_leader_id() {
            Ok(leader_id) => Ok(leader_id == self.id),
            Err(_) => Err(Error::CantGetShopId),
        }
    }

    pub fn get_leader_id(&self) -> Result<usize, Error> {
        if let Ok(leader_id) = self.leader_id.0.lock() {
            match self
                .leader_id
                .1
                .wait_while(leader_id, |leader_id| leader_id.is_none())
            {
                Ok(leader_id_guard) => {
                    if let Some(leader_id) = *leader_id_guard {
                        Ok(leader_id)
                    } else {
                        Err(Error::CantGetLeaderId)
                    }
                }
                Err(_) => Err(Error::CantGetLeaderId),
            }
        } else {
            Err(Error::CantLockLeaderId)
        }
    }

    // Get next shop id
    pub fn next(&self, id: usize) -> usize {
        (id + 1) % self.shops_amount as usize
    }

    // Set leader id value
    fn set_leader_id(&mut self, value: Option<usize>) {
        if let Ok(mut leader_id_lock) = self.leader_id.0.lock() {
            *leader_id_lock = value
        }
    }

    // Find new leader
    pub fn find_new(&mut self) {
        if let Ok(stop_lock) = self.stop.0.lock() {
            if *stop_lock {
                return;
            }
        }
        print!("\x1b[34m");
        println!("[SERVER OF SHOP {}]: Finding new leader", self.id);
        print!("\x1b[0m");
        self.set_leader_id(None);

        // Send ELECTION message to all shops
        match self.safe_send_next(&self.ids_to_msg(b'E', &[self.id]), self.id) {
            Ok(_) => {
                // Wait until there is no leader
                let (leader_id_lock, leader_id_cvar) = &*self.leader_id;
                if let Ok(leader_id_lock) = leader_id_lock.lock() {
                    if leader_id_cvar
                        .wait_while(leader_id_lock, |leader_id| leader_id.is_none())
                        .is_ok()
                    {}
                }
            }
            Err(_) => {
                // If there is no feedback from any node, it becomes leader
                self.set_leader_id(Some(self.id));
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

    // Set got ack value
    fn set_got_ack(&mut self, value: Option<usize>) {
        if let Ok(mut got_ack_lock) = self.got_ack.0.lock() {
            *got_ack_lock = value
        }
    }

    fn safe_send_next(&self, msg: &[u8], id: usize) -> Result<(), Error> {
        let next_id = self.next(id);
        if next_id == self.id {
            return Err(Error::Timeout);
        }
        self.clone_leader_election().set_got_ack(None);

        let _ = self.socket.send_to(msg, id_to_ctrladdr(next_id));

        if let Ok(got_ack_lock) = self.got_ack.0.lock() {
            let got_ack = self
                .got_ack
                .1
                .wait_timeout_while(got_ack_lock, TIMEOUT, |got_it| {
                    got_it.is_none() || got_it.unwrap() != next_id
                });
            if got_ack.unwrap().1.timed_out() {
                match self.safe_send_next(msg, next_id) {
                    Ok(_) => (),
                    Err(_) => return Err(Error::Timeout),
                }
            }
        }

        Ok(())
    }

    // Returns a buffer to receive messages
    fn get_buffer(self) -> Vec<u8> {
        let vec_capacity =
            1 + size_of::<usize>() + (self.shops_amount as usize + 1) * size_of::<usize>();
        let mut buf = Vec::with_capacity(vec_capacity);
        for _i in 0..vec_capacity {
            buf.push(0);
        }

        buf
    }

    fn parse_message(&self, buf: &[u8]) -> Result<(u8, Vec<usize>), Error> {
        let mut ids = vec![];

        if let Ok(value) = buf[1..1 + size_of::<usize>()].try_into() {
            let count = usize::from_le_bytes(value);
            let mut pos = 1 + size_of::<usize>();
            for _id in 0..count {
                if let Ok(value) = buf[pos..pos + size_of::<usize>()].try_into() {
                    ids.push(usize::from_le_bytes(value));
                    pos += size_of::<usize>();
                }
            }

            Ok((buf[0], ids))
        } else {
            Err(Error::CantParseMessage)
        }
    }

    fn receive_message(&mut self, buf: &mut [u8]) -> Result<SocketAddr, Error> {
        let (_size, from) = if let Ok(resp) = self.socket.recv_from(buf) {
            resp
        } else {
            return Err(Error::CantReceiveMessage);
        };
        Ok(from)
    }

    fn add_id_to_got_ack(&mut self, ids: &[usize]) {
        if let Ok(mut got_ack_lock) = self.got_ack.0.lock() {
            *got_ack_lock = Some(ids[0]);
        }
        self.got_ack.1.notify_all();
    }

    fn run(&mut self) -> Result<(), Error> {
        loop {
            let mut buf = self.clone_leader_election().get_buffer();
            let from = self.clone_leader_election().receive_message(&mut buf)?;
            if let Ok(stop_lock) = self.stop.0.lock() {
                if *stop_lock {
                    continue;
                }
            }
            let (msg_type, mut ids) = self.parse_message(&buf)?;
            match msg_type {
                b'A' => {
                    self.clone_leader_election().add_id_to_got_ack(&ids);
                }
                b'E' => {
                    self.socket
                        .send_to(&self.ids_to_msg(b'A', &[self.id]), from)
                        .expect("Error when sending message");
                    if ids.contains(&self.id) {
                        // Message has been sent to all nodes, send COORDINATOR message
                        if let Some(winner) = ids.iter().max() {
                            self.socket
                                .send_to(&self.ids_to_msg(b'C', &[*winner]), from)
                                .expect("Error when sending message");
                        }
                    } else {
                        // Message has not been sent to all nodes, send ELECTION message to next shop
                        ids.push(self.id);
                        let msg = self.ids_to_msg(b'E', &ids);
                        let clone = self.clone_leader_election();
                        thread::spawn(move || clone.safe_send_next(&msg, clone.id));
                    }
                }
                b'C' => {
                    let winner_id = Some(ids[0]);
                    self.clone_leader_election().set_leader_id(winner_id);
                    self.leader_id.1.notify_all();
                    self.socket
                        .send_to(&self.ids_to_msg(b'A', &[self.id]), from)
                        .expect("Error when sending message");
                    if !ids[1..].contains(&self.id) {
                        ids.push(self.id);
                        let msg = self.ids_to_msg(b'C', &ids);
                        let clone = self.clone_leader_election();
                        thread::spawn(move || clone.safe_send_next(&msg, clone.id));
                    }
                }
                _ => {
                    println!("[{}] ??? {:?}", self.id, ids);
                }
            }
        }
    }

    pub fn stop(&mut self) {
        let (stop_lock, _) = &*self.stop;
        if let Ok(mut stop_lock) = stop_lock.lock() {
            *stop_lock = true;
        }
    }

    pub fn up(&mut self) {
        let (stop_lock, _) = &*self.stop;
        if let Ok(mut stop_lock) = stop_lock.lock() {
            *stop_lock = false;
        }
    }

    pub fn clone_leader_election(&self) -> LeaderElection {
        LeaderElection {
            id: self.id,
            socket: self
                .socket
                .try_clone()
                .expect("Error when trying to clone udp socket"),
            leader_id: self.leader_id.clone(),
            got_ack: self.got_ack.clone(),
            stop: self.stop.clone(),
            shops_amount: self.shops_amount,
        }
    }
}
