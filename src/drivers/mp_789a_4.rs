use std::thread::sleep;
use std::time::Duration;

use super::MotionControlDriver;
use crate::drivers::serial::Serial;

const WR_DLY: u64 = 50; // milliseconds

pub struct Mp789a4 {
    comms: Serial,
    position: i64,
    long_name: String,
    short_name: String,
    moving: bool,
    homing: bool,
}

impl Mp789a4 {
    pub fn new(port_name: String) -> Result<Mp789a4, serialport::Error> {
        log::info!("Attempting to connect to MP789A4 at port {}.", port_name);

        // Initialize port communications.
        let mut comms = Serial::new(port_name.clone(), WR_DLY)?;

        // Request identification.
        comms.write_read(b" \r")?;

        if comms.recv_contains(b" v2.55\r\n#\r\n") {
            log::info!("Uninitialized MP789A4 detected.");
        } else if comms.recv_contains(b" #\r\n") {
            log::info!("Initialized MP789A4 detected.");
        } else {
            return Err(serialport::Error::new(
                serialport::ErrorKind::InvalidInput,
                "Invalid response from device.",
            ));
        }

        let mut dev = Mp789a4 {
            comms,
            position: 0,
            long_name: "McPherson 789A-4".to_string(),
            short_name: "MP789A4".to_string(),
            moving: false,
            homing: false,
        };

        dev.home()?;

        Ok(dev)
    }

    fn move_relative(&mut self, steps: i64) -> Result<(), serialport::Error> {
        match steps.cmp(&0) {
            std::cmp::Ordering::Less => {
                self.comms.write(format!("-{}\r", -steps).as_bytes())?;
            }
            std::cmp::Ordering::Equal => {
                log::warn!("No movement requested.");
            }
            std::cmp::Ordering::Greater => {
                self.comms.write(format!("+{}\r", steps).as_bytes())?;
            }
        }

        while self.is_moving()? {
            log::debug!("Blocking until movement completes.");
            sleep(Duration::from_millis(500));
        }

        self.position += steps;

        log::debug!("Movement complete.");

        Ok(())
    }

