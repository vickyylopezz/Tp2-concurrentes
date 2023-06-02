use actix::prelude::*;
use actix_rt::time::sleep;
use std::time::Duration;

use crate::{errors::Error, orders::Order};

#[derive(Message)]
#[rtype(result = "Result<(), Error>")]
pub struct ProcessOrder {
    pub order: Order,
}

#[derive(Clone)]
pub struct CoffeeMachine {
    pub id: u32,
}

impl Actor for CoffeeMachine {
    type Context = Context<Self>;
}

impl Handler<ProcessOrder> for CoffeeMachine {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: ProcessOrder, _ctx: &mut Self::Context) -> Self::Result {
        println!(
            "[COFFEE MACHINE {}]: PROCESSING ORDER {}",
            self.id, msg.order.id
        );
        let coffee_machine = self.clone();

        actix::spawn(async move {
            sleep(Duration::from_secs(2)).await;
            println!(
                "[COFFEE MACHINE {}]: ORDER {:?} ALREADY PROCESSED",
                coffee_machine.id, msg.order.id
            );
        });

        Ok(())
    }
}
