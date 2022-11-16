use std::{
    net::{SocketAddrV4, UdpSocket},
    thread,
    time::{Duration, Instant},
};

use crate::{
    packet::{Packet, PKT_SIZE},
    util::u8_to_py_hex_str,
    MAX_PINGS, TIME_BETEEN_PINGS, EXTRA_TIME_TO_WAIT,
};

pub fn fetch(
    pkt: Packet,
    socket: &UdpSocket,
    url: SocketAddrV4,
    time_until_drop: Duration,
) -> Option<(Duration, Vec<u8>)> {
    let ping_msg: Vec<u8> = pkt.into();
    let mut buf = [0; PKT_SIZE];

    println!(
        "\nSending packet {}:\t{}",
        u64::from_le_bytes([pkt.seq[0], pkt.seq[1], pkt.seq[2], pkt.seq[3], pkt.seq[4], 0, 0, 0]),
        u8_to_py_hex_str(&ping_msg)
    );
    println!("Chars: \"{}\"", String::from_utf8_lossy(&pkt.message));

    socket
        .set_read_timeout(Some(time_until_drop))
        .expect("Error setting read timeout");

    let start = Instant::now();
    socket.send_to(&ping_msg, url).expect("Error sending ping");
    let Ok((amt, _srcip)) = socket.recv_from(&mut buf) else {
        return None;
    };
    let elapsed = start.elapsed();

    let msg = buf[..amt].to_vec();

    Some((elapsed, msg))
}

pub fn analyze(pkts: &[Packet], socket: &UdpSocket, url: SocketAddrV4) {
    let mut min = Duration::MAX;
    let mut max = Duration::new(0, 0);
    let mut elapseds = vec![];

    // create a thread for each ping
    let mut handles = vec![];
    for (i, &pkt) in pkts.iter().enumerate() {
        let time_until_drop = (TIME_BETEEN_PINGS * (pkts.len() - i) as u32) + EXTRA_TIME_TO_WAIT;
        let socket = socket.try_clone().expect("Error cloning socket");
        let handle = std::thread::spawn(move || {
            let res = fetch(pkt, &socket, url, time_until_drop);

            if let Some((ref elapsed, ref msg)) = res {
                let pkt = Packet::from(msg.clone());
                // print response
                println!(
                    "\nReceived packet {}:\t\"{}\"",
                    u64::from_le_bytes([
                        pkt.seq[0], pkt.seq[1], pkt.seq[2], pkt.seq[3], pkt.seq[4], 0, 0, 0
                    ]),
                    u8_to_py_hex_str(msg)
                );
                println!("Chars:\t\"{}\"", String::from_utf8_lossy(msg));
                println!("Elapsed:{:?}", elapsed);
            } else {
                println!(
                    "\nPacket {} dropped",
                    u64::from_le_bytes([
                        pkt.seq[0], pkt.seq[1], pkt.seq[2], pkt.seq[3], pkt.seq[4], 0, 0, 0
                    ]),
                );
            };

            res
        });
        handles.push(handle);
        thread::sleep(TIME_BETEEN_PINGS);
    }
    for handle in handles {
        if let Some((elapsed, _)) = handle.join().expect("Error joining thread") {
            // update stats
            elapseds.push(elapsed);
            if elapsed < min {
                min = elapsed;
            }
            if elapsed > max {
                max = elapsed;
            }
        }
    }

    // calc stats
    let total = elapseds.iter().fold(Duration::new(0, 0), |a, &b| a + b);
    let avg = total / MAX_PINGS.into();
    let mdev = elapseds
        .iter()
        .map(|&d| {
            Duration::from_micros(
                d.as_micros()
                    .abs_diff(avg.as_micros())
                    .try_into()
                    .expect("Duration is longer than u64"),
            )
        })
        .fold(Duration::new(0, 0), |a, b| a + b)
        / MAX_PINGS.into();

    // print stats
    println!(
        "\n\n{} packets transmitted, {} received, {}% packet loss, time {:?}, min/avg/max/mdev = {:?}/{:?}/{:?}/{:?}",
        MAX_PINGS,
        elapseds.len(),
        100 - (elapseds.len() * 100 / MAX_PINGS as usize),
        total,
        min,
        avg,
        max,
        mdev
    );
}
