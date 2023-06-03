use actix::{Actor, Addr};
use actix_rt::System;
use std::{
    net::{SocketAddr, UdpSocket, IpAddr, Ipv4Addr},
    sync::Arc,
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

pub fn id_to_dataaddr(id: usize) -> SocketAddr {
    let port = (2234 + id) as u16;
    SocketAddr::from(([127, 0, 0, 1], port))
}

fn main() -> Result<(), Error> {
    System::new().block_on(async {
        let controller = InputController::new(std::env::args().nth(1), std::env::args().nth(2))?;
        let shop_id = controller.shop_id.clone();
        let orders = controller.get_orders()?;

        let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let port = 8000;
        let addr = SocketAddr::new(ip_addr, port);

        let socket = Arc::new(UdpSocket::bind(addr).expect("Error when binding server socket"));
        let server_addr = id_to_dataaddr(shop_id as usize);

        // Start coffee machines
        let coffee_machines = get_coffee_machines(socket.clone(), server_addr);
        for (idx, order) in orders.into_iter().enumerate() {
            let id = idx % coffee_machines.len();
            let coffee_machine = coffee_machines[id].clone();
            match coffee_machine
                .send(ProcessOrder {
                    order: order.clone(),
                })
                .await
            {
                Ok(_) => println!("[COFFEE MACHINE {}]: processing order {}", id, order.id),
                Err(_) => return Err(Error::CantSendMessage),
            }
        }

        System::current().stop();
        Ok(())
    })
}
