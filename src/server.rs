use std::net::TcpStream;

use crate::{
    controller::{self, PeerSocketInfo},
    proto::{self, recieve_request},
};

pub fn socket_handler(mut socket: TcpStream) {
    loop {
        println!("-----------------------------------");
        let resp = match recieve_request(&mut socket) {
            proto::Command::Led(enable) => controller::led(enable),
            proto::Command::Info => controller::info(&socket),
            proto::Command::BtnInterrupt(_) => controller::button_interrupt(),
            proto::Command::File(filename) => {
                crate::file_op::file_transfer_routine(&mut socket, filename);
                continue;
            }
            proto::Command::Exit => break,
            _ => panic!("Shouldn't be here!"),
        };
        proto::send_response(resp, &mut socket);
    }
    println!("Connection closed on {}", socket.peer_info_string());
}
