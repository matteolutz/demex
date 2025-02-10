use std::{
    net,
    sync::mpsc::{self, TryRecvError},
    thread,
};

use artnet_protocol::{ArtCommand, Output};

use super::DmxData;

const ARTNET_PORT: u16 = 6454;

pub fn start_artnet_output_thread(rx: mpsc::Receiver<DmxData>, socket_addr: String) {
    thread::spawn(move || {
        let socket = net::UdpSocket::bind((socket_addr, ARTNET_PORT)).unwrap();
        let broadcast_addr =
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
                socket.send_to(&command_bytes, broadcast_addr).unwrap();
            } else if recv_result.err().unwrap() == TryRecvError::Disconnected {
                break;
            }
        }
    });
}
