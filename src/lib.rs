pub mod controller;
pub mod led_driver;
pub mod proto;

pub mod protocom {
    pub mod request {
        include!(concat!(env!("OUT_DIR"), "/protocom.request.rs"));
    }

    pub mod response {
        include!(concat!(env!("OUT_DIR"), "/protocom.response.rs"));
    }
}
