use std::{
    net::{self, IpAddr},
    sync::mpsc::{self, TryRecvError},
    thread,
};

use artnet_protocol::{ArtCommand, Output, Poll};

use super::DmxData;

const ARTNET_PORT: u16 = 6454;

pub fn start_artnet_output_thread(
    rx: mpsc::Receiver<DmxData>,
    destination_ip: Option<String>,
    bind_ip: Option<String>,
) {
    thread::spawn(move || {
        let socket =
            net::UdpSocket::bind((bind_ip.unwrap_or("0.0.0.0".to_owned()), ARTNET_PORT)).unwrap();

        let destination_addr = net::SocketAddr::new(
            net::IpAddr::V4(
                destination_ip
                    .map(|ip| ip.parse().unwrap())
                    .unwrap_or(net::Ipv4Addr::BROADCAST),
            ),
            ARTNET_PORT,
        );
        socket
            .set_broadcast(if let IpAddr::V4(addr) = destination_addr.ip() {
                addr.is_broadcast()
            } else {
                false
            })
            .unwrap();

        // Send poll
        let poll_buff = ArtCommand::Poll(Poll::default()).write_to_buffer().unwrap();
        socket.send_to(&poll_buff, destination_addr).unwrap();

        loop {
            let recv_result = rx.try_recv();

            if let Ok((send_universe, send_universe_data)) = recv_result {
                let output_command = ArtCommand::Output(Output {
                    data: Vec::from(send_universe_data).into(),
                    port_address: send_universe.try_into().unwrap(),
                    ..Output::default()
                });

                let command_bytes = output_command.write_to_buffer().unwrap();
                socket.send_to(&command_bytes, destination_addr).unwrap();
            } else if recv_result.err().unwrap() == TryRecvError::Disconnected {
                break;
            }
        }
    });
}
