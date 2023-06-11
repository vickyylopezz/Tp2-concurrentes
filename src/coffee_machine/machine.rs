use actix::prelude::*;
use std::{
    net::{SocketAddr, UdpSocket},
    sync::Arc,
    thread::sleep,
    time::Duration,
};

use crate::{coffee_machine::orders::Order, errors::Error, message_sender::MessageSender};

const POINTS: &str = "points";

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

        self.handle_process_order(order.clone(), coffee_machine.id);
        self.handle_complete_message(order, coffee_machine.id)?;

        Ok(())
    }
}

impl CoffeeMachine {
    /// Handle sending of messages to server.
    fn send_message(&mut self, message: String, id: u32) -> Result<(), Error> {
        match MessageSender::send(
            self.socket.clone(),
            self.server_addr,
            message,
            None,
            None,
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

        self.handle_process_order(order.clone(), id);
        self.handle_complete_message(order, id)?;

        Ok(())
    }

    /// Handles BLOCK message.
    fn handle_block_message(&mut self, order: Order, id: u32) -> Result<(), Error> {
        let block_message = format!("block {}", order.customer_id);
        match self.send_message(block_message, id) {
            Ok(_) => (),
            Err(err) => match err {
                Error::ClientAlreadyBlocked => {
                    self.handle_client_already_blocked(order, id)?;
                }
                _ => return Err(Error::InvalidMessage),
            },
        }

        Ok(())
    }

    /// Handles process order.
    fn handle_process_order(&mut self, order: Order, id: u32) {
        sleep(Duration::from_secs(2));
        println!(
            "[COFFEE MACHINE {}]: order {:?} already processed",
            id, order.id
        );
    }

    /// Change order's payment method to cash.
    fn handle_not_enough_points(&mut self, order: Order, id: u32) -> Result<(), Error> {
        let complete_message = format!("complete {} {} cash", order.customer_id, order.price);
        self.send_message(complete_message, id)?;

        Ok(())
    }

    /// Handles COMPLETE message.
    fn handle_complete_message(&mut self, order: Order, id: u32) -> Result<(), Error> {
        let complete_message = format!(
            "complete {} {} {}",
            order.customer_id, order.price, order.payment_method
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
}
