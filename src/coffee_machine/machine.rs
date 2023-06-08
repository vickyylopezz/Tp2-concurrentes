use actix::prelude::*;
use std::{
    net::{SocketAddr, UdpSocket},
    sync::Arc,
    thread::sleep,
    time::Duration,
};

use crate::{coffee_machine::orders::Order, errors::Error, message_sender::MessageSender};

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

        // Send BLOCK message
        let block_message = format!("block {}", msg.order.customer_id);
        self.send_message(block_message, coffee_machine.id)?;

        // Process order
        sleep(Duration::from_secs(2));
        println!(
            "[COFFEE MACHINE {}]: order {:?} already processed",
            coffee_machine.id, msg.order.id
        );

        // Send COMPLETE message
        let complete_message = format!(
            "complete {} {} {}",
            msg.order.customer_id, msg.order.price, msg.order.payment_method
        );
        self.send_message(complete_message, coffee_machine.id)?;

        Ok(())
    }
}

impl CoffeeMachine {
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
            Err(e) => {
                println!(
                    "[COFFEE MACHINE {}]: Failed to send message to local server, {:?}",
                    id, e
                );
                return Err(Error::CantSendMessage);
            }
        }

        Ok(())
    }
}
