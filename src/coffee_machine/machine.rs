use actix::prelude::*;
use std::{
    net::{SocketAddr, UdpSocket},
    sync::Arc,
    thread::sleep,
    time::Duration,
};

use crate::{coffee_machine::orders::Order, message_sender::MessageSender};

#[derive(Message)]
#[rtype(result = "()")]
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
    type Result = ();

    fn handle(&mut self, msg: ProcessOrder, _ctx: &mut Self::Context) {
        let coffee_machine = self.clone();
        let message1 = format!("block {}", msg.order.customer_id).to_string();
        let message_bytes = message1.as_bytes();
        MessageSender::send(
            self.socket.clone(),
            self.server_addr,
            message_bytes,
            None,
            None,
        )
        .expect("Failed to send message to local server");
        // let _ = self.socket.send_to(message_bytes, self.server_addr);

        // Se procesa el pedido
        sleep(Duration::from_secs(2));

        println!(
            "[COFFEE MACHINE {}]: order {:?} already processed",
            coffee_machine.id, msg.order.id
        );

        // let message2 = format!("fail {}", msg.order.customer_id);
        let message2 = format!(
            "complete {} {} {}",
            msg.order.customer_id, msg.order.price, msg.order.payment_method
        )
        .to_string();

        MessageSender::send(
            self.socket.clone(),
            self.server_addr,
            message2.as_bytes(),
            None,
            None,
        )
        .expect("Failed to send ending message to local server");
    }
}
