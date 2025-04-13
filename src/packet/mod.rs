use crate::clock::Tick;
use std::fmt;

pub mod sequencer;

pub const MAX_PACKET_SIZE: usize = 520;

#[derive(Copy, Clone)]
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

    pub fn concat_u8(self, val: u8) -> Self {
        let mut result = Self {
            data: self.data,
            size: self.size + 1,
        };

        result.data[self.size] = val;

        result
    }

    pub fn concat_u16(self, val: u16) -> Self {
        let mut result = Self {
            data: self.data,
            size: self.size + 2,
        };

        result.data[self.size..self.size + 2].copy_from_slice(&val.to_le_bytes());
        result
    }

    pub fn concat_u32(self, val: u32) -> Self {
        let mut result = Self {
            data: self.data,
            size: self.size + 4,
        };
        result.data[self.size..self.size + 4].copy_from_slice(&val.to_le_bytes());
        result
    }

    pub fn concat_i8(self, val: i8) -> Self {
        let mut result = Self {
            data: self.data,
            size: self.size + 1,
        };

        result.data[self.size] = val as u8;

        result
    }

    pub fn concat_i16(self, val: i16) -> Self {
        let mut result = Self {
            data: self.data,
            size: self.size + 2,
        };

        result.data[self.size..self.size + 2].copy_from_slice(&val.to_le_bytes());
        result
    }

    pub fn concat_i32(self, val: i32) -> Self {
        let mut result = Self {
            data: self.data,
            size: self.size + 4,
        };
        result.data[self.size..self.size + 4].copy_from_slice(&val.to_le_bytes());
        result
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

    pub fn remaining(&self) -> usize {
        MAX_PACKET_SIZE - self.size
    }
}

impl fmt::Debug for Packet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Packet {{ data={:?} size={} }}",
            &self.data[..self.size],
            self.size
        )
    }
}
