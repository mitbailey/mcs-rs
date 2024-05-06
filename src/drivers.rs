use serialport;

pub mod mp_789a_4;


pub trait MotionControllerDriver {
    fn home(&mut self) -> Result<(), serialport::Error>;
    // fn _home(&mut self);
    fn get_position(&mut self) -> i64;
    fn stop(&mut self) -> Result<(), serialport::Error>;
    fn is_moving(&mut self) -> Result<bool, serialport::Error>;
    fn is_homing(&mut self) -> bool;
    fn move_to(&mut self);
    // fn _move_to(&mut self);
    fn move_relative(&mut self);
    fn short_name(&mut self) -> String;
    fn long_name(&mut self) -> String;
}

pub trait DetectorDriver {
    fn detect(&mut self);
}