    fn _home(&mut self) -> Result<(), serialport::Error> {
        log::info!("Homing MP789A4.");

        // Enable the 789A-4's homing circuit.
        self.comms.write_read(b"A1\r")?;

        // Check limit switch status.
        self.comms.write_read(b"]\r")?;

        // Carries out the 789A-4 homing algorithm as described in the manual.
        if self.comms.recv_contains(b"32")
            && (!self.comms.recv_contains(b"+") && !self.comms.recv_contains(b"-"))
        {
            log::info!("Homing switch blocked.");

            // Home switch blocked.
            // Move at constant velocity (23 kHz).
            self.comms.write(b"M+23000\r")?;

            loop {
                // Check limit status every 0.8 seconds.
                self.comms.write_read(b"]\r")?;

                if (self.comms.recv_contains(b"0") || self.comms.recv_contains(b"2"))
                    && (!self.comms.recv_contains(b"+") && !self.comms.recv_contains(b"-"))
                {
                    // Not-on-a-limit-switch status is 0 when stationary, 2 when in motion.
                    break;
                } else if (self.comms.recv_contains(b"64") || self.comms.recv_contains(b"128"))
                    && (!self.comms.recv_contains(b"+") && !self.comms.recv_contains(b"-"))
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
            self.comms.write(b"@\r")?;

            // Back into home switch 3 motor revolutions.
            self.comms.write(b"-108000\r")?;
            // Go 2 motor revolutions up.
            self.comms.write(b"+72000\r")?;
            // Enable 'high accuracy' circuit.
            self.comms.write(b"A24\r")?;

            // Find edge of home flag at 1000 steps/sec.
            self.comms.write_sleep(b"F1000,0\r", WR_DLY * 7)?;

            // Disable home circuit.
            self.comms.write(b"A0\r")?;
        } else if self.comms.recv_contains(b"0")
            && (!self.comms.recv_contains(b"+") && !self.comms.recv_contains(b"-"))
        {
            // Home switch not blocked.
            // Move at constant velocity (23 kHz).
            self.comms.write(b"M-23000\r")?;

            loop {
                // Check limit status every 0.8 seconds.
                self.comms.write_read(b"]\r")?;

                if (self.comms.recv_contains(b"32") || self.comms.recv_contains(b"34"))
                    && (!self.comms.recv_contains(b"+") && !self.comms.recv_contains(b"-"))
                {
                    // Home-switch-blocked status is 32 when stationary, 34 when in motion.
                    break;
                } else if (self.comms.recv_contains(b"64") || self.comms.recv_contains(b"128"))
                    && (!self.comms.recv_contains(b"+") && !self.comms.recv_contains(b"-"))
                {
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
            self.comms.write(b"@\r")?;

            // Back into home switch 3 motor revolutions.
            self.comms.write(b"-108000\r")?;
            // Go 2 motor revolutions up.
            self.comms.write(b"+72000\r")?;
            // Enable 'high accuracy' circuit.
            self.comms.write(b"A24\r")?;

            // Find edge of home flag at 1000 steps/sec.
            self.comms.write_sleep(b"F1000,0\r", WR_DLY * 7)?;

            // Disable home circuit.
            self.comms.write(b"A0\r")?;
        } else {
            log::error!("Unknown position to home from: {:?}", self.comms.get_recv());
            return Err(serialport::Error::new(
                serialport::ErrorKind::InvalidInput,
                "Unknown position to home from",
            ));
        }

        // The standard is for the device drivers to read 0 when homed if the controller does not itself provide a value.
        // It is up to the middleware to handle zero- and home-offsets.
        if self.is_moving()? {
            log::warn!("Post-home movement detected. Entering movement remediation.");
            self.comms.write_sleep(b"@\r", WR_DLY * 10)?;
        }

        let mut stop_attempts = 0;
        while self.is_moving()? {
            if stop_attempts > 3 {
                stop_attempts = 1;
                log::warn!("Re-commanding that device ceases movement.");
                self.comms.write(b"@\r")?;
            }
            stop_attempts += 1;
            log::warn!("Waiting for device to cease movement.");
            sleep(Duration::from_millis(500));
        }

        self.position = 0;

        Ok(())
    }

    fn _move_to(
        &mut self,
        position: i64,
        backlash_correction: i64,
    ) -> Result<(), serialport::Error> {
        let steps = position - self.position;

        if steps < 0 && backlash_correction > 0 {
            // Move to the backlash position.
            self.move_relative(steps - backlash_correction)?;
            // Move to the final position.
            self.move_relative(backlash_correction)?;
        } else {
            self.move_relative(steps)?;
        }

        Ok(())
    }
}

impl MotionControlDriver for Mp789a4 {
    fn home(&mut self) -> Result<(), serialport::Error> {
        self.homing = true;
        match self._home() {
            Ok(_) => {
                self.homing = false;
                log::info!("Homing complete.");
                Ok(())
            }
            Err(e) => {
                self.homing = false;
                log::error!("Homing failed: {:?}", e);
                Err(e)
            }
        }
    }

    fn get_position(&mut self) -> i64 {
        self.position
    }

    fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Stopping {}.", self.short_name());
        self.comms.write(b"@\r")?;
        self.comms.write(b"@\r")?;
        self.comms.write(b"@\r")?;

        Ok(())
    }

    fn is_moving(&mut self) -> Result<bool, serialport::Error> {
        // If we cannot acquire the hardware lock, then assume we are moving (could also be stopping, but we arent stopped yet).

        match self.moving {
            true => return Ok(true),
            false => {}
        }

        // Finally, ask the device if its moving.
        self.comms.write_read(b"^\r")?;
        if self.comms.recv_contains(b"0")
            && !self.comms.recv_contains(b"+")
            && !self.comms.recv_contains(b"-")
        {
            self.moving = false;
            Ok(false)
        } else {
            // If we cannot determine if the device is moving, assume it is.
            // self.moving = true;
            Ok(true)
        }
    }

    fn is_homing(&mut self) -> bool {
        self.homing
    }

    fn move_to(
        &mut self,
        position: i64,
        backlash_correction: i64,
    ) -> Result<(), serialport::Error> {
        self.moving = true;
        match self._move_to(position, backlash_correction) {
            Ok(_) => {
                self.moving = false;
                log::info!("Movement complete.");
                Ok(())
            }
            Err(e) => {
                self.moving = false;
                // self.stop()?;
                log::error!("Movement failed: {:?}", e);
                Err(e)
            }
        }
    }

    fn short_name(&mut self) -> String {
        self.short_name.clone()
    }

    fn long_name(&mut self) -> String {
        self.long_name.clone()
    }
}

pub struct Mp789a4Virtual {
    position: i64,
    long_name: String,
    short_name: String,
}

impl Mp789a4Virtual {
    pub fn new(port_name: String) -> Result<Mp789a4Virtual, serialport::Error> {
        let mut dev = Mp789a4Virtual {
            position: 0,
            long_name: "McPherson 789A-4".to_string(),
            short_name: "MP789A4".to_string(),
        };

        dev.home()?;

        Ok(dev)
    }

    fn move_relative(&mut self, steps: i64) -> Result<(), serialport::Error> {
        self.position += steps;

        Ok(())
    }
}

impl MotionControlDriver for Mp789a4Virtual {
    fn home(&mut self) -> Result<(), serialport::Error> {
        Ok(())
    }

    fn get_position(&mut self) -> i64 {
        self.position
    }

    fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Stopping {}.", self.short_name());

        Ok(())
    }

    fn is_moving(&mut self) -> Result<bool, serialport::Error> {
        Ok(true)
    }

    fn is_homing(&mut self) -> bool {
        false
    }

    fn move_to(
        &mut self,
        position: i64,
        backlash_correction: i64,
    ) -> Result<(), serialport::Error> {
        self.position = position;
        Ok(())
    }

    fn short_name(&mut self) -> String {
        self.short_name.clone()
    }

    fn long_name(&mut self) -> String {
        self.long_name.clone()
    }
}
