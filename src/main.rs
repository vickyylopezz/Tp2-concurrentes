use actix::{Actor, Addr};
use actix_rt::System;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    sync::Arc,
    thread,
};
use tp2::{
    coffee_machine::{CoffeeMachine, ProcessOrder},
    constants::COFFEE_MACHINES,
    errors::Error,
    input_controller::InputController,
};

/// Creates a list of [`CoffeeMachines`].
fn get_coffee_machines(socket: Arc<UdpSocket>, addr: SocketAddr) -> Vec<Addr<CoffeeMachine>> {
    let mut coffee_makers = Vec::new();
    for i in 0..COFFEE_MACHINES {
        println!("[COFFEE MACHINE {:?}]: starting", i);
        coffee_makers.push(
            CoffeeMachine {
                id: i,
                server_addr: addr,
                socket: socket.clone(),
            }
            .start(),
        );
    }

    coffee_makers
}

fn handle_client(socket: UdpSocket, num_oders: u32) {
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

fn main() -> Result<(), Error> {
    System::new().block_on(async {
        let controller = InputController::new(std::env::args().nth(1))?;
        let orders = controller.get_orders()?;

        // Start local server
        let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let port = 8000;
        let socket_addr = SocketAddr::new(ip_addr, port);
        let socket =
            Arc::new(UdpSocket::bind(socket_addr).expect("Error when binding server socket"));
        println!("[SERVER]: Listening on port 8000");
        let coffee_machines = get_coffee_machines(socket.clone(), socket_addr);

        let mut server_handler = vec![];
        let cloned_socket = socket.try_clone().expect("Error when cloning socket");
        let orders_clone = orders.clone();
        server_handler.push(thread::spawn(move || {
            handle_client(cloned_socket, orders_clone.len() as u32)
        }));

        for (idx, order) in orders.into_iter().enumerate() {
            let coffee_machine = coffee_machines[idx % coffee_machines.len()].clone();
            coffee_machine.send(ProcessOrder { order }).await.unwrap()
        }

        for handler in server_handler {
            handler.join().unwrap();
        }

        System::current().stop();
        Ok(())
    })
}
