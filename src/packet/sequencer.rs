use crate::clock::Tick;
use crate::packet::{MAX_PACKET_SIZE, Packet};

pub struct ReliableMessage {
    pub id: u32,
    pub timestamp: Tick,
    pub size: usize,
    pub message: [u8; MAX_PACKET_SIZE],
}

impl ReliableMessage {
    pub fn new(id: u32, message: &[u8]) -> Self {
        let len = message.len();

        // Is rust a real language? How the fuck do I initialize the struct without this unnecessary variable?
        let mut new_message: [u8; MAX_PACKET_SIZE] = [0; MAX_PACKET_SIZE];
        new_message[..len].copy_from_slice(&message);

        Self {
            id,
            timestamp: Tick::now(),
            size: len,
            message: new_message,
        }
    }
}

pub struct OutboundChunkedPacket {
    data: Vec<u8>,
    index: usize,

    max_outbound: usize,
    // This is the list of outbound ids that we are waiting for acks on.
    // When we are sending the huge message, we want to wait for acks to come in before
    // continuing to send more, so we only have a limited number of outbound reliables.
    outbound_ids: Vec<u32>,
}

impl OutboundChunkedPacket {
    pub fn new(message: &[u8], max_outbound: usize) -> Self {
        Self {
            data: message.to_vec(),
            index: 0,
            max_outbound,
            outbound_ids: Vec::new(),
        }
    }

    pub fn get_remaining(&self) -> usize {
        self.data.len() - self.index
    }
}

pub struct PacketSequencer {
    pub next_process_id: u32,
    pub next_reliable_gen_id: u32,
    pub reliable_sent: Vec<ReliableMessage>,
    pub reliable_queue: Vec<ReliableMessage>,

    pub outbound_chunked: Option<OutboundChunkedPacket>,
}

impl Iterator for PacketSequencer {
    type Item = Packet;

    // This will produce raw packets that should be sent.
    fn next(&mut self) -> Option<Self::Item> {
        // TODO: Go through reliable sent and determine if we should resend.

        if let Some(outbound_chunked) = &mut self.outbound_chunked {
            if outbound_chunked.outbound_ids.len() < outbound_chunked.max_outbound {
                outbound_chunked.max_outbound += 1;

                let header_size = 6;

                let mut size = outbound_chunked.get_remaining();
                if size > MAX_PACKET_SIZE - header_size {
                    size = MAX_PACKET_SIZE - header_size;
                }

                let mut data = [0; MAX_PACKET_SIZE];

                data[0] = 0x00;
                data[1] = 0x0A;
                data[2..6].copy_from_slice(&(size as u32).to_le_bytes());
                data[6..size + 6].copy_from_slice(
                    &outbound_chunked.data[outbound_chunked.index..outbound_chunked.index + size],
                );

                outbound_chunked.index += size;

                if outbound_chunked.get_remaining() == 0 {
                    self.outbound_chunked = None;
                }

                return Some(Packet::new(&data[..size]));
            }

            self.outbound_chunked = None;
        }

        None
    }
}

impl PacketSequencer {
    pub fn new() -> Self {
        Self {
            next_process_id: 0,
            next_reliable_gen_id: 0,
            reliable_sent: Vec::new(),
            reliable_queue: Vec::new(),

            outbound_chunked: None,
        }
    }

    pub fn pop_process_queue(&mut self) -> Option<ReliableMessage> {
        if let Some(index) = self
            .reliable_queue
            .iter()
            .position(|msg| msg.id == self.next_process_id)
        {
            self.next_process_id = self.next_process_id.wrapping_add(1);

            return Some(self.reliable_queue.swap_remove(index));
        }

        None
    }

    pub fn handle_ack(&mut self, id: u32) {
        if let Some(index) = self.reliable_sent.iter().position(|msg| msg.id == id) {
            self.reliable_sent.swap_remove(index);

            if let Some(outbound_chunked) = &mut self.outbound_chunked {
                if let Some(index) = outbound_chunked
                    .outbound_ids
                    .iter()
                    .position(|outbound_id| *outbound_id == id)
                {
                    outbound_chunked.outbound_ids.swap_remove(index);
                }
            }
        }
    }

    pub fn increment_id(&mut self) {
        self.next_reliable_gen_id = self.next_reliable_gen_id.wrapping_add(1);
    }
}
