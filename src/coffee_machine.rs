use actix::prelude::*;
use actix_rt::time::sleep;
use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use crate::{errors::Error, orders::Order};

#[derive(Message)]
#[rtype(result = "Result<(), Error>")]
pub struct ProcessOrder {
    pub orders: Arc<RwLock<Vec<Order>>>,
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
        println!("[COFFEE MACHINE {}]: GETTING ORDER", self.id);
        let coffee_machine = self.clone();

        loop {
            let order = if let Ok(mut orders) = msg.orders.write() {
                if !orders.is_empty() {
                    orders.remove(0)
                } else {
                    return Err(Error::NoMoreOrders);
                }
            } else {
                return Err(Error::CantWriteOrdersLock);
            };

            actix::spawn(async move {
                sleep(Duration::from_secs(2)).await;
                println!(
                    "[COFFEE MACHINE {}]: ALREADY PROCESS ORDER {:?}",
                    coffee_machine.id, order.id
                );
            });
        }
    }
}
