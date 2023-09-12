use std::{
    fs::File,
    io::ErrorKind,
    net::{TcpListener, TcpStream},
};

use crate::{
    led_driver,
    protocom::response::{response::ResponseType, FileHeader, Response, ServerInfo, Status},
};

pub trait PeerSocketInfo {
    fn peer_info_string(&self) -> String;
}

pub trait SelfSocketInfo {
    fn self_info_string(&self) -> String;
}

impl PeerSocketInfo for TcpStream {
    fn peer_info_string(&self) -> String {
        self.peer_addr()
            .expect("Couldn't get peer info")
            .to_string()
    }
}

impl SelfSocketInfo for TcpStream {
    fn self_info_string(&self) -> String {
        self.local_addr()
            .expect("Couldn't get socket info")
            .to_string()
    }
}

impl SelfSocketInfo for TcpListener {
    fn self_info_string(&self) -> String {
        self.local_addr()
            .expect("Couldn't get socket info")
            .to_string()
    }
}

pub fn led(enable: bool) -> Response {
    let mut new_resp = Response::default();
    let is_successful = led_driver::set_leds(enable);

    let _ = new_resp.response_type.insert(ResponseType::Status(Status {
        status: is_successful,
    }));

    new_resp
}

pub fn info(socket: &TcpStream) -> Response {
    let mut new_resp = Response::default();
    let ip = socket
        .local_addr()
        .expect("Cannot get local addr info from OS.")
        .ip()
        .to_string();
    let port = socket
        .local_addr()
        .expect("Cannot get local addr info from OS.")
        .port()
        .to_string();

    let _ = new_resp
        .response_type
        .insert(ResponseType::ServerInfo(ServerInfo { ip: ip, port: port }));

    new_resp
}

// BUTTON INTERRUPT DETECTION NOT WORKING
pub fn button_interrupt() -> Response {
    let mut new_resp = Response::default();
    let _ = new_resp
        .response_type
        .insert(ResponseType::Status(Status { status: false }));

    new_resp
}

pub fn file(filename: &str) -> (Response, Option<File>) {
    let mut new_resp = Response::default();
    if filename.contains('/') || filename.contains('\\') || filename.contains("../") {
        let _ = new_resp
            .response_type
            .insert(ResponseType::FileHeader(FileHeader {
                name: "Invalid filename.".to_string(),
                size: 0,
                status: false,
            }));
        return (new_resp, None);
    }
    let filename = filename.trim();
    let filename_full =
        std::path::Path::new("/home/ahmet/Documents/RustProtobufNetworking/server/file_storage")
            .join(filename);
    println!("FULL PATH:{}", filename_full.to_str().unwrap());
    let demanded_file = File::open(filename_full.to_str().unwrap());
    let file = match demanded_file {
        Ok(file) => file,
        Err(err) => match err.kind() {
            ErrorKind::NotFound => {
                let _ = new_resp
                    .response_type
                    .insert(ResponseType::FileHeader(FileHeader {
                        name: "File not found.".to_string(),
                        size: 0,
                        status: false,
                    }));
                return (new_resp, None);
            }
            _ => {
                let _ = new_resp
                    .response_type
                    .insert(ResponseType::FileHeader(FileHeader {
                        name: ("Internal error:".to_string()) + &err.to_string(),
                        size: 0,
                        status: false,
                    }));
                return (new_resp, None);
            }
        },
    };

    (get_file_header_response(filename, &file), Some(file))
}

fn get_file_header_response(filename: &str, file: &File) -> Response {
    let mut new_resp = Response::default();
    let file_size = file.metadata().unwrap().len();
    let _ = new_resp
        .response_type
        .insert(ResponseType::FileHeader(FileHeader {
            name: filename.to_string(),
            size: file_size,
            status: true,
        }));
    new_resp
}
