use actix::{clock::sleep as sleep_clock, Actor, Addr};
use std::{
    sync::{Arc, RwLock},
    time::Duration,
};
use tp2::{
    coffee_machine::{CoffeeMachine, ProcessOrder},
    constants::COFFEE_MACHINES,
    errors::Error,
    input_controller::InputController,
};

/// Returns a list of CoffeeMaker.
pub fn get_coffee_machines() -> Vec<Addr<CoffeeMachine>> {
    let mut coffee_makers = Vec::new();
    for i in 0..COFFEE_MACHINES {
        coffee_makers.push(CoffeeMachine { id: i }.start());
    }

    coffee_makers
}

#[actix_rt::main]
async fn main() -> Result<(), Error> {
    let controller = InputController::new(std::env::args().nth(1))?;
    let orders_list = controller.get_orders()?;
    println!("[INPUT CONTROLLER]: ORDERS TO PROCESS {:?}", orders_list);

    // Create coffee machine as actor
    let coffee_machines = get_coffee_machines();

    let orders = Arc::new(RwLock::new(orders_list));
    for coffee_machine in coffee_machines.clone() {
        // Send message to coffee machine to process orders
        coffee_machine.do_send(ProcessOrder {
            orders: orders.clone(),
        });
    }

    // Wait for coffee machine to stop processing orders
    sleep_clock(Duration::from_secs(5)).await;

    Ok(())
}
