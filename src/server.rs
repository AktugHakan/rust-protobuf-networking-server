use std::{io::Read, net::TcpStream};

use crate::{
    controller::{self, PeerSocketInfo},
    proto::{self, recieve_request, send_response},
};

pub fn socket_handler(mut socket: TcpStream) {
    loop {
        println!("-----------------------------------");
        let resp = match recieve_request(&mut socket) {
            proto::Command::Led(enable) => controller::led(enable),
            proto::Command::Info => controller::info(&socket),
            proto::Command::BtnInterrupt(_) => controller::button_interrupt(),
            proto::Command::File(filename) => {
                file_transfer_routine(&mut socket, filename);
                continue;
            }
            proto::Command::Exit => break,
            _ => panic!("Shouldn't be here!"),
        };
        proto::send_response(resp, &mut socket);
    }
    println!("Connection closed on {}", socket.peer_info_string());
}

fn send_file(mut file: std::fs::File, socket: &mut TcpStream) {
    let mut file_pb = crate::protocom::response::Response::default();
    let mut file_buf: Vec<u8> = vec![0; usize::try_from(file.metadata().unwrap().len()).unwrap()];
    file.read(&mut file_buf).unwrap();
    let _ = file_pb
        .response_type
        .insert(crate::protocom::response::response::ResponseType::File(
            crate::protocom::response::File { file: file_buf },
        ));

    proto::send_response(file_pb, socket);
}

fn file_transfer_routine(socket: &mut TcpStream, filename: String) {
    let file = controller::file(&filename);
    send_response(file.0, socket);
    if file.2 {
        if let proto::Command::FileAccept(accept) = recieve_request(socket) {
            if accept {
                send_file(file.1.unwrap(), socket);
            }
        } else {
            panic!("Expected a file accept, got another command type.");
        }
    }
}
