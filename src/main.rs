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

const HOST_ADDR: Ipv4Addr = Ipv4Addr::LOCALHOST;
const HOST_PORT: u16 = 30001;
const CLIENT_ADDR: Ipv4Addr = Ipv4Addr::LOCALHOST;
const CLIENT_PORT: u16 = 30000;

const TIME_BETEEN_PINGS: Duration = Duration::from_millis(500);
const EXTRA_TIME_TO_WAIT: Duration = Duration::from_secs(5);
const MAX_PINGS: u8 = 27;

fn main() {
    let host_url: SocketAddrV4 = SocketAddrV4::new(HOST_ADDR, HOST_PORT);
    let client_url: SocketAddrV4 = SocketAddrV4::new(CLIENT_ADDR, CLIENT_PORT);
    let socket = UdpSocket::bind(host_url).expect("Error binding socket");

    // socket
    //     .set_read_timeout(Some(TIME_UNTIL_DROPPED))
    //     .expect("Failed to set timeout");

    // Create the packets
    let pkts = (0..MAX_PINGS)
        .map(|_| Packet::default())
        .collect::<Vec<Packet>>();

    // print response and stats
    analyze(&pkts, &socket, client_url);
}
