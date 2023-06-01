use tp2::{errors::Error, input_controller::InputController};

fn main() -> Result<(), Error> {
    let input_controller = InputController::new(std::env::args().nth(1))?;
    let orders_list = input_controller.get_orders()?;
    println!("[INPUT CONTROLLER] ORDERS TO PROCESS: {:?}", orders_list);

    Ok(())
}
