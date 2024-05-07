use super::drivers;

// Holds an index corresponding to each axis of movement.
// The index is set by the user when they assign a device to an axis using a combobox.
// This allows the GUI to then access the arbitrarily ordered list of MotionControllers using these indices.
pub struct MovementAxesIndices {
    pub md_idx: Option<usize>, // main drive
    // filter_wheel: Option<MotionController>,
    pub sr_idx: Option<usize>, // sample rotation
    pub sa_idx: Option<usize>, // sample angle
    pub st_idx: Option<usize>, // sample translation
    pub dr_idx: Option<usize>, // detector rotation
}

impl MovementAxesIndices {
    pub fn new() -> MovementAxesIndices {
        MovementAxesIndices {
            md_idx: None,
            sr_idx: None,
            sa_idx: None,
            st_idx: None,
            dr_idx: None,
        }
    }
}

pub struct MotionController {
    pub id: usize,
}

impl MotionController {
    pub fn new(id: usize) -> MotionController {
        MotionController {
            id,
        }
    }

    // Each MotionController func will either:
    // - just call the driver function
    // - convert units + above
    // - perform checks + above

    // fn all_stop(&self);
    // fn set_limits(&self);
    // fn set_offset(&self);
    // fn get_offset(&self);
    // fn get_steps_per_value(&self);
    // fn is_dummy(&self);
    // fn home(&self);
    // fn _home(&self);
    // fn get_position(&self);
    // fn is_homing(&self);
    // fn is_moving(&self);
    // fn move_to(&self);
    // fn _move_to(&self);
    // fn stop(&self);
    // fn port_name(&self);
    // fn short_name(&self);
    // fn long_name(&self);
}

pub struct Detector {

}

impl Detector {
    pub fn new(id: usize) -> Detector {
        Detector {
        }
    }

    // fn detect(&self);
    // fn is_dummy(&self);
    // fn short_name(&self);
    // fn long_name(&self);
}