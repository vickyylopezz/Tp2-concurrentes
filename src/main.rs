use std::{
    env,
    process,
};

use tp2::{server::Server,
};

fn id_missing() -> i32 {
    println!("Number of shop must be specified");
    return -1;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        process::exit(id_missing());
    }
    println!("Sucursal numero {} corriendo", args[1]);
    println!("Cantidad de sucursales: {}", args[2]);
    //Shop running
    let server = Server::new(args[1].parse::<i32>().unwrap(), args[2].parse::<i32>().unwrap());
    server.handle_client();
}
