use std::{
    io::{Read, Seek, SeekFrom},
    net::TcpStream,
};

use crate::{
    controller,
    proto::{self, recieve_request, send_response, Command},
    MB,
};

use sha2::{Digest, Sha256};

fn send_file(file: &mut std::fs::File, socket: &mut TcpStream) {
    loop {
        if let Command::FileAck(next) = proto::recieve_request(socket) {
            if next.is_some() {
                let next = next.unwrap();
                let file_seg = get_file_segment(file, next);
                let mut file_pb = crate::protocom::response::Response::default();
                let _ = file_pb.response_type.insert(
                    crate::protocom::response::response::ResponseType::File(
                        crate::protocom::response::File {
                            file: Some(file_seg),
                            segment_no: Some(next),
                        },
                    ),
                );

                proto::send_response(file_pb, socket);
            } else {
                break;
            }
        } else {
            break;
        }
    }
    println!("Transfer complete!");
}

fn send_md5(file: &mut std::fs::File, socket: &mut TcpStream) {
    file.seek(SeekFrom::Start(0)).unwrap();
    let mut hasher = Sha256::new();
    let mut buffer = Vec::with_capacity(file.metadata().unwrap().len() as usize);
    file.read_to_end(&mut buffer).unwrap();
    hasher.update(buffer);

    let file_hash = hasher.finalize().to_vec();
    let resp = controller::file_hash(file_hash);
    send_response(resp, socket);
}

pub fn file_transfer_routine(socket: &mut TcpStream, filename: String) {
    let file = controller::file(&filename);
    send_response(file.0, socket);
    if file.2 {
        if let proto::Command::FileAccept(accept) = recieve_request(socket) {
            if accept {
                let mut file = file.1.unwrap();
                send_file(&mut file, socket);
                send_md5(&mut file, socket);
            }
        } else {
            panic!("Expected a file accept, got another command type.");
        }
    }
}

fn get_file_segment(file: &mut std::fs::File, segment_no: u64) -> Vec<u8> {
    file.seek(SeekFrom::Start((MB - 100) * segment_no))
        .expect("Cannot seek that point!");
    let mut file_seg: Vec<u8> = vec![0; (MB as usize) - 100];
    let read_into_len = file.read(&mut file_seg).unwrap();
    file_seg.truncate(read_into_len);
    file_seg
}
