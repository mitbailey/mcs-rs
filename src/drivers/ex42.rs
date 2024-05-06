struct Ex42 {
    x: i32,
}

impl MotionControllerDriver for Ex42 {
    fn home(&self) {
        println!("home");
    }

    fn _home(&self) {
        println!("_home");
    }

    fn get_position(&self) {
        println!("get_position");
    }

    fn stop(&self) {
        println!("all_stop");
    }

    fn is_moving(&self) {
        println!("is_moving");
    }

    fn is_homing(&self) {
        println!("is_homing");
    }

    fn move_to(&self) {
        println!("move_to");
    }

    fn _move_to(&self) {
        println!("_move_to");
    }

    fn move_relative(&self) {
        println!("move_relative");
    }

    fn short_name(&self) {
        println!("short_name");
    }

    fn long_name(&self) {
        println!("long_name");
    }
}

impl Ex42 {
    pub fn new() -> Ex42 {
        Ex42 {
            x: 0,
        }
    }

    // Other functions unique to this, such as the 792's multi-axis selection function "set_axis".
}