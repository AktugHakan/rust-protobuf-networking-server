use std::{fs::File, io::ErrorKind, net::TcpStream};

use crate::{
    led_driver,
    protocom::response::{response::ResponseType, FileHeader, Response, ServerInfo, Status},
};

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
    let demanded_file = File::open(filename);
    let file = match demanded_file {
        Ok(file) => file,
        Err(err) => {
            let mut new_resp = Response::default();
            match err.kind() {
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
            }
        }
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
