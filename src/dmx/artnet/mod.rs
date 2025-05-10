use std::{
    collections::HashMap,
    net::{self, SocketAddr},
    sync::mpsc::{self, TryRecvError},
    thread, time,
};

use artnet_protocol::{ArtCommand, Output, Poll, PortAddress};
use egui_probe::EguiProbe;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use super::DmxData;

#[derive(Clone, Default, Debug, Serialize, Deserialize, EguiProbe)]
pub struct ArtnetOutputConfig {
    pub broadcast: bool,

    #[serde(default)]
    pub broadcast_addresses: Vec<String>,

    pub bind_ip: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct ArtNetOutputNode {
    pub addr: SocketAddr,
}

const ARTNET_PORT: u16 = 6454;

pub fn start_broadcast_artnet_output_thread(
    rx: mpsc::Receiver<DmxData>,
    config: ArtnetOutputConfig,
) {
    thread::spawn(move || {
        let socket = net::UdpSocket::bind((
            config.bind_ip.unwrap_or_else(|| "0.0.0.0".to_owned()),
            ARTNET_PORT,
        ))
        .unwrap();
        socket
            .set_read_timeout(Some(time::Duration::from_secs(3)))
            .unwrap();
        nix::sys::socket::setsockopt(&socket, nix::sys::socket::sockopt::ReuseAddr, &true).unwrap();

        let broadcast_addresses = config
            .broadcast_addresses
            .iter()
            .map(|addr| net::SocketAddr::new(net::IpAddr::V4(addr.parse().unwrap()), ARTNET_PORT))
            .collect::<Vec<_>>();

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
                for addr in &broadcast_addresses {
                    socket.send_to(&command_bytes, addr).unwrap();
                }
            } else if recv_result.err().unwrap() == TryRecvError::Disconnected {
                break;
            }
        }
    });
}

pub fn start_artnet_output_thread(rx: mpsc::Receiver<DmxData>, config: ArtnetOutputConfig) {
    thread::spawn(move || {
        log::debug!("Starting ArtNet thread..");

        let socket = net::UdpSocket::bind((
            config.bind_ip.unwrap_or_else(|| "0.0.0.0".to_owned()),
            ARTNET_PORT,
        ))
        .unwrap();
        socket
            .set_read_timeout(Some(time::Duration::from_secs(3)))
            .unwrap();
        nix::sys::socket::setsockopt(&socket, nix::sys::socket::sockopt::ReuseAddr, &true).unwrap();

        let broadcast_addresses = config
            .broadcast_addresses
            .iter()
            .map(|addr| net::SocketAddr::new(net::IpAddr::V4(addr.parse().unwrap()), ARTNET_PORT))
            .collect::<Vec<_>>();

        socket.set_broadcast(true).unwrap();
        socket.set_nonblocking(true).unwrap();

        let mut nodes: HashMap<PortAddress, Vec<ArtNetOutputNode>> = HashMap::new();

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

                if let Some(nodes) = nodes.get_mut(&send_universe) {
                    nodes.retain(|node| {
                        if let Ok(_) = socket.send_to(&command_bytes, node.addr) {
                            true
                        } else {
                            // If we fail to send, the node is probably disconnected and we should remove it
                            // from the list of nodes.
                            // If anything else hapended, it should added back to the list with the next poll.
                            log::debug!("Failed to send to node {:?}, removing it..", node);
                            false
                        }
                    });
                }
            } else if recv_result.err().unwrap() == TryRecvError::Disconnected {
                break;
            }

            // handle incoming artnet commands
            if last_socket_in_update.elapsed().as_millis() > 40 {
                // should we send a poll?
                if last_poll_sent.is_none() || last_poll_sent.unwrap().elapsed().as_secs_f64() > 3.0
                {
                    for addr in &broadcast_addresses {
                        log::debug!("Sending ArtNet Poll to broadcast address {}..", addr);
                        socket.send_to(&poll_buff, addr).unwrap();
                    }

                    last_poll_sent = Some(time::Instant::now());
                }

                // check for incoming data
                if let Ok((length, _)) = socket.recv_from(&mut input_buffer) {
                    let command = ArtCommand::from_buffer(&input_buffer[..length]).unwrap();

                    match command {
                        ArtCommand::PollReply(poll_reply) => {
                            let net_and_subnet: u16 = ((poll_reply.port_address[0] as u16) << 8)
                                | ((poll_reply.port_address[1] as u16) << 4);

                            for uni in poll_reply.swout.iter().unique() {
                                let universe: PortAddress =
                                    (net_and_subnet | *uni as u16).try_into().unwrap();

                                let node = ArtNetOutputNode {
                                    addr: (poll_reply.address, ARTNET_PORT).into(),
                                };

                                let universe_nodes = nodes.entry(universe).or_default();
                                if !universe_nodes.contains(&node) {
                                    universe_nodes.push(node);
                                }
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
