use std::{
    collections::HashMap,
    net::{self, SocketAddr},
    sync::{
        mpsc::{self, TryRecvError},
        Arc,
    },
    thread, time,
};

use artnet_protocol::{ArtCommand, Output, Poll, PortAddress};
use egui::ahash::HashSet;
use itertools::Itertools;
use parking_lot::RwLock;

use super::DmxData;

#[derive(Debug, PartialEq, Eq, Hash)]
struct ArtNetOutputNode {
    pub addr: SocketAddr,
}

const ARTNET_PORT: u16 = 6454;

pub fn start_broadcast_artnet_output_thread(rx: mpsc::Receiver<DmxData>, bind_ip: Option<String>) {
    thread::spawn(move || {
        let socket =
            net::UdpSocket::bind((bind_ip.unwrap_or("0.0.0.0".to_owned()), ARTNET_PORT)).unwrap();
        socket
            .set_read_timeout(Some(time::Duration::from_secs(3)))
            .unwrap();

        let broacast_addr =
            net::SocketAddr::new(net::IpAddr::V4(net::Ipv4Addr::BROADCAST), ARTNET_PORT);
        socket.set_broadcast(true).unwrap();

        loop {
            let recv_result = rx.try_recv();

            if let Ok((send_universe, send_universe_data)) = recv_result {
                let output_command = ArtCommand::Output(Output {
                    data: Vec::from(send_universe_data).into(),
                    port_address: send_universe.try_into().unwrap(),
                    ..Output::default()
                });

                let command_bytes = output_command.write_to_buffer().unwrap();
                socket.send_to(&command_bytes, broacast_addr).unwrap();
            } else if recv_result.err().unwrap() == TryRecvError::Disconnected {
                break;
            }
        }
    });
}

pub fn start_artnet_output_thread(rx: mpsc::Receiver<DmxData>, bind_ip: Option<String>) {
    thread::spawn(move || {
        let socket =
            net::UdpSocket::bind((bind_ip.unwrap_or("0.0.0.0".to_owned()), ARTNET_PORT)).unwrap();
        socket
            .set_read_timeout(Some(time::Duration::from_secs(3)))
            .unwrap();

        let broadcast_addr =
            net::SocketAddr::new(net::IpAddr::V4(net::Ipv4Addr::BROADCAST), ARTNET_PORT);
        socket.set_broadcast(true).unwrap();
        socket.set_nonblocking(true).unwrap();

        let socket_lock = Arc::new(RwLock::new(socket));
        let nodes_lock = Arc::new(RwLock::new(
            HashMap::<PortAddress, HashSet<ArtNetOutputNode>>::new(),
        ));

        let socket_lock_cloned = socket_lock.clone();
        let nodes_lock_cloned = nodes_lock.clone();
        thread::spawn(move || {
            let socket_lock = socket_lock_cloned;
            let nodes_lock = nodes_lock_cloned;

            let mut poll_sent: Option<time::Instant> = None;

            loop {
                if poll_sent.is_none() || poll_sent.unwrap().elapsed().as_secs_f64() >= 3.0 {
                    // Send poll
                    let poll_buff = ArtCommand::Poll(Poll::default()).write_to_buffer().unwrap();
                    socket_lock
                        .write()
                        .send_to(&poll_buff, broadcast_addr)
                        .unwrap();
                    poll_sent = Some(time::Instant::now());
                }

                let mut buffer = [0u8; 1024];

                if let Ok((length, _addr)) = socket_lock.read().recv_from(&mut buffer) {
                    let command = ArtCommand::from_buffer(&buffer[..length]).unwrap();
                    match command {
                        ArtCommand::PollReply(poll_reply) => {
                            let mut nodes = nodes_lock.write();

                            let net_and_subnet: u16 = (poll_reply.port_address[0] as u16) << 8
                                | (poll_reply.port_address[1] as u16) << 4;

                            for uni in poll_reply.swout.iter().unique() {
                                let universe: PortAddress =
                                    (net_and_subnet | *uni as u16).try_into().unwrap();

                                let node = ArtNetOutputNode {
                                    addr: (poll_reply.address, ARTNET_PORT).into(),
                                };

                                nodes.entry(universe).or_default().insert(node);
                            }
                        }
                        _ => {}
                    }
                }

                thread::sleep(time::Duration::from_millis(40));
            }
        });

        loop {
            let recv_result = rx.try_recv();

            if let Ok((send_universe, send_universe_data)) = recv_result {
                let send_universe = send_universe.try_into().unwrap();

                let output_command = ArtCommand::Output(Output {
                    data: Vec::from(send_universe_data).into(),
                    port_address: send_universe,
                    ..Output::default()
                });

                let command_bytes = output_command.write_to_buffer().unwrap();

                let socket = socket_lock.write();
                if let Some(nodes) = nodes_lock.read().get(&send_universe) {
                    for node in nodes {
                        socket.send_to(&command_bytes, node.addr).unwrap();
                    }
                }
            } else if recv_result.err().unwrap() == TryRecvError::Disconnected {
                break;
            }
        }
    });
}
