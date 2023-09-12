use std::{
    io::{Read, Write},
    net::TcpStream,
};

use prost::Message;

use crate::protocom::request::{self, request::RequestType};

pub enum Command {
    Led(bool),
    Info,
    BtnInterrupt(u32),
    File(String),
    FileAccept(bool),
    Exit,
}

pub enum Response {
    Status,
    Info(String),
    FileHeader(String, u64, bool),
    File(), // TODO: type of byte array
}

pub fn decode_request_or_panic(msg: &[u8]) -> Command {
    let req = request::Request::decode(msg).expect("Cannot parse request.");
    if req.request_type.is_none() {
        return Command::Exit;
    }
    match req.request_type.unwrap() {
        RequestType::Ledctrl(led_control) => Command::Led(led_control.enable),
        RequestType::Info(_) => Command::Info,
        RequestType::Btnint(btnint) => Command::BtnInterrupt(btnint.timeout_us),
        RequestType::File(file_info) => Command::File(file_info.file_name),
        RequestType::FileAccept(accept) => Command::FileAccept(accept.accept),
    }
}

pub fn send_response<ProtoMessage: Message>(msg: ProtoMessage, socket: &mut TcpStream) {
    let mut message: Vec<u8> = Vec::with_capacity(msg.encoded_len());
    msg.encode(&mut message).unwrap();
    socket.write(&message).unwrap();
}

pub fn recieve_request(socket: &mut TcpStream) -> Command {
    let mut request: Vec<u8> = vec![0; 1024];
    let msg_len = socket.read(&mut request).unwrap();
    let request = &request[..msg_len];
    decode_request_or_panic(request)
}
