use prost::Message;
use server::proto;

use server::controller;
use std::io::Write;
use std::{
    io::Read,
    net::{Ipv4Addr, TcpListener},
    thread,
};

fn main() {
    let listener_socket =
        TcpListener::bind((Ipv4Addr::UNSPECIFIED, 2001)).expect("Cannot create the socket.");
    println!("----------------------------------");
    println!(
        "Listening on {}",
        listener_socket.local_addr().unwrap().to_string()
    );
    loop {
        let new_incoming_connection = listener_socket.accept();
        if let Ok(new_connection) = new_incoming_connection {
            let mut new_connection = new_connection.0;
            println!(
                "New connection: {}",
                new_connection.local_addr().unwrap().to_string()
            );

            let builder =
                thread::Builder::new().name(new_connection.local_addr().unwrap().to_string());
            builder
                .spawn(move || loop {
                    let mut request: Vec<u8> = Vec::new();
                    if let Ok(_) = new_connection.read(&mut request) {
                        let resp = match proto::decode_request_or_panic(request) {
                            proto::Command::Led(enable) => controller::led(enable),
                            proto::Command::Info => controller::info(&new_connection),
                            proto::Command::BtnInterrupt(_) => controller::button_interrupt(),
                            proto::Command::File(filename) => {
                                let file = controller::file(&filename);
                                if file.1.is_some() {
                                    file.0
                                    // TODO: SEND FILE ITSELF
                                } else {
                                    file.0
                                }
                            }
                            proto::Command::Exit => break,
                        };
                        let mut message: Vec<u8> = Vec::new();
                        resp.encode(&mut message).unwrap();
                        new_connection.write(&message).unwrap();
                    } else {
                        println!("Recieving request failed.")
                    }
                })
                .expect("Couldn't create a thread.");
        } else {
            println!("Couldn't establish incoming connection.");
        }
    }
}
