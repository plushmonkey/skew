use crate::clock::Tick;

pub mod sequencer;

pub const MAX_PACKET_SIZE: usize = 520;

pub struct Packet {
    pub data: [u8; MAX_PACKET_SIZE],
    pub size: usize,
}

impl Packet {
    pub fn empty() -> Self {
        Self {
            data: [0; MAX_PACKET_SIZE],
            size: 0,
        }
    }

    pub fn new(message: &[u8]) -> Self {
        let size = message.len();
        let mut data = [0; MAX_PACKET_SIZE];

        data[..size].copy_from_slice(message);

        Self { data, size }
    }

    pub fn new_reliable(id: u32, message: &[u8]) -> Self {
        let size = message.len() + 6;
        let mut data = [0; MAX_PACKET_SIZE];

        data[0] = 0x00;
        data[1] = 0x03;
        data[2..6].copy_from_slice(&id.to_le_bytes());
        data[6..message.len() + 6].copy_from_slice(message);

        Self { data, size }
    }

    pub fn new_reliable_ack(id: u32) -> Self {
        let size = 6;
        let mut data = [0; MAX_PACKET_SIZE];

        data[0] = 0x00;
        data[1] = 0x04;
        data[2..6].copy_from_slice(&id.to_le_bytes());

        Self { data, size }
    }

    pub fn new_sync_response(recv_timestamp: Tick) -> Self {
        let size = 10;
        let mut data = [0; MAX_PACKET_SIZE];

        let local_timestamp = Tick::now();

        data[0] = 0x00;
        data[1] = 0x06;
        data[2..6].copy_from_slice(&recv_timestamp.value().to_le_bytes());
        data[6..10].copy_from_slice(&local_timestamp.value().to_le_bytes());

        Self { data, size }
    }

    pub fn write_u8(&mut self, val: u8) {
        self.data[self.size] = val;
        self.size += 1;
    }

    pub fn write_u16(&mut self, val: u16) {
        self.data[self.size..self.size + 2].copy_from_slice(&val.to_le_bytes());
        self.size += 2;
    }

    pub fn write_u32(&mut self, val: u32) {
        self.data[self.size..self.size + 4].copy_from_slice(&val.to_le_bytes());
        self.size += 4;
    }

    pub fn write_i8(&mut self, val: i8) {
        self.data[self.size] = val as u8;
        self.size += 1;
    }

    pub fn write_i16(&mut self, val: i16) {
        self.data[self.size..self.size + 2].copy_from_slice(&val.to_le_bytes());
        self.size += 2;
    }

    pub fn write_i32(&mut self, val: i32) {
        self.data[self.size..self.size + 4].copy_from_slice(&val.to_le_bytes());
        self.size += 4;
    }
}
