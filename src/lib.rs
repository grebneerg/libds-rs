extern crate byteorder;
extern crate chrono;
#[macro_use]
extern crate bitflags;

use byteorder::{ByteOrder, NetworkEndian};
use chrono::prelude::*;

use std::default::Default;
use std::io;
use std::io::Read;
use std::net::{IpAddr, SocketAddr, TcpStream, UdpSocket};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

mod connection;
mod ds;
mod joystick;
mod messages;
mod packet;
mod states;

use connection::DSConnection;
use ds::DriverStationState;
use joystick::Joystick;
use messages::*;
use packet::PacketWriter;
use states::{Alliance, RobotMode};

pub struct DriverStation {
    pub state: Arc<Mutex<DriverStationState>>,
    connection: Option<DSConnection>,
}

impl DriverStation {
    pub fn new() -> Self {
        DriverStation {
            state: Arc::new(Mutex::new(Default::default())),
            connection: None,
        }
    }

    pub fn connect(&mut self, addr: IpAddr) -> io::Result<()> {
        if let Some(conn) = self.connection.take() {
            drop(conn);
        }
        self.connection = Some(DSConnection::new(addr, self.state.clone())?);
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        match self.connection {
            None => false,
            Some(ref conn) => match conn.status() {
                Ok(_) => true,
                Err(_) => false,
            },
        }
    }
}
