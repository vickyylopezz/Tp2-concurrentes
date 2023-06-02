use actix::{clock::sleep as sleep_clock, Actor, Addr};
use std::time::Duration;
use tp2::{
    coffee_machine::{CoffeeMachine, ProcessOrder},
    constants::COFFEE_MACHINES,
    errors::Error,
    input_controller::InputController,
};

/// Creates a list of [`CoffeeMachines`].
fn get_coffee_machines() -> Vec<Addr<CoffeeMachine>> {
    let mut coffee_makers = Vec::new();
    for i in 0..COFFEE_MACHINES {
        println!("[COFFEE MACHINE {:?}]: STARTING", i);
        coffee_makers.push(CoffeeMachine { id: i }.start());
    }

    coffee_makers
}

#[actix_rt::main]
async fn main() -> Result<(), Error> {
    let controller = InputController::new(std::env::args().nth(1))?;
    let orders = controller.get_orders()?;

    let coffee_machines = get_coffee_machines();
    for (idx, order) in orders.into_iter().enumerate() {
        let coffee_machine = coffee_machines[idx % coffee_machines.len()].clone();
        // Send message to coffee machine to process orders
        coffee_machine.do_send(ProcessOrder { order });
    }

    // Wait for coffee machine to stop processing orders
    sleep_clock(Duration::from_secs(5)).await;

    Ok(())
}
