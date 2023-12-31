use actix::{Actor, Addr};
use actix_rt::System;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    sync::Arc,
};
use tp2::{
    coffee_machine::{
        input_controller::InputController,
        machine::{CoffeeMachine, ProcessOrder},
    },
    constants::COFFEE_MACHINES,
    errors::Error,
};

/// Creates a list of [`CoffeeMachine`].
fn get_coffee_machines(
    socket: Arc<UdpSocket>,
    addr: SocketAddr,
    shop_id: u32,
) -> Vec<Addr<CoffeeMachine>> {
    let mut coffee_makers = Vec::new();
    for i in 0..COFFEE_MACHINES {
        println!("[COFFEE MACHINE {:?}]: starting", i);
        coffee_makers.push(
            CoffeeMachine {
                id: i,
                server_addr: addr,
                socket: socket.clone(),
                shop_id,
            }
            .start(),
        );
    }

    coffee_makers
}

pub fn id_to_dataaddr(id: usize) -> SocketAddr {
    let port = (3234 + id) as u16;
    SocketAddr::from(([127, 0, 0, 1], port))
}

fn main() -> Result<(), Error> {
    System::new().block_on(async {
        let controller = InputController::new(std::env::args().nth(1), std::env::args().nth(2))?;
        let shop_id = controller.shop_id;
        let orders = controller.get_orders()?;

        let ip_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let port = 8000 + shop_id as u16;
        let addr = SocketAddr::new(ip_addr, port);

        let socket =
            Arc::new(UdpSocket::bind(addr).expect("Error when binding coffee machines socket"));
        let server_addr = id_to_dataaddr(shop_id as usize);

        // Start coffee machines
        let coffee_machines = get_coffee_machines(socket.clone(), server_addr, shop_id);
        for (idx, order) in orders.into_iter().enumerate() {
            let id = idx % coffee_machines.len();
            let coffee_machine = coffee_machines[id].clone();
            match coffee_machine
                .send(ProcessOrder {
                    order: order.clone(),
                })
                .await
            {
                Ok(_) => (),
                Err(_) => return Err(Error::CantSendMessage),
            }
        }

        System::current().stop();
        Ok(())
    })
}
