use std::{
    collections::HashMap,
    net::{self, SocketAddr},
    sync::mpsc::{self, TryRecvError},
    thread, time,
};

use artnet_protocol::{ArtCommand, Output, Poll, PortAddress};
use egui::ahash::HashSet;
use itertools::Itertools;

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

        let mut nodes: HashMap<PortAddress, HashSet<ArtNetOutputNode>> = HashMap::new();

        let poll_buff = ArtCommand::Poll(Poll::default()).write_to_buffer().unwrap();
        let mut last_poll_sent: Option<time::Instant> = None;
        let mut input_buffer = [0u8; 512];

        let mut last_socket_in_update = time::Instant::now();

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

                if let Some(nodes) = nodes.get(&send_universe) {
                    for node in nodes {
                        socket.send_to(&command_bytes, node.addr).unwrap();
                    }
                }
            } else if recv_result.err().unwrap() == TryRecvError::Disconnected {
                break;
            }

            // handle incoming artnet commands
            if last_socket_in_update.elapsed().as_millis() > 40 {
                // should we send a poll?
                if last_poll_sent.is_none() || last_poll_sent.unwrap().elapsed().as_secs_f64() > 3.0
                {
                    socket.send_to(&poll_buff, broadcast_addr).unwrap();
                    last_poll_sent = Some(time::Instant::now());
                }

                // check for incoming data
                if let Ok((length, _)) = socket.recv_from(&mut input_buffer) {
                    let command = ArtCommand::from_buffer(&input_buffer[..length]).unwrap();

                    match command {
                        ArtCommand::PollReply(poll_reply) => {
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

                last_socket_in_update = time::Instant::now();
            }
        }
    });
}
