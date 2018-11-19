extern crate byteorder;
extern crate chrono;

use byteorder::{ByteOrder, NetworkEndian};
use chrono::prelude::*;

use std::default::Default;
use std::io;
use std::io::Read;
use std::net::{IpAddr, SocketAddr, TcpStream, UdpSocket};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

mod joystick;
mod messages;
mod packet;
mod states;

use joystick::Joystick;
use messages::*;
use packet::PacketWriter;
use states::{Alliance, RobotMode};

const TIMEZONE: &'static str = "UTC";

struct DSConnection {
    thread: JoinHandle<()>,
    sender: mpsc::Sender<Signal>,
    errors: mpsc::Receiver<io::Result<()>>,
    err: Option<io::Error>,
}

impl DSConnection {
    fn release(self) {
        drop(self);
    }

    fn new(addr: IpAddr, state: Arc<Mutex<DriverStationState>>) -> io::Result<Self> {
        let (sender_signal, receiver_signal) = mpsc::channel::<Signal>();

        let (sender_res, receiver_res) = mpsc::channel::<io::Result<()>>();

        let mut tcp = TcpStream::connect(SocketAddr::new(addr, 1740))?;
        tcp.set_nonblocking(true)?;

        let udp = UdpSocket::bind(SocketAddr::new(addr, 1150))?;
        udp.set_nonblocking(true)?;

        let t = thread::spawn(move || loop {
            match receiver_signal.try_recv() {
                Ok(Signal::Disconnect) | Err(mpsc::TryRecvError::Disconnected) => break,
                _ => {}
            }

            let mut udp_buf = vec![0u8; 100];
            match udp.recv(&mut udp_buf) {
                Ok(n) => unimplemented!(),
                Err(e) => {
                    if e.kind() != io::ErrorKind::WouldBlock {
                        sender_res.send(Err(e)).unwrap();
                    }
                }
            }

            let mut size_buf = vec![0u8; 2];
            match tcp.read_exact(&mut size_buf) {
                Ok(_) => {
                    let size = NetworkEndian::read_u16(&size_buf);
                    let mut buf = vec![0u8; size as usize];
                    match tcp.read_exact(&mut buf) {
                        Ok(_) => if let Some(packet) = RioTcpPacket::from_bytes(buf) {
                            state.lock().unwrap().update_from_tcp(packet);
                        },
                        Err(e) => {
                            if e.kind() != io::ErrorKind::WouldBlock {
                                sender_res.send(Err(e)).unwrap();
                            }
                        }
                    }
                }
                Err(e) => {
                    if e.kind() != io::ErrorKind::WouldBlock {
                        sender_res.send(Err(e)).unwrap();
                    }
                }
            }
        });

        Ok(DSConnection {
            thread: t,
            sender: sender_signal,
            errors: receiver_res,
            err: None,
        })
    }

    fn status(&self) -> io::Result<()> {
        match self.errors.try_recv() {
            Err(mpsc::TryRecvError::Empty) => Ok(()),
            Err(mpsc::TryRecvError::Disconnected) => Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "thread not connected",
            )),
            Ok(ior) => ior,
        }
    }
}

impl Drop for DSConnection {
    fn drop(&mut self) {
        self.sender.send(Signal::Disconnect).unwrap_or(());
    }
}

pub struct DriverStation {
    state: Arc<Mutex<DriverStationState>>,
    connection: Option<DSConnection>,
}

impl DriverStation {
    fn connect(&mut self, addr: IpAddr) -> io::Result<()> {
        if let Some(conn) = self.connection.take() {
            conn.release();
        }
        self.connection = Some(DSConnection::new(addr, self.state.clone())?);
        Ok(())
    }

    fn is_connected(&self) -> bool {
        match self.connection {
            None => false,
            Some(ref conn) => match conn.status() {
                Ok(_) => true,
                Err(_) => false,
            },
        }
    }
}

pub struct DriverStationState {
    joysticks: Vec<Option<Joystick>>,
    estop: bool,
    enabled: bool,
    mode: RobotMode,
    alliance: Alliance,
    game_data: String,
    competition: String,
    sequence_num: u16,
    request_time: bool,
}

impl DriverStationState {
    pub fn new() -> Self {
        Default::default()
    }

    fn udp_packet(&mut self) -> Vec<u8> {
        let mut packet = PacketWriter::new();

        // Packet number in case they arrive out of order
        packet.write_u16(self.sequence_num);
        self.sequence_num += 1;

        packet.write_u8(0x01); // comm version
        packet.write_u8(self.control_byte()); // control byte
        packet.write_u8(0); // TODO: actually restart code or rio with this byte.
        packet.write_u8(self.alliance.to_position_u8()); // alliance

        // joystick tags
        for stick in &self.joysticks {
            if let Some(stick) = stick {
                let mut tag = stick.udp_tag();
                packet.write_u8(tag.len() as u8 + 1); // size
                packet.write_u8(0x0c); // id
                packet.write_vec(tag); // joystick tag info
            } else {
                // Empty joystick tag
                packet.write_u8(0x01); // size
                packet.write_u8(0x0c); // id
            }
        }

        // datetime and timezone
        if self.request_time {
            // timezone
            packet.write_u8(TIMEZONE.len() as u8 + 1); // size
            packet.write_u8(0x10); // id
            packet.write_slice(TIMEZONE.as_bytes());

            // date and time
            packet.write_vec(date_packet());
        }

        packet.into_vec()
    }

    fn control_byte(&self) -> u8 {
        let mut byte: u8 = 0;
        if self.estop {
            byte |= 0b1000_0000;
        }
        // fms is never connected, but if it were that would go here
        if self.enabled {
            byte |= 0b0000_0100;
        }

        byte |= self.mode as u8;

        byte
    }

    fn update_from_tcp(&mut self, packet: RioTcpPacket) {
        unimplemented!();
    }
}

fn date_packet() -> Vec<u8> {
    let mut packet = PacketWriter::new();
    let now = Utc::now();
    packet.write_u8(11); // size
    packet.write_u8(0x0f); // id
    let nanos = now.nanosecond();
    let micros = nanos / 1000;
    packet.write_u32(micros);
    packet.write_u8(now.second() as u8);
    packet.write_u8(now.minute() as u8);
    packet.write_u8(now.hour() as u8);
    packet.write_u8(now.day() as u8); // should this be day0?
    packet.write_u8(now.month0() as u8);
    packet.write_u8((now.year() - 1900) as u8);
    packet.into_vec()
}

impl Default for DriverStationState {
    fn default() -> Self {
        DriverStationState {
            joysticks: vec![None; 6],
            estop: false,
            enabled: false,
            mode: RobotMode::Teleop,
            alliance: Alliance::Red(1),
            game_data: String::new(),
            competition: String::from("unknown"),
            sequence_num: 0,
            request_time: false,
        }
    }
}

enum Signal {
    Udp,
    Tcp,
    Disconnect,
}
