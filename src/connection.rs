use std::io;
use std::io::Read;
use std::net::{IpAddr, SocketAddr, TcpStream, UdpSocket};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

use byteorder::{ByteOrder, NetworkEndian};

use ds::DriverStationState;
use messages::*;

pub struct DSConnection {
    thread: JoinHandle<()>,
    sender: mpsc::Sender<Signal>,
    errors: mpsc::Receiver<io::Result<()>>,
    err: Option<io::Error>,
}

impl DSConnection {
    pub fn release(self) {
        drop(self);
    }

    pub fn new(addr: IpAddr, state: Arc<Mutex<DriverStationState>>) -> io::Result<Self> {
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

    pub fn status(&self) -> io::Result<()> {
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

pub enum Signal {
    Udp,
    Tcp,
    Disconnect,
}
