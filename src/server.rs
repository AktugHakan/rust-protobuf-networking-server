use std::net::TcpStream;

use crate::{
    controller::{self, PeerSocketInfo},
    proto::{self, recieve_request, Command},
};

pub fn socket_handler(mut socket: TcpStream) {
    loop {
        println!("-----------------------------------");
        let req = recieve_request(&mut socket);
        if request_action(&mut socket, req) {
            break;
        }
    }
    println!("Connection closed on {}", socket.peer_info_string());
}

fn request_action(socket: &mut TcpStream, request: Command) -> bool {
    let req = match request {
        proto::Command::Led(enable) => controller::led(enable),
        proto::Command::Info => controller::info(&socket),
        proto::Command::BtnInterrupt(_) => controller::button_interrupt(),
        proto::Command::File(filename) => {
            crate::file_op::file_transfer_routine(socket, filename);
            return false;
        }
        proto::Command::Exit => return true,
        _ => panic!("Shouldn't be here!"),
    };
    proto::send_response(req, socket);
    return false;
}
