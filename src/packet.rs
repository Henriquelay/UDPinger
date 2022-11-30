use rand::Rng;
use std::sync::Mutex;

pub const SEQ_BYTES: usize = 5;
pub const REQTYPE_BYTES: usize = 1;
pub const TIMESTAMP_BYTES: usize = 4;
pub const MSG_BYTES: usize = 30;

// Assuming all messages have the `Tipo da requisição` field, and that has a bit set to 1 when it's a PONG type message
pub const PKT_SIZE: usize = SEQ_BYTES + REQTYPE_BYTES + TIMESTAMP_BYTES + MSG_BYTES;

static MSG_COUNTER: Mutex<usize> = Mutex::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    Ping,
    Pong,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Packet {
    pub seq: [u8; 5],
    pub type_: Type,
    pub timestamp: [u8; 4],
    pub message: [u8; MSG_BYTES],
}

impl Ord for Packet {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.seq.cmp(&other.seq)
    }
}

impl PartialOrd for Packet {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
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
impl TryFrom<Vec<u8>> for Packet {
    type Error = &'static str;

    fn try_from(pkt_bytes: Vec<u8>) -> Result<Self, Self::Error> {
        let len = pkt_bytes.len();
        if len < PKT_SIZE - MSG_BYTES {
            return Err("Invalid packet size");
        }

        #[allow(clippy::range_plus_one)]
        let pkt_type = match pkt_bytes
            .get(SEQ_BYTES..SEQ_BYTES + REQTYPE_BYTES)
            .ok_or("Invalid packet size")?
        {
            [0] => Type::Pong,
            [1] => Type::Ping,
            _ => return Err("Invalid packet type"),
        };

        let seq = pkt_bytes[..SEQ_BYTES]
            .try_into()
            .map_err(|_| "Error converting &[u8] to [u8; SEQ_BYTES]")?;
        let type_ = pkt_type;
        let timestamp = pkt_bytes
            [SEQ_BYTES + REQTYPE_BYTES..SEQ_BYTES + REQTYPE_BYTES + TIMESTAMP_BYTES]
            .try_into()
            .map_err(|_| "Error converting &[u8] to [u8; TIMESTAMP_BYTES]")?;
        let mut received_msg_iter = pkt_bytes[SEQ_BYTES + REQTYPE_BYTES + TIMESTAMP_BYTES..].iter();
        let mut message = [0; MSG_BYTES];
        (0..MSG_BYTES).for_each(|byte| {
            message[byte] = *received_msg_iter.nth(byte).unwrap_or(&0);
        });

        Ok(Packet {
            seq,
            type_,
            timestamp,
            message,
        })
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

const NAME: &str = "HenriqueCoutinhoLayber";
const NAME_LEN: usize = 22;

/// Return a valid random message
pub fn rand_message(_rng: &mut impl Rng, _seq: usize) -> [u8; MSG_BYTES] {
    let bytes = &NAME.as_bytes()[0..22];
    let mut message = [0; MSG_BYTES];
    message[..bytes.len()].copy_from_slice(bytes);
    // fill the rest with 0
    message[bytes.len()..].copy_from_slice(&[0; MSG_BYTES - NAME_LEN]);
    message
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
