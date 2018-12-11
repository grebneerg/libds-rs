use std::io;
use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, TcpStream, UdpSocket};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use byteorder::{ByteOrder, NetworkEndian};

use ds::DriverStationState;
use messages::{ds::tcp::*, rio::*};

pub struct DSConnection {
    thread: JoinHandle<()>,
    sender: mpsc::Sender<Signal>,
    errors: mpsc::Receiver<io::Result<()>>,
    err: Option<io::Error>,
}

impl DSConnection {
    pub fn new(addr: IpAddr, state: Arc<Mutex<DriverStationState>>) -> io::Result<Self> {
        let (sender_signal, receiver_signal) = mpsc::channel::<Signal>();

        let (sender_res, receiver_res) = mpsc::channel::<io::Result<()>>();

        // TODO: Create sockets within other thread and add mechanism to detect status of connections
        // (connecting, failed, connected, error, etc)

        let mut last = Instant::now();

        let t = thread::spawn(move || {
            println!("udp start");
            let udp = UdpSocket::bind("169.254.65.205:1149").unwrap();

            udp.connect(SocketAddr::new(addr.clone(), 1110)).unwrap();
            println!("udp 2");
            let udp_recv = UdpSocket::bind("169.254.65.205:1150").unwrap();
            udp_recv.set_nonblocking(true).unwrap();
            println!("udp started");

            udp.send(state.lock().unwrap().udp_packet().as_ref())
                .unwrap();

            println!("tcp start");
            let mut tcp = TcpStream::connect(SocketAddr::new(addr.clone(), 1740)).unwrap();
            tcp.set_nonblocking(true).unwrap();

            println!(
                "{:?}",
                GameData::new(state.lock().unwrap().game_data.clone()).to_packet()
            );
            tcp.write(
                GameData::new(state.lock().unwrap().game_data.clone())
                    .to_packet()
                    .as_slice(),
            )
            .unwrap();
			tcp.write(state.lock().unwrap().match_info.clone().to_packet().as_slice()).unwrap();

            loop {
                match receiver_signal.try_recv() {
                    Ok(Signal::Disconnect) | Err(mpsc::TryRecvError::Disconnected) => break,
                    Ok(Signal::Tcp(tag)) => {
                        match tcp.write(tag.to_packet().as_slice()) {
                            Ok(n) => println!("wrote"),
                            Err(e) => {} //TODO
                        }
                    }
                    _ => {}
                }

                let mut udp_buf = vec![0u8; 100];
                match udp_recv.recv_from(&mut udp_buf) {
                    Ok((n, f)) => {
                        if let Some(packet) = RioUdpPacket::from_bytes(Vec::from(&udp_buf[0..n])) {
                            state.lock().unwrap().update_from_udp(packet);
                        }
                    }
                    Err(e) => {
                        if e.kind() != io::ErrorKind::WouldBlock {
                            if let Err(e) = sender_res.send(Err(e)) {
                                break;
                            }
                        }
                    }
                }

                let mut size_buf = vec![0u8; 2];
                match tcp.read_exact(&mut size_buf) {
                    Ok(_) => {
                        let size = NetworkEndian::read_u16(&size_buf);
                        println!("tcp size: {}", size);
                        let mut buf = vec![0u8; size as usize];
                        match tcp.read_exact(&mut buf) {
                            Ok(_) => {
                                if let Some(packet) = RioTcpPacket::from_bytes(buf) {
                                    state.lock().unwrap().update_from_tcp(packet);
                                }
                            }
                            Err(e) => {
                                if e.kind() != io::ErrorKind::WouldBlock {
                                    if let Err(e) = sender_res.send(Err(e)) {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        if e.kind() != io::ErrorKind::WouldBlock {
                            if let Err(e) = sender_res.send(Err(e)) {
                                break;
                            }
                        }
                    }
                }

                if last.elapsed() >= Duration::from_millis(20) {
                    last = Instant::now();
					let packet = state.lock().unwrap().udp_packet();
                    match udp.send(packet.as_ref()) {
                        Ok(s) => println!("udp sent {:?}", packet),
                        Err(e) => {
                            if e.kind() != io::ErrorKind::WouldBlock {
                                if let Err(e) = sender_res.send(Err(e)) {
                                    break;
                                }
                            }
                        }
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

    pub fn send_tcp(&self, tag: TcpTag) {
        self.sender.send(Signal::Tcp(tag)).unwrap();
    }
}

impl Drop for DSConnection {
    fn drop(&mut self) {
        self.sender.send(Signal::Disconnect).unwrap_or(());
    }
}

pub enum Signal {
    Tcp(TcpTag),
    Disconnect,
}
