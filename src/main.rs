use actix::{Actor, Addr};
use actix_rt::System;
use std::{
    net::{SocketAddr, UdpSocket},
    sync::Arc,
};
use tp2::{
    coffee_machine::{CoffeeMachine, ProcessOrder},
    constants::COFFEE_MACHINES,
    errors::Error,
    input_controller::InputController,
    server::Server,
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

fn main() -> Result<(), Error> {
    System::new().block_on(async {
        let controller = InputController::new(std::env::args().nth(1))?;
        let orders = controller.get_orders()?;

        // Start local server
        let server = Server::new(orders.clone())?;
        let socket = Arc::new(server.socket);

        // Start coffee machines
        let coffee_machines = get_coffee_machines(socket.clone(), server.addr);
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
