use actix::prelude::*;
use actix_rt::time::sleep;
use std::{time::Duration, sync::Arc, net::{UdpSocket, SocketAddr}};

use crate::orders::Order;

#[derive(Message)]
#[rtype(result = "()")]
pub struct ProcessOrder {
    pub order: Order,
}

#[derive(Clone)]
pub struct CoffeeMachine {
    pub id: u32,
    pub server_addr: SocketAddr,
    pub socket: Arc<UdpSocket>
}

impl Actor for CoffeeMachine {
    type Context = Context<Self>;
}

impl Handler<ProcessOrder> for CoffeeMachine {
    type Result = ();

    fn handle(&mut self, msg: ProcessOrder, _ctx: &mut Self::Context) {
        println!(
            "[COFFEE MACHINE {}]: processing order {}",
            self.id, msg.order.id
        );
        let coffee_machine = self.clone();
        let mensaje = "Orden de prueba".to_string();
        let mensaje_bytes = mensaje.as_bytes();
        let _ = self.socket.send_to(&mensaje_bytes, self.server_addr.clone());

        actix::spawn(async move {
            sleep(Duration::from_secs(2)).await;
            println!(
                "[COFFEE MACHINE {}]: order {:?} already processed",
                coffee_machine.id, msg.order.id
            );
        });
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct BlockCustomer {
    pub customer_id: u32,
}
