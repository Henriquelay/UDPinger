use std::{
    convert::TryFrom,
    net::{SocketAddrV4, UdpSocket},
    thread::{self},
    time::{Duration, Instant},
};

use crate::{
    packet::{Packet, Type, MSG_BYTES, PKT_SIZE, SEQ_BYTES},
    util::seq_to_u64,
    EXTRA_TIME_TO_WAIT, MAX_PINGS, TIME_BETEEN_PINGS,
};

fn fetch(pkt: Packet, socket: &UdpSocket) -> Result<Instant, String> {
    let ping_msg: Vec<u8> = pkt.into();

    if cfg!(debug_assertions) {
        println!(
            "Sending packet {}",
            u64::from_le_bytes([
                pkt.seq[0], pkt.seq[1], pkt.seq[2], pkt.seq[3], pkt.seq[4], 0, 0, 0
            ]),
            // u8_to_py_hex_str(&ping_msg)
        );
        // println!("Chars: \"{}\"", String::from_utf8_lossy(&pkt.message));
    }

    // dbg!(ping_msg.bytes());
    // dbg!(ping_msg.bytes().count());

    let start = Instant::now();
    socket
        .send(&ping_msg[0..MSG_BYTES])
        .map_err(|_| "Error sending ping")?;

    #[cfg(debug_assertions)]
    println!(
        "Packet {} sent",
        u64::from_le_bytes([pkt.seq[0], pkt.seq[1], pkt.seq[2], pkt.seq[3], pkt.seq[4], 0, 0, 0]),
    );

    Ok(start)
}

fn listen(socket: &UdpSocket, time_until_drop: Duration) -> Result<(Packet, usize), String> {
    let mut buf = [0; PKT_SIZE];

    socket
        .set_read_timeout(Some(time_until_drop))
        .expect("Error setting read timeout");

    dbg!("Antes de ouvir");
    let res_size = socket.recv(&mut buf).map_err(|e| {
        dbg!(e);
        "Recv failed"
    })?;
    dbg!("Depois de ouvir");

    // print response
    #[cfg(debug_assertions)]
    println!(
        "Received packet {}",
        seq_to_u64(&buf[0..SEQ_BYTES]),
        // u8_to_py_hex_str(buf[8..res_size].as_ref())
    );

    let pkt = Packet::try_from(buf[..res_size].to_vec())?;

    Ok((pkt, res_size))
}

type SendHandle = Vec<(thread::JoinHandle<Result<Instant, String>>, Packet)>;
type ListenHandle = Vec<thread::JoinHandle<Result<(Duration, Packet), String>>>;

fn listen_all(
    send_handles: SendHandle,
    socket: &UdpSocket,
    client_url: SocketAddrV4,
    pkts: &[Packet],
) -> Result<ListenHandle, String> {
    let mut listen_handles = vec![];
    let len = send_handles.len();
    for (i, (send_handle, _)) in send_handles.into_iter().enumerate() {
        let ttl = (TIME_BETEEN_PINGS * (len - i) as u32) + EXTRA_TIME_TO_WAIT;
        let socket = socket.try_clone().map_err(|_| "Error cloning socket")?;

        let pkts = pkts.to_vec();

        // wait for the ping to be sent
        let start = send_handle
            .join()
            .map_err(|_| "Error joining thread interna??")??;

        let listen_handle = thread::spawn(move || -> Result<(Duration, Packet), String> {
            socket
                .connect(client_url)
                .map_err(|_| "Error connecting socket")?;
            let (pkt, received_bytes_count) = listen(&socket, ttl)?;
            let elapsed = start.elapsed();

            let expected_pkt = &pkts[seq_to_u64(&pkt.seq) as usize];

            if pkt.seq != expected_pkt.seq {
                return Err("Received packet with wrong sequence number".to_string());
            }

            // if pkt.message != expected_pkt.message {
            //     return Err("Received packet with different message".to_string());
            // }

            if u32::from_le_bytes(pkt.timestamp) > u32::from_le_bytes(expected_pkt.timestamp) {
                return Err("Received packet with invalid timestamp".to_string());
            }

            if pkt.type_ != Type::Pong {
                return Err("Received packet with wrong type".to_string());
            }

            println!(
                "{} byte from {}: seq={} ttl={} time={:?}",
                received_bytes_count,
                client_url,
                seq_to_u64(&pkt.seq),
                ttl.as_secs(),
                elapsed
            );

            if cfg!(debug_assertions) {
                // println!(
                //     "Chars:\t\"{}\"",
                //     String::from_utf8_lossy(pkt.message.as_ref())
                // );
                println!("Elapsed:{:?}", elapsed);
            }
            Ok((elapsed, pkt))
        });

        listen_handles.push(listen_handle);
    }
    Ok(listen_handles)
}

// TODO remove this lint
#[allow(clippy::too_many_lines)]
pub fn analyze(
    pkts: &mut [Packet],
    socket: &UdpSocket,
    client_url: SocketAddrV4,
) -> Result<(), String> {
    // create a thread for each ping and send the ping
    let mut send_handles = vec![];
    for (i, &mut pkt) in pkts.iter_mut().enumerate() {
        let socket = socket.try_clone().map_err(|_| "Error cloning socket")?;
        socket
            .connect(client_url)
            .map_err(|_| "Error connecting socket")?;
        let fetch = thread::spawn(move || {
            thread::sleep(TIME_BETEEN_PINGS * (i + 1) as u32);
            fetch(pkt, &socket)
        });
        send_handles.push((fetch, pkt));
    }

    // Analyse every pong asynchronously
    let listen_handles = listen_all(send_handles, socket, client_url, pkts)?;

    let mut elapseds = vec![];
    let mut received_pkt = vec![];
    for handle in listen_handles {
        let elapsed = handle.join().map_err(|e| {
            dbg!(e);
            "Error joining thread: Could not listen to message"
        })?;
        match elapsed {
            Ok((elapsed, pkt)) => {
                elapseds.push(elapsed);
                received_pkt.push(pkt);
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }

    // print stats
    let receiveds = elapseds.len();
    #[allow(clippy::cast_precision_loss)]
    let packet_loss = (1.0 - (receiveds as f64 / pkts.len() as f64)) * 100.0;
    print!(
        "--- {} UDP ping statistics ---\n{} packets transmitted, {} packets received, {:.2}% packet loss",
        client_url,
        MAX_PINGS,
        receiveds,
        packet_loss
    );

    if receiveds > 0 {
        // calc stats
        let min = elapseds.iter().min().ok_or("Error getting min")?;
        let max = elapseds.iter().max().ok_or("Error getting max")?;
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
        println!(
            ", time {:?}, min/avg/max/mdev = {:?}/{:?}/{:?}/{:?}",
            total, min, avg, max, mdev
        );
    }
    Ok(())
}
