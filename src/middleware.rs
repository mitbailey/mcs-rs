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
    pub id: usize,
    pub driver: Box<dyn drivers::MotionControlDriver>,
}

impl MotionController {
    pub fn new(id: usize, driver: Box<dyn drivers::MotionControlDriver>) -> MotionController {
        MotionController {
            id,
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
    fn detect(&mut self) -> f64;
    fn short_name(&mut self);
    fn long_name(&mut self);
}

pub struct Detector {
    pub id: usize,
    pub driver: Box<dyn drivers::DetectorDriver>,
}

impl Detector {
    pub fn new(id: usize, driver: Box<dyn drivers::DetectorDriver>) -> Detector {
        Detector {
            id,
            driver,
        }
    }
}

impl DetectorMiddleware for Detector {
    fn detect(&mut self) -> f64 {
        self.driver.detect()
    }

    fn short_name(&mut self) {
        todo!()
    }

    fn long_name(&mut self) {
        todo!()
    }
}