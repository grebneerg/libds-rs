extern crate libds;

use std::thread;
use std::time;

use libds::{
    states::{Alliance, RobotMode},
    DriverStation,
};

fn main() {
    let mut ds = DriverStation::new();
    ds.connect([169, 254, 204, 207].into()).unwrap();

    ds.set_mode(RobotMode::Auto);
    ds.set_alliance(Alliance::Blue(2));
	ds.set_game_data("rrr".to_string());

    println!("we connected");
    thread::sleep(time::Duration::from_millis(2000));
    ds.set_enabled(true);
    println!("we enabled");
    thread::sleep(time::Duration::from_millis(2000));
    ds.set_enabled(false);
    println!("we disabled");
    thread::sleep(time::Duration::from_millis(2000));
    println!("we done");
}
