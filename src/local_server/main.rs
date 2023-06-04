use std::{env, process};

use tp2::{errors::Error, local_server::server::Server};

fn id_missing() -> i32 {
    println!("Number of shop must be specified");
    -1
}

fn parse_arg(args: Vec<String>, id: usize) -> Result<i32, Error> {
    if let Ok(parsed_value) = args[id].parse::<i32>() {
        Ok(parsed_value)
    } else {
        Err(Error::CantGetShopId)
    }
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        process::exit(id_missing());
    }

    let shop_id = parse_arg(args.clone(), 1)?;
    let shop_amount = parse_arg(args, 2)?;
    println!("Nº OF SHOPS: {}", shop_amount);

    // Start shop server
    let server = Server::new(shop_id, shop_amount);
    server.run()?;

    Ok(())
}
