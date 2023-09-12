use server::controller::{PeerSocketInfo, SelfSocketInfo};

use std::{
    net::{Ipv4Addr, TcpListener},
    thread,
};

fn main() {
    // Start server
    let listener_socket =
        TcpListener::bind((Ipv4Addr::UNSPECIFIED, 2001)).expect("Cannot create the socket.");
    println!("Listening on {}", listener_socket.self_info_string());

    // Listener server loop
    loop {
        let new_incoming_connection = listener_socket.accept();
        if let Ok(new_connection) = new_incoming_connection {
            let new_connection = new_connection.0;
            println!("New connection: {}", new_connection.peer_info_string());

            // Launch new thread
            let builder = thread::Builder::new().name(new_connection.peer_info_string());
            builder
                .spawn(move || server::server::socket_handler(new_connection))
                .expect("Couldn't create a thread.");
        } else {
            println!("Couldn't establish incoming connection.");
        }
    }
}
