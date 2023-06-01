use actix::clock::sleep as sleep_clock;
use actix::prelude::*;
use actix_rt::time::sleep;
use std::time::Duration;
use tp2::{errors::Error, input_controller::InputController, orders::Order};

#[derive(Message)]
#[rtype(result = "()")]
struct ProcessOrder {
    order: Order,
}

#[derive(Clone)]
struct CoffeeMachine {
    id: u32,
}

impl Actor for CoffeeMachine {
    type Context = Context<Self>;
}

impl Handler<ProcessOrder> for CoffeeMachine {
    type Result = ();

    fn handle(&mut self, msg: ProcessOrder, _ctx: &mut Self::Context) {
        println!(
            "[COFFEE MACHINE {}]: processing order {:?}",
            self.id, msg.order.id
        );
        let coffee_machine = self.clone();
        actix::spawn(async move {
            sleep(Duration::from_secs(5)).await;
            println!(
                "[COFFEE MACHINE {}]: already process order {:?}",
                coffee_machine.id, msg.order.id
            );
        });
    }
}

#[actix_rt::main]
async fn main() -> Result<(), Error> {
    let controller = InputController::new(std::env::args().nth(1))?;
    let orders = controller.get_orders()?;
    println!("[INPUT CONTROLLER] ORDERS TO PROCESS: {:?}", orders);

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
