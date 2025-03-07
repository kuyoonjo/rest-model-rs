use std::sync::atomic::{AtomicUsize, Ordering};

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use std::{convert::TryInto, time::SystemTime};

use hex::{self};
use once_cell::sync::Lazy;
use rand::{random, rng, Rng};

const TIMESTAMP_SIZE: usize = 4;
const PROCESS_ID_SIZE: usize = 5;
const COUNTER_SIZE: usize = 3;

const TIMESTAMP_OFFSET: usize = 0;
const PROCESS_ID_OFFSET: usize = TIMESTAMP_OFFSET + TIMESTAMP_SIZE;
const COUNTER_OFFSET: usize = PROCESS_ID_OFFSET + PROCESS_ID_SIZE;

const MAX_U24: usize = 0xFF_FFFF;

static OID_COUNTER: Lazy<AtomicUsize> =
    Lazy::new(|| AtomicUsize::new(rng().random_range(0..=MAX_U24)));

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct ObjectId {
    id: [u8; 12],
}

impl ObjectId {
    /// Generates a new [`ObjectId`], represented in bytes.
    /// See the [docs](http://www.mongodb.com/docs/manual/reference/object-id/)
    /// for more information.
    pub fn new() -> Self {
        let timestamp = Self::gen_timestamp();
        let process_id = Self::gen_process_id();
        let counter = Self::gen_count();

        Self::from_parts(timestamp, process_id, counter)
    }

    /// Constructs a new ObjectId wrapper around the raw byte representation.
    pub const fn from_bytes(bytes: [u8; 12]) -> ObjectId {
        ObjectId { id: bytes }
    }

    /// Construct an `ObjectId` from its parts.
    /// See the [docs](http://www.mongodb.com/docs/manual/reference/object-id/)
    /// for more information.
    pub fn from_parts(seconds_since_epoch: u32, process_id: [u8; 5], counter: [u8; 3]) -> Self {
        let mut bytes = [0; 12];

        bytes[TIMESTAMP_OFFSET..(TIMESTAMP_OFFSET + TIMESTAMP_SIZE)]
            .clone_from_slice(&u32::to_be_bytes(seconds_since_epoch));
        bytes[PROCESS_ID_OFFSET..(PROCESS_ID_OFFSET + PROCESS_ID_SIZE)]
            .clone_from_slice(&process_id);
        bytes[COUNTER_OFFSET..(COUNTER_OFFSET + COUNTER_SIZE)].clone_from_slice(&counter);

        Self::from_bytes(bytes)
    }

    /// Convert this [`ObjectId`] to its hex string representation.
    pub fn to_hex(self) -> String {
        hex::encode(self.id)
    }

    /// Generates a new timestamp representing the current seconds since epoch.
    fn gen_timestamp() -> u32 {
        #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        let timestamp: u32 = (js_sys::Date::now() / 1000.0) as u32;
        #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
        let timestamp: u32 = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("system clock is before 1970")
            .as_secs()
            .try_into()
            .unwrap(); // will succeed until 2106 since timestamp is unsigned

        timestamp
    }

    /// Generate a random 5-byte array.
    fn gen_process_id() -> [u8; 5] {
        static BUF: Lazy<[u8; 5]> = Lazy::new(random);

        *BUF
    }

    /// Gets an incremental 3-byte count.
    /// Represented in Big Endian.
    fn gen_count() -> [u8; 3] {
        let u_counter = OID_COUNTER.fetch_add(1, Ordering::SeqCst);

        // Mod result instead of OID_COUNTER to prevent threading issues.
        let u = u_counter % (MAX_U24 + 1);

        // Convert usize to writable u64, then extract the first three bytes.
        let u_int = u as u64;

        let buf = u_int.to_be_bytes();
        let buf_u24: [u8; 3] = [buf[5], buf[6], buf[7]];
        buf_u24
    }
}
