use std::{
    fs::File,
    io::ErrorKind,
    net::{TcpListener, TcpStream},
};

use crate::{
    led_driver,
    protocom::response::{
        response::ResponseType, FileHash, FileHeader, Response, ServerInfo, Status,
    },
    MB,
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
        status: Some(is_successful),
    }));

    new_resp
}

pub fn file_hash(hash: Vec<u8>) -> Response {
    let mut new_resp = Response::default();
    let _ = new_resp
        .response_type
        .insert(ResponseType::FileHash(FileHash { digest: Some(hash) }));

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
        .insert(ResponseType::ServerInfo(ServerInfo {
            ip: Some(ip),
            port: Some(port),
        }));

    new_resp
}

// BUTTON INTERRUPT DETECTION NOT WORKING
pub fn button_interrupt() -> Response {
    let mut new_resp = Response::default();
    let _ = new_resp.response_type.insert(ResponseType::Status(Status {
        status: Some(false),
    }));

    new_resp
}

pub fn file(filename: &str) -> (Response, Option<File>, bool) {
    let mut new_resp = Response::default();
    if filename.contains('/') || filename.contains('\\') || filename.contains("../") {
        let _ = new_resp
            .response_type
            .insert(ResponseType::FileHeader(FileHeader {
                name: Some("Invalid filename.".to_string()),
                size: Some(0),
                status: Some(false),
                segment_count: None,
            }));
        return (new_resp, None, false);
    }
    let filename = filename.trim();
    let filename_full = std::path::Path::new("file_storage/").join(filename);
    println!("FULL PATH:{}", filename_full.to_str().unwrap());
    let demanded_file = File::open(filename_full.to_str().unwrap());
    let file = match demanded_file {
        Ok(file) => file,
        Err(err) => match err.kind() {
            ErrorKind::NotFound => {
                println!("User demanded non-existing file '{}'", filename);
                let _ = new_resp
                    .response_type
                    .insert(ResponseType::FileHeader(FileHeader {
                        name: Some("File not found.".to_string()),
                        size: Some(0),
                        status: Some(false),
                        segment_count: None,
                    }));
                return (new_resp, None, false);
            }
            _ => {
                let _ = new_resp
                    .response_type
                    .insert(ResponseType::FileHeader(FileHeader {
                        name: Some(("Internal error:".to_string()) + &err.to_string()),
                        size: Some(0),
                        status: Some(false),
                        segment_count: None,
                    }));
                return (new_resp, None, false);
            }
        },
    };

    (get_file_header_response(filename, &file), Some(file), true)
}

fn get_file_header_response(filename: &str, file: &File) -> Response {
    let mut new_resp = Response::default();
    let file_size = file.metadata().unwrap().len();
    let _ = new_resp
        .response_type
        .insert(ResponseType::FileHeader(FileHeader {
            name: Some(filename.to_string()),
            size: Some(file_size),
            status: Some(true),
            segment_count: Some((file_size + (MB - 100 + 1)) / (MB - 100)),
        }));
    new_resp
}
