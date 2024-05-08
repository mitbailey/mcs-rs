use super::DetectorDriver;
// use crate::drivers::serial::Serial;

pub struct Ki6485 {
    port_name: String,
}

impl Ki6485 {
    pub fn new(port_name: String, samples: i32) -> Ki6485 {
        Ki6485 {
            port_name,
        }
    }
}

impl DetectorDriver for Ki6485 {
    fn detect(&mut self) -> f64 {
        rand::random::<f64>()
    }
}

pub struct Ki6485Virtual {
    port_name: String,
}

impl Ki6485Virtual {
    pub fn new(port_name: String, samples: i32) -> Ki6485Virtual {
        Ki6485Virtual {
            port_name,
        }
    }
}

impl DetectorDriver for Ki6485Virtual {
    fn detect(&mut self) -> f64 {
        rand::random::<f64>()
    }
}