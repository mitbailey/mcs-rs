// use log::*;
use serialport::SerialPort;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

const TIMEOUT: u64 = 50;

pub struct Serial {
    port: Arc<Mutex<Box<dyn SerialPort>>>,
    recv: [u8; 32],
    write_delay: u64,
}

impl Serial {
    pub fn new(port_name: String, write_delay: u64) -> Result<Serial, serialport::Error> {
        let port = serialport::new(port_name, 9600)
            .timeout(Duration::from_millis(TIMEOUT))
            .open()?;

        // let recv = &mut [0; 32];

        Ok(Serial {
            port: Arc::new(Mutex::new(port)),
            recv: [0; 32],
            write_delay,
        })
    }
    
    pub fn recv_contains(&self, seq: &[u8]) -> bool {
        self.recv.windows(seq.len()).any(|window| window == seq)
    }

    pub fn get_recv(&self) -> [u8; 32] {
        self.recv
    }

    pub fn _write(&mut self, buf: &[u8]) -> Result<(), serialport::Error> {
        log::info!("Writing: {:?}", buf);
        let retval = Ok(self.port.lock().unwrap().write_all(buf)?);
        sleep(Duration::from_millis(self.write_delay));
        retval
    }

    pub fn xfer_sleep(&mut self, buf: &[u8], sleep_time: u64) -> Result<(), serialport::Error> {
        log::info!("Writing: {:?}", buf);
        let retval = Ok(self.port.lock().unwrap().write_all(buf)?);
        sleep(Duration::from_millis(sleep_time));
        retval
    }

    pub fn _read(&mut self) -> Result<usize, serialport::Error> {
        let retval = self.port.lock().unwrap().read(&mut self.recv)?;
        log::info!("Read: {:?}", self.recv);
        Ok(retval)
    }

    pub fn xfer(&mut self, buf: &[u8]) -> Result<usize, serialport::Error> {
        self._write(buf)?;
        self._read()
    }
}