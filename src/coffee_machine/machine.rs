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
const COMPLETED: bool = true;
const FAILED: bool = false;

#[derive(Message)]
#[rtype(result = "Result<(), Error>")]
pub struct Block {
    pub order: Order,
}

#[derive(Message)]
#[rtype(result = "Result<(), Error>")]
pub struct Complete {
    pub order: Order,
}

#[derive(Message)]
#[rtype(result = "Result<(), Error>")]
pub struct Fail {
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

impl Handler<Block> for CoffeeMachine {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: Block, _ctx: &mut Self::Context) -> Self::Result {
        let coffee_machine = self.clone();

        let block_message = format!("block {} {}", msg.order.customer_id, self.shop_id);
        self.send_message(block_message, coffee_machine.id);
        Ok(())
    }
}

impl Handler<Complete> for CoffeeMachine {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: Complete, _ctx: &mut Self::Context) -> Self::Result {
        let coffee_machine = self.clone();
        let order = msg.order;

        let complete_message = format!(
            "complete {} {} {} {}",
            order.customer_id, order.price, order.payment_method, self.shop_id
        );
        self.send_message(complete_message, coffee_machine.id);

        Ok(())
    }
}

impl Handler<Fail> for CoffeeMachine {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: Fail, _ctx: &mut Self::Context) -> Self::Result {
        let coffee_machine = self.clone();
        let order = msg.order;

        let fail_message = format!("fail {} {}", order.customer_id, self.shop_id);
        self.send_message(fail_message, coffee_machine.id)?;

        Ok(())
    }
}

impl CoffeeMachine {
    /// Handles messages to server.
    fn send_message(&mut self, message: String, id: u32) -> Result<(), Error> {
        match MessageSender::send(self.socket.clone(), self.server_addr, message, None, id) {
            Ok(_) => (),
            Err(err) => return Err(err),
        }

        Ok(())
    }
}
