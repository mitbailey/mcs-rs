use super::DetectorDriver;
use crate::drivers::serial::Serial;

const WR_DLY: u64 = 100;
const SHORT_NAME: &str = "KI 6485";
const LONG_NAME: &str = "Keithley Instruments 6485 Picoammeter";

pub struct Ki6485 {
    comms: Serial,
}

// Public functions.
impl Ki6485 {
    pub fn new(port_name: String, samples: i32) -> Result<Ki6485, serialport::Error> {
        log::info!(
            "Attempting to connect to {} at port {}.",
            SHORT_NAME,
            port_name
        );

        // Initialize port communications.
        let mut comms = Serial::new(port_name.clone(), WR_DLY)?;

        // Request identification.
        comms.xfer_sleep(b"*RST\r", 400)?;
        comms.xfer(b"*IDN?\r")?;

        if comms.recv_contains(b"KEITHLEY INSTRUMENTS INC.,MODEL 6485") {
            log::info!("Connected to {} at port {}.", SHORT_NAME, port_name);
        } else {
            log::error!("Failed to connect to {} at port {}.", SHORT_NAME, port_name);
            return Err(serialport::Error::new(
                serialport::ErrorKind::NoDevice,
                "Failed to connect to device.",
            ));
        }

        // Set up device.
        // Set up and start up command sequence for the KI 6485.
        comms.xfer(b"SYST:ZCH ON\r")?;
        comms.xfer(b"RANG 2e-9\r")?;
        comms.xfer(b"INIT\r")?;
        comms.xfer(b"SYST:ZCOR:ACQ\r")?; // acquire zero current
        comms.xfer(b"SYST:ZCOR ON\r")?; // perform zero correction
        comms.xfer(b"RANG:AUTO ON\r")?; // enable auto range
        comms.xfer(b"SYST:ZCH OFF\r")?; // disable zero check
        comms.xfer(b"SYST:ZCOR OFF\r")?; // disable zero correction
        comms.xfer(b"AVER ON\r")?;
        comms.xfer(b"AVER:TCON REP\r")?;
        comms.xfer(format!("AVER:COUN {}\r", samples).as_bytes())?; // enable averaging

        log::debug!("Init complete");

        Ok(Ki6485 { comms })
    }

    // Not applicable to all detectors so not part of the interface.
    pub fn set_samples(&mut self, mut samples: i32) -> Result<(), serialport::Error> {
        // Set samples between 2 and 20
        if samples < 2 {
            samples = 2;
        }
        else if samples > 20 {
            samples = 20;
        }

        self.comms.xfer(format!("AVER:COUN {}\r", samples).as_bytes())?;
        Ok(())
    }
}

// Public interface.
impl DetectorDriver for Ki6485 {
    fn detect(&mut self) -> Result<f64, serialport::Error> {
        self.comms.xfer(b"READ?\r")?;
        
        // Probably a better way to do this.
        // Expected format:
        // MeasurementA,Timestamp,Error
        let mut recv = self.comms.get_recv();
        let mut msg = String::from_utf8(recv.to_vec()).unwrap();
        
        let mut words = msg.split(",");
        
        let mut value = words.next().unwrap().chars();
        let timestamp = words.next().unwrap();
        let error = words.next().unwrap();

        value.next_back();
        let mes = value.as_str().parse::<f64>().unwrap();

        Ok(mes * 1e12) // Convert from amps to picoamps
    }

    fn short_name(&mut self) -> String {
        SHORT_NAME.to_string()
    }

    fn long_name(&mut self) -> String {
        LONG_NAME.to_string()
    }
}

//
// Virtual Device
//
//

pub struct Ki6485Virtual {
    port_name: String,
}

impl Ki6485Virtual {
    pub fn new(port_name: String, samples: i32) -> Ki6485Virtual {
        Ki6485Virtual { port_name }
    }
}

impl DetectorDriver for Ki6485Virtual {
    fn detect(&mut self) -> Result<f64, serialport::Error> {
        Ok(rand::random::<f64>())
    }

    fn short_name(&mut self) -> String {
        SHORT_NAME.to_string()
    }

    fn long_name(&mut self) -> String {
        LONG_NAME.to_string()
    }
}
