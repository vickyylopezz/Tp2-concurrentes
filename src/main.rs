use actix::{clock::sleep as sleep_clock, Actor};
use std::time::Duration;
use tp2::{
    coffee_machine::{CoffeeMachine, ProcessOrder},
    errors::Error,
    input_controller::InputController,
};

#[actix_rt::main]
async fn main() -> Result<(), Error> {
    let controller = InputController::new(std::env::args().nth(1))?;
    let orders = controller.get_orders()?;
    println!("[INPUT CONTROLLER]: ORDERS TO PROCESS {:?}", orders);

    // Create coffee machines as actors
    let coffee_machine1 = CoffeeMachine { id: 1 }.start();
    let coffee_machine2 = CoffeeMachine { id: 2 }.start();

    // Send message to coffee machines to process orders
    coffee_machine1.do_send(ProcessOrder {
        order: orders[0].clone(),
    });
    coffee_machine2.do_send(ProcessOrder {
        order: orders[1].clone(),
    });

    // Wait for coffee machines to stop processing orders
    sleep_clock(Duration::from_secs(10)).await;

    Ok(())
}
