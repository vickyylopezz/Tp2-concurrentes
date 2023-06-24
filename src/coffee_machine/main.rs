use actix::{Actor, Addr, Message};
use actix_rt::System;
use rand::Rng;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    sync::Arc, time::Duration, thread::sleep, pin::Pin, future::Future,
};
use tp2::{
    coffee_machine::{
        input_controller::InputController,
        machine::{CoffeeMachine, Block, Complete, Fail}, orders::Order, self,
    },
    constants::COFFEE_MACHINES,
    errors::Error, message_sender::MessageSender,
};

const POINTS: &str = "points";
const COMPLETED: bool = true;
const FAILED: bool = false;

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
 /// Returns true if order's payment method is points.
 fn pay_with_points(order: Order) -> bool {
    order.payment_method == *POINTS
}

/// Dummy function that returns true if the order has been completed.
/// Returns false if there was a failure.
fn is_completed() -> bool {
    let mut rng = rand::thread_rng();
    let num: u32 = rng.gen_range(0..=10);
    if num <= 3 {
        FAILED
    } else {
        COMPLETED
    }
}

fn set_attempts(attempts: Option<usize>) -> usize {
    attempts.unwrap_or(1)
}

async fn handle_block_message(socket: Arc<UdpSocket>, id: usize, attempts: Option<usize>, coffee_machine: Addr<CoffeeMachine>, order: Order) -> Result<(), Error> {
        let mut attempts = set_attempts(attempts);
        while attempts > 0 {
            attempts -= 1;
            coffee_machine.send(Block {
                order: order.clone(),
            })
            .await;
            match MessageSender::recv(socket.clone(), id as u32, Some(Duration::new(5, 0))){
                Ok(_) => break,
                Err(err) => match  err {
                    Error::ClientAlreadyBlocked => {
                        sleep(Duration::from_secs(10));
                        if pay_with_points(order.clone()) {
                            handle_block_message(socket.clone(), id, Some(attempts), coffee_machine.clone(), order.clone());
                        }
                    },
                    Error::Timeout => continue,
                    _ => return Err(Error::InvalidMessage),

                    
                },
            }        
        }
        Ok(())
}

async fn handle_complete_message(socket: Arc<UdpSocket>, id: usize, attempts: Option<usize>, coffee_machine: Addr<CoffeeMachine>, mut order: Order) -> Result<(), Error>{
    let mut attempts = set_attempts(attempts);
    while attempts > 0 {
        attempts -= 1;
        coffee_machine.send(Complete {
            order: order.clone(),
        })
        .await;
        match MessageSender::recv(socket.clone(), id as u32, Some(Duration::new(5, 0))){
            Ok(_) => break,
            Err(err) => match  err {
                Error::NotEnoughPoints => {
                    order.payment_method = "cash".to_string();
                    handle_complete_message(socket.clone(), id, Some(attempts), coffee_machine.clone(), order.clone());
                },
                _ => return Err(Error::InvalidMessage),

                
            },
        }        
    }
    Ok(())
} 

async fn handle_fail_message(socket: Arc<UdpSocket>, id: usize, attempts: Option<usize>, coffee_machine: Addr<CoffeeMachine>, order: Order) -> Result<(), Error>{
    let mut attempts = set_attempts(attempts);
    while attempts > 0 {
        attempts -= 1;
        coffee_machine.send(Fail {
            order: order.clone(),
        })
        .await;
        match MessageSender::recv(socket.clone(), id as u32, Some(Duration::new(5, 0))){
            Ok(_) => break,
            Err(_) => return Err(Error::InvalidMessage),
        }        
    }
    Ok(())
} 
fn main() -> Result<(), Error> {
    System::new().block_on(async {
        let controller = InputController::new(std::env::args().nth(1), std::env::args().nth(2))?;
        let shop_id = controller.shop_id;
        let orders = controller.get_orders()?;
        let attempts = 3;
        
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
           
            if pay_with_points(order.clone()) {
                handle_block_message(socket.clone(), id, Some(attempts), coffee_machine.clone(), order.clone()).await;
            }

            sleep(Duration::from_secs(3));
            println!(
                "[COFFEE MACHINE {}]: order {:?} already processed",
                id, order.id
            );
            if is_completed() {
                handle_complete_message(socket.clone(), id, Some(attempts), coffee_machine.clone(), order.clone()).await;
            } else {
                handle_fail_message(socket.clone(), id, Some(attempts), coffee_machine, order);
            };
        }

        System::current().stop();
        Ok(())
    })
}