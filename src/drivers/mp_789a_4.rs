use log::*;
use serialport::SerialPort;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

use super::MotionControllerDriver;

const WR_DLY: u64 = 50; // milliseconds

fn subseq(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

pub struct Mp789a4 {
    port: Arc<Mutex<Box<dyn SerialPort>>>,
    recv: [u8; 32],

    position: i64,

    homing: bool,
    moving: bool,

    move_lock: Arc<Mutex<()>>, // Move attempts should trylock, and abort if locked.
    stop_lock: Arc<Mutex<()>>, // Stop attempts should trylock, and abort if locked.
    backlash_lock: Arc<Mutex<()>>, // Stop attempts should trylock, and abort if locked.
}

impl Mp789a4 {
    pub fn new(port_name: String) -> Result<Mp789a4, serialport::Error> {
        log::info!("Attempting to connect to MP789A4 at port {}.", port_name);

        let mut port = serialport::new(port_name, 9600)
            .timeout(Duration::from_millis(100))
            .open()?;

        let recv = &mut [0; 32];

        // Request identification.
        port.write(b" \r")?;
        port.read(recv)?;
        match &recv[..] {
            b" v2.55\r\n#\r\n" => {
                // Handle case when response is " v2.55\r\n#\r\n"
                log::info!("Uninitialized MP789A4 detected.");
            }
            b" #\r\n" => {
                // Handle case when response is " #\r\n"
                log::info!("Initialized MP789A4 detected.");
            }
            _ => {
                return Err(serialport::Error::new(
                    serialport::ErrorKind::InvalidInput,
                    "Invalid response from MP789A4",
                ));
            }
        }

        let mut dev = Mp789a4 {
            port: Arc::new(Mutex::new(port)),
            recv: [0; 32],
            position: 0,
            homing: false,
            move_lock: Arc::new(Mutex::new(())),
            stop_lock: Arc::new(Mutex::new(())),
            backlash_lock: Arc::new(Mutex::new(())),
            moving: false,
        };

        dev.home();

        Ok(dev)
    }

    fn recv_contains(&self, seq: &[u8]) -> bool {
        self.recv.windows(seq.len()).any(|window| window == seq)
    }

    fn write(&mut self, buf: &[u8]) -> Result<(), serialport::Error> {
        log::info!("Writing to MP789A4: {:?}", buf);
        let retval = Ok(self.port.lock().unwrap().write_all(buf)?);
        sleep(Duration::from_millis(WR_DLY));
        retval
    }

    fn write_sleep(&mut self, buf: &[u8], sleep_time: u64) -> Result<(), serialport::Error> {
        log::info!("Writing to MP789A4: {:?}", buf);
        let retval = Ok(self.port.lock().unwrap().write_all(buf)?);
        sleep(Duration::from_millis(sleep_time));
        retval
    }

    fn read(&mut self) -> Result<usize, serialport::Error> {
        let retval = self.port.lock().unwrap().read(&mut self.recv)?;
        log::info!("Read from MP789A4: {:?}", self.recv);
        Ok(retval)
    }

    fn write_read(&mut self, buf: &[u8]) -> Result<usize, serialport::Error> {
        self.write(buf)?;
        self.read()
    }
}

impl MotionControllerDriver for Mp789a4 {
    fn home(&mut self) -> Result<(), serialport::Error> {

        let _lock = match &self.move_lock.try_lock() {
            Ok(l) => l,
            Err(_) => {
                log::warn!("MP789A4 is moving, aborting home.");
                return Err(serialport::Error::new(
                    serialport::ErrorKind::Unknown,
                    "MP789A4 is moving, aborting home.",
                ));
            }
        };

        // match self.move_lock.try_lock() {
        //     Ok(_) => (),
        //     Err(_) => {
        //         log::warn!("MP789A4 is already homing, aborting.");
        //         Err(serialport::Error::new(
        //             serialport::ErrorKind::Unknown,
        //             "MP789A4 is already moving",
        //         ))
        //     }
        // }

        log::info!("Homing MP789A4.");

        // let mut port = self.port.lock().unwrap();

        // Enable the 789A-4's homing circuit.
        self.write_read(b"A1\r")?;

        // Check limit switch status.
        self.write_read(b"]\r")?;

        // Carries out the 789A-4 homing algorithm as described in the manual.
        if self.recv_contains(b"32") && (!self.recv_contains(b"+") && !self.recv_contains(b"-")) {
            log::info!("Homing switch blocked.");

            // Home switch blocked.
            // Move at constant velocity (23 kHz).
            self.write(b"M+23000\r")?;

            loop {
                // Check limit status every 0.8 seconds.
                self.write_read(b"]\r")?;

                if (self.recv_contains(b"0") || self.recv_contains(b"2"))
                    && (!self.recv_contains(b"+") && !self.recv_contains(b"-"))
                {
                    // Not-on-a-limit-switch status is 0 when stationary, 2 when in motion.
                    break;
                } else if (self.recv_contains(b"64") || self.recv_contains(b"128"))
                    && (!self.recv_contains(b"+") && !self.recv_contains(b"-"))
                {
                    // If we have hit either of the extreme limit switches and stopped.
                    log::error!(
                        "Hit edge limit switch when homing. Does this device have a home sensor?"
                    );
                    // TODO: Should NOT be a serialport error.
                    return Err(serialport::Error::new(
                        serialport::ErrorKind::InvalidInput,
                        "Hit edge limit switch when homing. Does this device have a home sensor?",
                    ));
                }

                sleep(Duration::from_millis(800));
            }

            // Soft stop when homing flag is located.
            self.write(b"@\r")?;
            // Back into home switch 3 motor revolutions.
            self.write(b"-108000\r")?;
            // Go 2 motor revolutions up.
            self.write(b"+72000\r")?;
            // Enable 'high accuracy' circuit.
            self.write(b"A24\r")?;
            // Find edge of home flag at 1000 steps/sec.
            self.write_sleep(b"F1000,0\r", WR_DLY * 7)?;
            // Disable home circuit.
            self.write(b"A0\r")?;
        } else if self.recv_contains(b"0")
            && (!self.recv_contains(b"+") && !self.recv_contains(b"-"))
        {
            // Home switch not blocked.
            // Move at constant velocity (23 kHz).
            self.write(b"M-23000\r")?;

            loop {
                // Check limit status every 0.8 seconds.
                self.write_read(b"]\r")?;

                if (self.recv_contains(b"32") || self.recv_contains(b"34"))
                    && (!self.recv_contains(b"+") && !self.recv_contains(b"-"))
                {
                    // Home-switch-blocked status is 32 when stationary, 34 when in motion.
                    break;
                } else if (self.recv_contains(b"64") || self.recv_contains(b"128"))
                    && (!self.recv_contains(b"+") && !self.recv_contains(b"-"))
                {
                    // haystack.windows(needle.len()).position(|window| window == needle)
                    // If we have hit either of the extreme limit switches and stopped.
                    // TODO: Some 789s don't have a limit switch. In this case, we will need to home using the lower limit switch... ?
                    log::error!(
                        "Hit edge limit switch when homing. Does this device have a home sensor?"
                    );
                    // TODO: Should NOT be a serialport error.
                    return Err(serialport::Error::new(
                        serialport::ErrorKind::InvalidInput,
                        "Hit edge limit switch when homing. Does this device have a home sensor?",
                    ));
                }

                sleep(Duration::from_millis(800));
            }

            // Soft stop when homing flag is located.
            self.write(b"@\r")?;
            // Back into home switch 3 motor revolutions.
            self.write(b"-108000\r")?;
            // Go 2 motor revolutions up.
            self.write(b"+72000\r")?;
            // Enable 'high accuracy' circuit.
            self.write(b"A24\r")?;
            // Find edge of home flag at 1000 steps/sec.
            self.write_sleep(b"F1000,0\r", WR_DLY * 7)?;
            // Disable home circuit.
            self.write(b"A0\r")?;
        } else {
            log::error!("Unknown position to home from: {:?}", self.recv);
            return Err(serialport::Error::new(
                serialport::ErrorKind::InvalidInput,
                "Unknown position to home from",
            ));
        }

        // The standard is for the device drivers to read 0 when homed if the controller does not itself provide a value.
        // It is up to the middleware to handle zero- and home-offsets.
        if self.is_moving()? {
            log::warn!("Post-home movement detected. Entering movement remediation.");
            self.write_sleep(b"@\r", WR_DLY * 10)?;
        }

        let mut stop_attempts = 0;
        while self.is_moving()? {
            if stop_attempts > 3 {
                stop_attempts = 1;
                log::warn!("Re-commanding that device ceases movement.");
                self.write(b"@\r")?;
            }
            stop_attempts += 1;
            log::warn!("Waiting for device to cease movement.");
            sleep(Duration::from_millis(500));
        }

        self.position = 0;
        self.homing = false;

        Ok(())
    }

    fn get_position(&mut self) -> i64 {
        todo!()
    }

    fn stop(&mut self) -> Result<(), serialport::Error> {
        let _lock = match &self.stop_lock.try_lock() {
            Ok(l) => l,
            Err(_) => {
                log::warn!("MP789A4 is already stopping, aborting stop.");
                return Err(serialport::Error::new(
                    serialport::ErrorKind::Unknown,
                    "MP789A4 is stopping, aborting stop.",
                ));
            }
        };

        log::info!("Stopping MP789A4.");
        self.write(b"@\r")?;
        self.write(b"@\r")?;
        self.write(b"@\r")?;

        Ok(())
    }

    fn is_moving(&mut self) -> Result<bool, serialport::Error> {
        // If we cannot acquire the backlash lock, then we are moving.
        match self.backlash_lock.try_lock() {
            Ok(_) => (),
            Err(_) => {
                self.moving = true;
                return Ok(true);
            }
        }

        // If we cannot acquire the movement lock, then we are moving.
        match self.move_lock.try_lock() {
            Ok(_) => (),
            Err(_) => {
                self.moving = true;
                return Ok(true);
            }
        }

        // Finally, ask the device if its moving.
        for i in 0..3 {
            self.write_read(b"^\r")?;
            if self.recv_contains(b"0") && !self.recv_contains(b"+") && !self.recv_contains(b"-") {
                self.moving = false;
                return Ok(false);
            } else {
                self.moving = true;
                return Ok(true);
            }
        }

        // If we cannot determine if the device is moving, assume it is.
        self.moving = true;
        Ok(true)
    }

    fn is_homing(&mut self) -> bool {
        todo!()
    }

    fn move_to(&mut self) {
        todo!()
    }

    fn move_relative(&mut self) {
        todo!()
    }

    fn short_name(&mut self) -> String {
        todo!()
    }

    fn long_name(&mut self) -> String {
        todo!()
    }
}
