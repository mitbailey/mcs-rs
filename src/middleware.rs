use crate::drivers;

// Holds an index corresponding to each axis of movement.
// The index is set by the user when they assign a device to an axis using a combobox.
// This allows the GUI to then access the arbitrarily ordered list of MotionControllers using these indices.
pub struct MovementAxesIndices {
    pub md_idx: Option<usize>, // main drive
    pub fw_idx: Option<MotionController>, // filter wheel
    pub sr_idx: Option<usize>, // sample rotation
    pub sa_idx: Option<usize>, // sample angle
    pub st_idx: Option<usize>, // sample translation
    pub dr_idx: Option<usize>, // detector rotation
}

impl Default for MovementAxesIndices {
    fn default() -> MovementAxesIndices {
        MovementAxesIndices {
            md_idx: None,
            fw_idx: None,
            sr_idx: None,
            sa_idx: None,
            st_idx: None,
            dr_idx: None,
        }
    }
}

pub trait MotionControlMiddleware {
    fn all_stop(&self);
    fn set_limits(&self);
    fn set_offset(&self);
    fn get_offset(&self);
    fn set_steps_per_value(&self);
    fn get_steps_per_value(&self);
    // fn is_dummy(&self);
    fn home(&self);
    fn get_position(&self);
    fn is_homing(&self);
    fn is_moving(&self);
    fn move_to(&self);
    fn stop(&self);
    fn port_name(&self);
    fn short_name(&self);
    fn long_name(&self);
}

pub struct MotionController {
    pub driver: Box<dyn drivers::MotionControlDriver>,
}

impl MotionController {
    pub fn new(driver: Box<dyn drivers::MotionControlDriver>) -> MotionController {
        MotionController {
            driver,
        }
    }
}

impl MotionControlMiddleware for MotionController {
    fn all_stop(&self) {
        todo!()
    }

    fn set_limits(&self) {
        todo!()
    }

    fn set_offset(&self) {
        todo!()
    }

    fn get_offset(&self) {
        todo!()
    }

    fn set_steps_per_value(&self) {
        todo!()
    }

    fn get_steps_per_value(&self) {
        todo!()
    }

    fn home(&self) {
        todo!()
    }

    fn get_position(&self) {
        todo!()
    }

    fn is_homing(&self) {
        todo!()
    }

    fn is_moving(&self) {
        todo!()
    }

    fn move_to(&self) {
        todo!()
    }

    fn stop(&self) {
        todo!()
    }

    fn port_name(&self) {
        todo!()
    }

    fn short_name(&self) {
        todo!()
    }

    fn long_name(&self) {
        todo!()
    }
}

pub trait DetectorMiddleware {
    fn new_scan(&mut self);
    fn get_last_scan(&self) -> Vec<f64>;
    fn detect(&mut self) -> f64;
    fn short_name(&mut self);
    fn long_name(&mut self);
}

pub struct Detector {
    pub driver: Box<dyn drivers::DetectorDriver>,
    
    scans: Vec<Vec<f64>>,
}

impl Detector {
    pub fn new(driver: Box<dyn drivers::DetectorDriver>) -> Detector {
        Detector {
            driver,
            scans: Vec::new(),
        }
    }
}

impl DetectorMiddleware for Detector {
    /// Detector data is always appended to the latest vector in scans.
    /// This function creates a new empty scan vector for following data.
    fn new_scan(&mut self) {
        self.scans.push(Vec::new());
    }

    fn get_last_scan(&self) -> Vec<f64> {
        self.scans.last().unwrap().to_owned()
    }

    fn detect(&mut self) -> f64 {
        let data = self.driver.detect();

        // Put the data into the last vector in the vector of vectors.
        let retval = data.clone().unwrap();
        self.scans.last_mut().unwrap().push(data.unwrap());
        retval
    }

    fn short_name(&mut self) {
        todo!()
    }

    fn long_name(&mut self) {
        todo!()
    }
}