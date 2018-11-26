extern crate libds;

use std::thread;
use std::time;

use libds::DriverStation;

fn main() {
    let mut ds = DriverStation::new();
    ds.connect([169, 254, 204, 207].into()).unwrap();
    println!("we connected");
    thread::sleep(time::Duration::from_millis(2000));
    ds.state.lock().unwrap().enabled = true;
    println!("we enabled");
    thread::sleep(time::Duration::from_millis(2000));
    ds.state.lock().unwrap().enabled = false;
    println!("we disabled");
    thread::sleep(time::Duration::from_millis(2000));
    println!("we done");
}
