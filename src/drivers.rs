use serialport;

pub mod serial;
pub mod mp_789a_4;
pub mod ki_6485;

// TODO: Implement custom errors instead of shoe-horning serialport::Error in everywhere.
// So, we cannot use mutex<()> as some sort of auto-resetting boolean, because thats not how mutexes work and the borrow checkers get angry (rightfully so). Therefore, we need public functions such as "home" that simply set self.homing to true and then call the real, private, do_home() function. Why? Because otherwise if an error propagates, and we are setting the self.homing boolean within the function, it will not be unset (homing forever). This way, if theres an error, we can reset the boolean before propagating the error again.
pub trait MotionControlDriver {
    fn home(&mut self) -> Result<(), serialport::Error>;
    fn get_position(&mut self) -> i64;
    fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    fn is_moving(&mut self) -> Result<bool, serialport::Error>;
    fn is_homing(&mut self) -> bool;
    fn move_to(&mut self, position: i64, backlash_correction: i64) -> Result<(), serialport::Error>;
    fn short_name(&mut self) -> String;
    fn long_name(&mut self) -> String;
}

// move_relative is not included in the trait bc the user only ever wants to move to an absolute position, and some controllers have absolute position commands directly. Some do not - only those must implement a relative move function.

pub trait DetectorDriver {
    fn detect(&mut self) -> f64;
}