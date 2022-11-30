#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]

mod packet;
mod ping;
mod util;

use packet::Packet;
use ping::analyze;

use std::{
    net::{Ipv4Addr, SocketAddrV4, UdpSocket},
    time::Duration,
};

use crate::packet::PKT_SIZE;

const HOST_ADDR: Ipv4Addr = Ipv4Addr::LOCALHOST;
const HOST_PORT: u16 = 30001;
// const CLIENT_ADDR: Ipv4Addr = Ipv4Addr::LOCALHOST;
const CLIENT_ADDR: Ipv4Addr = Ipv4Addr::new(168, 227, 188, 22);
const CLIENT_PORT: u16 = 30000;

const TIME_BETEEN_PINGS: Duration = Duration::from_millis(500);
const EXTRA_TIME_TO_WAIT: Duration = Duration::from_secs(2);
const MAX_PINGS: u8 = 10;

fn main() {
    let host_url: SocketAddrV4 = SocketAddrV4::new(HOST_ADDR, HOST_PORT);
    let client_url: SocketAddrV4 = SocketAddrV4::new(CLIENT_ADDR, CLIENT_PORT);
    let socket = UdpSocket::bind(host_url).expect("Error binding socket");
    // socket.connect(client_url).expect("Error connecting socket");

    // socket
    //     .set_read_timeout(Some(TIME_UNTIL_DROPPED))
    //     .expect("Failed to set timeout");

    // Create the packets

    #[cfg(debug_assertions)]
    println!("Debug mode ON. Compile or run with --release to disable debug mode.");

    let mut pkts = (0..MAX_PINGS)
        .map(|_| Packet::default())
        .collect::<Vec<Packet>>();

    // print response and stats
    println!("PING {} ({}): {} data bytes", HOST_ADDR, host_url, PKT_SIZE);
    analyze(&mut pkts, &socket, client_url).expect("Error analyzing packets");
}
