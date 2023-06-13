use std::{
    collections::VecDeque,
    fs::File,
    io::{BufRead, BufReader, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{
    action::Action, constants::TIMEOUT, errors::Error,
    local_server::leader_election::LeaderElection, message_parser::MessageParser,
    payment_method::Method, points_handler::PointsHandler,
};

pub fn id_to_dataaddr(id: usize) -> SocketAddr {
    let port = (2234 + id) as u16;
    SocketAddr::from(([127, 0, 0, 1], port))
}

pub struct Server {
    pub addr: SocketAddr,
    pub socket: Arc<UdpSocket>,
    pub coffee_machine_socket: Arc<UdpSocket>,
    pub shop_id: u32,
    pub shops_amount: u32,
    pub points_handler: Arc<Mutex<PointsHandler>>,
    pub down: Arc<AtomicBool>,
    pub log: File,
    pub log_down: File,
    pub shop_leader: LeaderElection,
    pub sync: Arc<AtomicBool>,
    pub msg_queue: VecDeque<(String, Action)>,
}

impl Server {
    pub fn new(shop_id: u32, shops_amount: u32) -> Server {
        let addr = id_to_dataaddr(shop_id as usize);
        let socket = Arc::new(UdpSocket::bind(addr).expect("Error when binding server socket"));
        let addr_cm = id_to_dataaddr(shop_id as usize + 1000);
        let coffee_machine_socket =
            Arc::new(UdpSocket::bind(addr_cm).expect("Error when binding coffee_machine socket"));

        println!(
            "[SERVER OF SHOP {}]: listening on port {}",
            shop_id,
            addr.port()
        );
        let points_handler = PointsHandler::new();
        let log_file_name = format!("log_{}.txt", shop_id);
        let log_file = File::create(log_file_name).expect("Error creating de log file");
        let log_down_file_name = format!("log_down_{}.txt", shop_id);
        let log_down_file = File::create(log_down_file_name).expect("Error creating de log file");

        Server {
            addr,
            socket,
            coffee_machine_socket,
            shop_id,
            shops_amount,
            points_handler: Arc::new(Mutex::new(points_handler)),
            down: Arc::new(AtomicBool::new(false)),
            log: log_file,
            log_down: log_down_file,
            shop_leader: LeaderElection::new(shop_id as usize, shops_amount),
            sync: Arc::new(AtomicBool::new(false)),
            msg_queue: VecDeque::new(),
        }
    }

    pub fn run(self) -> Result<(), Error> {
        // let mut shop_leader = LeaderElection::new(self.shop_id as usize, self.shops_amount);
        let mut coffee_machine = self.clone();
        let mut server = self.clone();
        let mut threads_handler: Vec<JoinHandle<Result<(), Error>>> = vec![];
        let coffee_machine_clone = coffee_machine.clone();
        threads_handler.push(thread::spawn(move || loop {
            if coffee_machine.shop_leader.am_i_leader()? {
                // println!(
                //     "[SERVER FROM SHOP {}]: im leader cm",
                //     coffee_machine.shop_id
                // );
                if let Err(err) = coffee_machine.receive_from_coffee_machines_leader() {
                    // println!("[SERVER FROM SHOP {}]: {:?}", coffee_machine.shop_id, err);
                }
            } else {
                // println!(
                //     "[SERVER FROM SHOP {}]: im not leader cm",
                //     coffee_machine.shop_id
                // );
                if let Err(err) = coffee_machine.receive_from_coffee_machines_local_server() {
                    // println!("[SERVER FROM SHOP {}]: {:?}", coffee_machine.shop_id, err);
                }
            }
        }));

        threads_handler.push(thread::spawn(move || loop {
            if server.shop_leader.am_i_leader()? {
                // println!("[SERVER FROM SHOP {}]: im leader server", server.shop_id);
                if let Err(err) = server.receive_from_servers() {
                    // println!("[SERVER FROM SHOP {}]: {:?}", coffee_machine_clone.shop_id, err);
                }
            } else {
                // println!(
                //     "[SERVER FROM SHOP {}]: im not leader server",
                //     server.shop_id
                // );
                if server.receive_from_leader().is_err() {
                    server.shop_leader.find_new();
                };
            }
        }));

        for thread in threads_handler {
            thread.join().expect("Error joining threads")?;
        }
        Ok(())
    }

    fn receive_from_coffee_machines_leader(&mut self) -> Result<(), Error> {
        let mut buf = [0u8; 1024];
        let _ = self.coffee_machine_socket.set_read_timeout(Some(TIMEOUT));
        if !self.sync.load(Ordering::SeqCst){
            match self.coffee_machine_socket.recv_from(&mut buf) {
                Ok((size, from)) => {
                    let message = String::from_utf8_lossy(&buf[..size]).into_owned();
                    println!(
                        "[SERVER FROM SHOP {}]: get {} from {}",
                        self.shop_id, message, from
                    );
                    self.handle_extra_messages(message.clone());
    
                    if let Some(msg) = self.responder_leader(message.clone(), from) {
                        if !self.down.load(Ordering::SeqCst) {
                            self.resend_to_servers(message)
                        };
                        println!(
                            "[SERVER FROM SHOP {}]: send {} to {}",
                            self.shop_id, msg, from
                        );
                        self.socket
                            .send_to(msg.as_bytes(), from)
                            .expect("Error sending message to server");
                    }
                }
                Err(_) => return Err(Error::Timeout),
            }
    
        }
        
        Err(Error::Sync)
    }

    fn receive_from_servers(&mut self) -> Result<(), Error> {
        let mut buf = [0u8; 1024];
        let _ = self.socket.set_read_timeout(Some(TIMEOUT));
        match self.socket.recv_from(&mut buf) {
            Ok((size, from)) => {
                if !self.down.load(Ordering::SeqCst) {
                    let message = String::from_utf8_lossy(&buf[..size]).into_owned();
                    println!(
                        "[SERVER FROM SHOP {}]: get {} from {}",
                        self.shop_id, message, from
                    );
                    if let Some(msg) = self.responder_leader(message.clone(), from) {
                        if !self.sync.load(Ordering::SeqCst) {
                            self.resend_to_servers(message)
                        };
                        println!(
                            "[SERVER FROM SHOP {}]: send {} to {}",
                            self.shop_id, msg, from
                        );
                        self.socket
                            .send_to(msg.as_bytes(), from)
                            .expect("Error sending message to server");
                    }
                    return Ok(());
                }
            }
            Err(_) => return Err(Error::Timeout),
        }

        Err(Error::Down)
    }

    /// Receives messages from the leader
    fn receive_from_leader(&mut self) -> Result<String, Error> {
        let mut buf = [0u8; 1024];
        let _ = self.socket.set_read_timeout(Some(Duration::new(3, 0)));

        match self.socket.recv_from(&mut buf) {
            Ok((size, from)) => {
                if !self.down.load(Ordering::SeqCst) {
                    let message = String::from_utf8_lossy(&buf[..size]).into_owned();
                    println!(
                        "[SERVER FROM SHOP {}]: get {} from {}",
                        self.shop_id, message, from
                    );
                    if let Some(msg) = self.responder_local_server(message.clone(), from) {
                        println!(
                            "[SERVER FROM SHOP {}]: send {} to {}",
                            self.shop_id, msg, from
                        );
                        self.socket
                            .send_to(msg.as_bytes(), from)
                            .expect("Error sending message");
                    }
                    return Ok(message);
                }
                Err(Error::Down)
            }
            Err(_) => Err(Error::Timeout),
        }
    }

    /// Receives messages from the coffees machines
    fn receive_from_coffee_machines_local_server(&mut self) -> Result<(), Error> {
        let mut buf = [0u8; 1024];
        let _ = self.coffee_machine_socket.set_read_timeout(Some(TIMEOUT));
        match self.coffee_machine_socket.recv_from(&mut buf) {
            Ok((size, from)) => {
                if !self.sync.load(Ordering::SeqCst) {
                    let message = String::from_utf8_lossy(&buf[..size]).into_owned();
                    println!(
                        "[SERVER FROM SHOP {}]: get {} from {}",
                        self.shop_id, message, from
                    );
                    let extra = self.handle_extra_messages(message.clone());

                    if !self.down.load(Ordering::SeqCst) {
                        self.resend_message_to_leader(message);
                        return Ok(());
                    } else if extra != Some(Action::Up) {
                        if let Some(msg) = self.responder_local_server(message, from) {
                            println!(
                                "[SERVER FROM SHOP {}]: send {} to {}",
                                self.shop_id, msg, from
                            );
                            self.coffee_machine_socket
                                .send_to(msg.as_bytes(), from)
                                .expect("Error sending message");
                        }
                        return Ok(());
                    }
                }
                Ok(())
            }
            Err(_) => Err(Error::Timeout),
        }
    }

    fn handle_extra_messages(&mut self, message: String) -> Option<Action> {
        if let Ok(msg) = MessageParser::parse(message) {
            match msg {
                Action::Up => {
                    println!("[SERVER FROM SHOP {}]: UP", self.shop_id);
                    self.sync.store(true, Ordering::SeqCst);
                    self.sync_with_leader();
                    return Some(msg);
                }
                Action::Down => {
                    println!("[SERVER FROM SHOP {}]: DOWN", self.shop_id);
                    self.shop_leader.stop();
                    self.down.store(true, Ordering::SeqCst);
                    return Some(msg);
                }
                _ => (),
            }
        }
        None
    }
    fn sync_with_leader(&mut self) {
        self.shop_leader.up();
        self.shop_leader.find_new();
        let log_name = format!("log_{}.txt", self.shop_id);
        let reader = BufReader::new(File::open(log_name).expect("Error opening the log file"));
        let line_count = reader.lines().count();
        let msg = format!("SYNC {}", line_count);
        if let Ok(leader) = self.shop_leader.am_i_leader() {
            if leader {
                //TODO: leader down
                if let Ok(addr) = self.broadcast() {
                    print!("\x1b[31m"); // Texto en color rojo
                    println!("Me constestó: {}", addr);
                    print!("\x1b[0m");
                    self.send_down_log_broadcast();

                    self.resend_message(msg, addr);
                };
            } else {
                let leader_addr = id_to_dataaddr(self.shop_leader.get_leader_id().unwrap());
                self.resend_message(msg, leader_addr);

                self.send_down_log(leader_addr);
            }
        }
        self.down.store(false, Ordering::SeqCst);
        //self.sync.store(false, Ordering::SeqCst);
        println!("ACA FUE LA SINCRONIZACION");
    }

    fn send_down_log_broadcast(&mut self) {
        let log_name = format!("log_down_{}.txt", self.shop_id);
        let reader = BufReader::new(File::open(log_name).expect("Error opening the log file"));
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    for i in 0..self.shops_amount {
                        let addr = id_to_dataaddr(i as usize);
                        if i != self.shop_id {
                            println!(
                                "[SERVER FROM SHOP {}]: send {} to {}",
                                self.shop_id, line, addr
                            );
                            // self.write_log(line.clone());
                            self.socket
                                .send_to(line.as_bytes(), addr)
                                .expect("Error sending message");
                        }
                    }
                }
                Err(err) => {
                    println!("Error reading line: {}", err);
                    return;
                }
            }
        }
    }

    fn broadcast(&mut self) -> Result<SocketAddr, Error> {
        for i in 0..self.shops_amount {
            let addr = id_to_dataaddr(i as usize);
            if i != self.shop_id {
                self.socket.send_to("TRY".as_bytes(), addr).unwrap();
                
            }
        }
        let mut buf = [0u8; 1024];
        self.socket.set_read_timeout(Some(Duration::from_secs(3))).expect("Error setin");
        if let Ok((_, from)) = self.socket.recv_from(&mut buf) {
            println!("[SERVER FROM SHOP {}]: get ACK from {}",
            self.shop_id, from);
            return Ok(from);
        }
        Err(Error::Timeout)
    }

    fn send_down_log(&mut self, leader_addr: SocketAddr) {
        let log_name = format!("log_down_{}.txt", self.shop_id);
        let reader = BufReader::new(File::open(log_name).expect("Error opening the log file"));
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    self.socket
                        .send_to(line.as_bytes(), leader_addr)
                        .expect("Error sending message");
                }
                Err(err) => {
                    println!("Error reading line: {}", err);
                    return;
                }
            }
        }
    }

    fn resend_message(&self, message: String, from: SocketAddr) {
        println!(
            "[SERVER FROM SHOP {}] resend {} to {}",
            self.shop_id, message, from
        );
        self.socket
            .send_to(message.as_bytes(), from)
            .expect("Error sending message");
    }

    pub fn process_action(
        &mut self,
        message: String,
        act: Action,
        from: SocketAddr,
    ) -> Option<String> {
        match act {
            Action::Block(client_id, _) => {
                println!("[SERVER FROM SHOP {}]: BLOCK", self.shop_id);
                if !self.down.load(Ordering::SeqCst) {
                    self.write_log(message);

                    let msg = self.block_client(client_id);     
                    return Some(msg);
                } else {
                    self.write_down_log(message);
                    let msg = self.block_client(client_id);             
                    return Some(msg);
                }
            }
            Action::CompleteOrder(client_id, price, method, _) => {
                println!("[SERVER FROM SHOP {}]: COMPLETE", self.shop_id);
                if !self.down.load(Ordering::SeqCst) {
                    self.write_log(message);

                    let msg = self.complete_order(client_id, price, method);
                    return Some(msg);
                } else {
                    let msg = self.acumulate_points(client_id, method);
                    if msg.is_some() {
                        self.write_down_log(message);
                        return msg;
                    } else {
                        return Some(format!("notEnough {}", client_id));
                    }
                }
            }
            Action::FailOrder(client_id, _) => {
                println!("[SERVER FROM SHOP {}]: FAIL", self.shop_id);
                if !self.down.load(Ordering::SeqCst) {
                    self.write_log(message);
                    if let Ok(mut lock) = self.points_handler.lock() {
                        lock.unblock(client_id);
                    }
                    return Some("ACK".to_string());
                } else {
                    self.write_down_log(message);

                    if let Ok(mut lock) = self.points_handler.lock() {
                        lock.unblock(client_id);
                    }                    
                    return Some("ACK".to_string());
                }
            }
            Action::Try => {
                println!("[SERVER FROM SHOP {}]: TRY", self.shop_id);
                return Some("ACK".to_string());
            }
            Action::Sync(lines) => {
                self.send_sync(lines, from);
            }
            _ => (),
        }
        None
    }

    pub fn responder_leader(&mut self, message: String, from: SocketAddr) -> Option<String> {
        let act = match MessageParser::parse(message.clone()) {
            Ok(m) => m,
            Err(_) => return None,
        };
        if self.sync.load(Ordering::SeqCst) {
            match act {
                Action::SyncStart => None,
                Action::SyncEnd => {
                    while !self.msg_queue.is_empty() {
                        let (message, action) = self.msg_queue.pop_front()?;
                        self.process_action(message, action, from);
                    }
                    self.sync.store(false, Ordering::SeqCst);
                    None
                }
                _ => {
                    self.msg_queue
                        .push_back((message.clone(), MessageParser::parse(message).unwrap()));
                    Some("ACK".to_string())
                }
            }
        } else {
            self.process_action(message, act, from)
        }
    }

    fn send_sync(&mut self, lines: u32, from: SocketAddr) {
        let msg_start: &str = "SYNCSTART";
        self.socket
            .send_to(msg_start.to_string().as_bytes(), from)
            .expect("Error sending message");

        let log_name = format!("log_{}.txt", self.shop_id);
        let reader = BufReader::new(File::open(log_name).expect("Error opening the log file"));
        let lines_to_skip = if lines > 0 { lines } else { 0 };
        for (_, line) in reader.lines().skip(lines_to_skip as usize).enumerate() {
            match line {
                Ok(line) => {
                    println!(
                        "[SERVER FROM SHOP {}]: send {} to {}",
                        self.shop_id, line, from
                    );
                    self.socket
                        .send_to(line.as_bytes(), from)
                        .expect("Error sendind message");
                }
                Err(err) => {
                    println!("Error reading line: {}", err);
                    return;
                }
            }
        }

        let msg_end: &str = "SYNCEND";
        self.socket
            .send_to(msg_end.to_string().as_bytes(), from)
            .expect("Error sending message");
    }

    pub fn responder_local_server(&mut self, message: String, from: SocketAddr) -> Option<String> {
        if let Ok(msg) = MessageParser::parse(message.clone()) {
            match msg {
                Action::Block(client_id, shop_id) => {
                    println!("[SERVER FROM SHOP {}]: BLOCK", self.shop_id);
                    if !self.down.load(Ordering::SeqCst) {
                        self.write_log(message);
                    } else {
                        self.write_down_log(message);
                    }
                    let msg = self.block_client(client_id);

                    if shop_id == self.shop_id {
                        self.coffee_machine_socket
                            .send_to(msg.as_bytes(), coffee_machine_addr(self.shop_id))
                            .expect("Error sending message to coffee machine");
                    }
                    return Some(msg);
                }
                Action::CompleteOrder(client_id, price, method, shop_id) => {
                    println!("[SERVER FROM SHOP {}]: COMPLETE", self.shop_id);
                    if !self.down.load(Ordering::SeqCst) {
                        self.write_log(message);
                        let msg = self.complete_order(client_id, price, method);
                        if shop_id == self.shop_id {
                            self.coffee_machine_socket
                                .send_to(msg.as_bytes(), coffee_machine_addr(self.shop_id))
                                .expect("Error sending message to coffee machine");
                        }
                        return Some(msg);
                    } else {
                        let msg = match self.acumulate_points(client_id, method) {
                            Some(msg) => {
                                self.write_down_log(message);
                                msg
                            }
                            None => format!("notEnough {}", client_id),
                        };

                        if shop_id == self.shop_id {
                            self.coffee_machine_socket
                                .send_to(msg.as_bytes(), coffee_machine_addr(self.shop_id))
                                .expect("Error sending message to coffee machine");
                        }
                        return Some(msg);
                    }
                }
                Action::FailOrder(client_id, shop_id) => {
                    println!("[SERVER FROM SHOP {}]: BLOCK", self.shop_id);
                    if !self.down.load(Ordering::SeqCst) {
                        self.write_log(message);
                    } else {
                        self.write_down_log(message)
                    }
                    if let Ok(mut lock) = self.points_handler.lock() {
                        lock.unblock(client_id);
                    }                    if shop_id == self.shop_id {
                        self.coffee_machine_socket
                            .send_to("ACK".as_bytes(), coffee_machine_addr(self.shop_id))
                            .expect("Error sending message to coffee machine");
                    }
                    return Some("ACK".to_string());
                }
                Action::Try => {
                    return Some("ACK".to_string());
                }
                Action::Sync(lines) => {
                    self.send_sync(lines, from);
                }
                _ => (),
            }
        }
        None
    }

    fn acumulate_points(&mut self, client_id: u32, method: Method) -> Option<String> {
        match method {
            Method::Cash => {
                if let Ok(mut lock) = self.points_handler.lock() {
                    lock.unblock(client_id);
                }                
                Some("ACK".to_string())
            }
            Method::Points => None,
        }
    }
    fn complete_order(&mut self, client_id: u32, price: u32, method: Method) -> String {
        let message = match self.update_points(client_id, price as i32, method) {
            Ok(_) => "ACK".to_string(),
            Err(_) => {
                format!("notEnough {}", client_id)
            }
        };
        if let Ok(mut lock) = self.points_handler.lock() {
            lock.unblock(client_id);
        }
        message
    }

    fn update_points(&mut self, client_id: u32, points: i32, method: Method) -> Result<(), Error> {
        match method {
            Method::Cash => {
                    if let Ok(mut lock) = self.points_handler.lock() {
                        return lock.update_points(client_id, points)
                    } else {
                        return Err(Error::Lock);
                    }
                },
            Method::Points => {
                    if let Ok(mut lock) = self.points_handler.lock() {
                        return lock.update_points(client_id, -points)
                    } else {
                        return Err(Error::Lock);
                    }
                },
        }
    }
    fn write_log(&mut self, message: String) {
        let mut log_msg = message;
        log_msg.push('\n');
        self.log
            .write_all(log_msg.as_bytes())
            .expect("Error writing log file");
    }

    fn write_down_log(&mut self, message: String) {
        let mut log_msg = message;
        log_msg.push('\n');
        self.log_down
            .write_all(log_msg.as_bytes())
            .expect("Error writing log file");
    }

    pub fn block_client(&mut self, client_id: u32) -> String {
        if let Ok(mut lock) = self.points_handler.lock() {
            match lock.block(client_id) {
                Ok(_) => return "ACK".to_string(),
                Err(_) => {
                    return format!("alreadyBlocked {}", client_id)
                }
            }
        } else {
            "Error".to_string()
        }
        
    }

    fn resend_message_to_leader(&mut self, message: String) {
        let leader_id = self.shop_leader.get_leader_id().unwrap();
        let leader_addr = id_to_dataaddr(leader_id);
        println!(
            "[SERVER FROM SHOP {}]: send {} to {}",
            self.shop_id, message, leader_addr
        );

        self.socket
            .send_to(message.as_bytes(), leader_addr)
            .expect("Error sending message to leader");
    }

    fn resend_to_servers(&mut self, message: String) {
        //Send update to the others servers
        for i in 0..self.shops_amount {
            let addr = id_to_dataaddr(i as usize);

            if i != self.shop_id {
                println!(
                    "[SERVER FROM SHOP {}]: send UPDATE to shop {}",
                    self.shop_id, i
                );
                self.socket
                    .send_to(message.as_bytes(), addr)
                    .expect("Error sending message to server");
            }
        }
    }

    fn clone(&self) -> Server {
        Server {
            addr: self.addr,
            socket: self.socket.clone(),
            coffee_machine_socket: self.coffee_machine_socket.clone(),
            shop_id: self.shop_id,
            shops_amount: self.shops_amount,
            points_handler: self.points_handler.clone(),
            down: self.down.clone(),
            log: self
                .log
                .try_clone()
                .expect("Error when trying to clone log file"),
            log_down: self
                .log_down
                .try_clone()
                .expect("Error when trying to clone log file"),
            shop_leader: self.shop_leader.clone_leader_election(),
            sync: self.sync.clone(),
            msg_queue: VecDeque::new(),
        }
    }
}

fn coffee_machine_addr(shop_id: u32) -> SocketAddr {
    let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let port = 8000 + shop_id as u16;
    SocketAddr::new(ip_addr, port)
}
