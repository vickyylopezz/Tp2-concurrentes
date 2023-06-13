use actix::prelude::*;
use rand::Rng;
use std::{
    net::{SocketAddr, UdpSocket},
    sync::Arc,
    thread::sleep,
    time::Duration,
};

use crate::{coffee_machine::orders::Order, errors::Error, message_sender::MessageSender};

const POINTS: &str = "points";
const COMPLETED: u32 = 1;

#[derive(Message)]
#[rtype(result = "Result<(), Error>")]
pub struct ProcessOrder {
    pub order: Order,
}

#[derive(Clone)]
pub struct CoffeeMachine {
    pub id: u32,
    pub server_addr: SocketAddr,
    pub socket: Arc<UdpSocket>,
    pub shop_id: u32,
}

impl Actor for CoffeeMachine {
    type Context = Context<Self>;
}

impl Handler<ProcessOrder> for CoffeeMachine {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: ProcessOrder, _ctx: &mut Self::Context) -> Self::Result {
        let coffee_machine = self.clone();
        let order = msg.order;

        if self.pay_with_points(order.clone()) {
            self.handle_block_message(order.clone(), coffee_machine.id)?;
        }

        self.handle_process_order(order, coffee_machine.id)?;

        Ok(())
    }
}

impl CoffeeMachine {
    /// Handles messages to server.
    fn send_message(&mut self, message: String, id: u32) -> Result<(), Error> {
        match MessageSender::send(
            self.socket.clone(),
            self.server_addr,
            message,
            None,
            Some(Duration::new(10, 0)),
            id,
        ) {
            Ok(_) => (),
            Err(err) => return Err(err),
        }

        Ok(())
    }

    /// Returns true if order's payment method is points.
    fn pay_with_points(&mut self, order: Order) -> bool {
        order.payment_method == *POINTS
    }

    /// Handles ClientAlreadyBlocked message.
    fn handle_client_already_blocked(&mut self, order: Order, id: u32) -> Result<(), Error> {
        if self.pay_with_points(order.clone()) {
            self.handle_block_message(order.clone(), id)?;
        }

        self.handle_process_order(order, id)?;

        Ok(())
    }

    /// Handles BLOCK message.
    fn handle_block_message(&mut self, order: Order, id: u32) -> Result<(), Error> {
        let block_message = format!("block {} {}", order.customer_id, self.shop_id);
        match self.send_message(block_message, id) {
            Ok(_) => (),
            Err(err) => match err {
                Error::ClientAlreadyBlocked => {
                    sleep(Duration::from_secs(10));
                    self.handle_client_already_blocked(order, id)?;
                }
                _ => return Err(Error::InvalidMessage),
            },
        }

        Ok(())
    }

    /// Dummy function that returns true if the order has been completed.
    /// Returns false if there was a failure.
    fn is_completed(&self) -> bool {
        let mut rng = rand::thread_rng();
        let num: u32 = rng.gen_range(0..=1);

        num == COMPLETED
    }

    /// Handles process order.
    fn handle_process_order(&mut self, order: Order, id: u32) -> Result<(), Error> {
        sleep(Duration::from_secs(3));
        println!(
            "[COFFEE MACHINE {}]: order {:?} already processed",
            id, order.id
        );

        if self.is_completed() {
            self.handle_complete_message(order, id)?;
        } else {
            self.handle_fail_message(order, id)?;
        };

        Ok(())
    }

    /// Change order's payment method to cash.
    fn handle_not_enough_points(&mut self, order: Order, id: u32) -> Result<(), Error> {
        let complete_message = format!(
            "complete {} {} cash {}",
            order.customer_id, order.price, self.shop_id
        );
        self.send_message(complete_message, id)?;

        Ok(())
    }

    /// Handles COMPLETE message.
    fn handle_complete_message(&mut self, order: Order, id: u32) -> Result<(), Error> {
        let complete_message = format!(
            "complete {} {} {} {}",
            order.customer_id, order.price, order.payment_method, self.shop_id
        );
        match self.send_message(complete_message, id) {
            Ok(_) => (),
            Err(err) => match err {
                Error::NotEnoughPoints => self.handle_not_enough_points(order, id)?,
                _ => return Err(err),
            },
        }

        Ok(())
    }

    /// Handles FAIL message.
    fn handle_fail_message(&mut self, order: Order, id: u32) -> Result<(), Error> {
        let fail_message = format!("fail {} {}", order.customer_id, self.shop_id);
        self.send_message(fail_message, id)?;

        Ok(())
    }
}
