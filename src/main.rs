use prost::Message;
use server::controller::{PeerSocketInfo, SelfSocketInfo};
use server::proto;
use server::protocom;

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
    println!("Listening on {}", listener_socket.self_info_string());
    loop {
        let new_incoming_connection = listener_socket.accept();
        if let Ok(new_connection) = new_incoming_connection {
            let mut new_connection = new_connection.0;
            println!("New connection: {}", new_connection.peer_info_string());

            let builder = thread::Builder::new().name(new_connection.peer_info_string());
            builder
                .spawn(move || {
                    loop {
                        let mut file = Default::default();
                        let mut request: Vec<u8> = vec![0; 1024];
                        if let Ok(msg_len) = new_connection.read(&mut request) {
                            let request = &request[..msg_len];
                            let resp = match proto::decode_request_or_panic(request) {
                                proto::Command::Led(enable) => controller::led(enable),
                                proto::Command::Info => controller::info(&new_connection),
                                proto::Command::BtnInterrupt(_) => controller::button_interrupt(),
                                proto::Command::File(filename) => {
                                    file = controller::file(&filename);
                                    if file.1.is_some() {
                                        file.0
                                    } else {
                                        println!("Client demanded unknown file: {}", filename);
                                        file.0
                                    }
                                }
                                proto::Command::Exit => break,
                                _ => panic!("Shouldn't be here!"),
                            };
                            let mut message: Vec<u8> = Vec::new();
                            resp.encode(&mut message).unwrap();
                            new_connection.write(&message).unwrap();

                            if let server::protocom::response::response::ResponseType::FileHeader(
                                fh,
                            ) = resp.response_type.unwrap()
                            {
                                if fh.status {
                                    let mut request: Vec<u8> = vec![0; 1024];
                                    let msg_len = new_connection.read(&mut request).unwrap();
                                    if let proto::Command::FileAccept(accept) =
                                        proto::decode_request_or_panic(&request[..msg_len])
                                    {
                                        if accept {
                                            let mut file_pb =
                                                protocom::response::Response::default();
                                            let mut file = file.1.unwrap();
                                            let mut file_buf: Vec<u8> =
                                                vec![
                                                    0;
                                                    usize::try_from(file.metadata().unwrap().len())
                                                        .unwrap()
                                                ];
                                            file.read(&mut file_buf).unwrap();
                                            let _ = file_pb.response_type.insert(
                                                protocom::response::response::ResponseType::File(
                                                    protocom::response::File { file: file_buf },
                                                ),
                                            );

                                            let mut message: Vec<u8> =
                                                Vec::with_capacity(file_pb.encoded_len());
                                            file_pb.encode(&mut message).unwrap();
                                            println!("Encoded length: {}", file_pb.encoded_len());
                                            new_connection.write_all(&message).unwrap();
                                        }
                                    } else {
                                        panic!("Expected a file accept, got another command type.");
                                    }
                                }
                            }
                        } else {
                            println!("Recieving request failed.")
                        }
                    }
                    println!("Connection closed on {}", new_connection.peer_info_string());
                })
                .expect("Couldn't create a thread.");
        } else {
            println!("Couldn't establish incoming connection.");
        }
    }
}
