pub mod controller;
pub mod file_op;
pub mod led_driver;
pub mod network;
pub mod proto;
pub mod server;

pub const MB: u64 = 1024 * 1024;

pub mod protocom {
    pub mod request {
        include!(concat!(env!("OUT_DIR"), "/protocom.request.rs"));
    }

    pub mod response {
        include!(concat!(env!("OUT_DIR"), "/protocom.response.rs"));
    }
}
