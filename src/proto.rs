use std::{io::ErrorKind, net::TcpStream};

use prost::Message;

use crate::{
    network::TcpWithSize,
    protocom::request::{self, request::RequestType},
};

pub enum Command {
    Led(bool),
    Info,
    BtnInterrupt(u32),
    File(String),
    FileAccept(bool),
    FileAck(Option<u64>),
    Exit,
}

pub enum Response {
    Status,
    Info(String),
    FileHeader(String, u64, bool),
    File(),
    FileHash([u8; 32]),
}

pub fn decode_request_or_panic(msg: &[u8]) -> Command {
    let req = request::Request::decode(msg).expect("Cannot parse request.");
    if req.request_type.is_none() {
        return Command::Exit;
    }
    match req.request_type.unwrap() {
        RequestType::Ledctrl(led_control) => Command::Led(led_control.enable.unwrap()),
        RequestType::Info(_) => Command::Info,
        RequestType::Btnint(btnint) => Command::BtnInterrupt(btnint.timeout_us.unwrap()),
        RequestType::File(file_info) => Command::File(file_info.file_name.unwrap()),
        RequestType::FileAccept(accept) => Command::FileAccept(accept.accept.unwrap()),
        RequestType::FileAck(next) => Command::FileAck(next.next),
    }
}

pub fn send_response<ProtoMessage: Message>(msg: ProtoMessage, socket: &mut TcpStream) {
    let mut message: Vec<u8> = Vec::with_capacity(msg.encoded_len());
    msg.encode(&mut message).unwrap();
    socket.send(&message).expect("Couldn't send message.");
}

pub fn recieve_request(socket: &mut TcpStream) -> Command {
    let req = socket.recieve();
    let req = match req {
        Ok(req) => req,
        Err(err) => match err.kind() {
            ErrorKind::UnexpectedEof => return Command::Exit,
            _ => panic!("Data transfer error"),
        },
    };
    decode_request_or_panic(&req)
}
