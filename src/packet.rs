use rand::Rng;
use std::sync::Mutex;

pub const SEQ_BYTES: usize = 5;
pub const TIMESTAMP_BYTES: usize = 4;
pub const REQTYPE_BYTES: usize = 1;
pub const MSG_BYTES: usize = 30;

// Assuming all messages have the `Tipo da requisição` field, and that has a bit set to 1 when it's a PONG type message
pub const PKT_SIZE: usize = SEQ_BYTES + REQTYPE_BYTES + TIMESTAMP_BYTES + MSG_BYTES;

static MSG_COUNTER: Mutex<usize> = Mutex::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
    Ping,
    Pong,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Packet {
    pub seq: [u8; 5],
    pub type_: Type,
    pub timestamp: [u8; 4],
    pub message: [u8; MSG_BYTES],
}

impl Default for Packet {
    fn default() -> Self {
        let mut counter = MSG_COUNTER.lock().expect("Mutex poisoned");
        let current = *counter;
        *counter += 1;
        Self::new(&mut rand::thread_rng(), current)
    }
}

impl Packet {
    fn new(rng: &mut impl Rng, index: usize) -> Self {
        let seq = index.to_le_bytes()[..5]
            .try_into()
            .expect("Error converting usize to [u8; 5]");

        Self {
            seq,
            type_: Type::Ping,
            timestamp: [2; 4],
            message: rand_message(rng, index),
        }
    }
}

// Deserialization
impl From<Vec<u8>> for Packet {
    fn from(pkt_bytes: Vec<u8>) -> Self {
        let len = pkt_bytes.len();
        assert!(len >= PKT_SIZE - MSG_BYTES, "Invalid packet size {}", len);

        #[allow(clippy::range_plus_one)]
        let is_ping = pkt_bytes[SEQ_BYTES..SEQ_BYTES + REQTYPE_BYTES]
            .iter()
            .all(|&x| x == 0);
        let pkt_type = if is_ping { Type::Ping } else { Type::Pong };
        let seq = pkt_bytes[..SEQ_BYTES]
            .try_into()
            .expect("Error converting &[u8] to [u8; SEQ_BYTES]");
        let type_ = pkt_type;
        let timestamp = pkt_bytes
            [SEQ_BYTES + REQTYPE_BYTES..SEQ_BYTES + REQTYPE_BYTES + TIMESTAMP_BYTES]
            .try_into()
            .expect("Error converting &[u8] to [u8; TIMESTAMP_BYTES]");
        let mut received_msg_iter = pkt_bytes[SEQ_BYTES + REQTYPE_BYTES + TIMESTAMP_BYTES..].iter();
        let mut message = [0; MSG_BYTES];
        (0..MSG_BYTES).for_each(|byte| {
            message[byte] = *received_msg_iter.nth(byte).unwrap_or(&0);
        });

        Packet {
            seq,
            type_,
            timestamp,
            message,
        }
    }
}

// Serialization
impl From<Packet> for Vec<u8> {
    fn from(pkt: Packet) -> Self {
        let mut pkt_bytes = vec![];

        pkt_bytes.extend_from_slice(&pkt.seq);
        pkt_bytes.extend_from_slice(&{
            let mut ping_slice = [0; REQTYPE_BYTES];
            if let Type::Ping = pkt.type_ {
                ping_slice[0] = 1;
            }
            ping_slice
        });
        pkt_bytes.extend_from_slice(&pkt.timestamp);
        pkt_bytes.extend_from_slice(&pkt.message);

        assert!(
            pkt_bytes.len() == PKT_SIZE,
            "Invalid packet size {}",
            pkt_bytes.len()
        );

        pkt_bytes
    }
}

/// Return a valid random message
pub fn rand_message(_rng: &mut impl Rng, seq: usize) -> [u8; MSG_BYTES] {
    let mut slice = [(65 + seq) as u8; MSG_BYTES];
    for (i, num) in slice.iter_mut().enumerate() {
        *num += i as u8;
    }
    slice
}

#[cfg(test)]
mod test {

    #[test]
    fn sequential_packets() {
        for i in 0..10u8 {
            let pkt = super::Packet::default();
            assert_eq!(pkt.seq[0], i);
        }
    }
}
