use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_PACKET_SIZE: usize = 520;

// TODO: Read and serialize
const ARENA_SETTINGS: [u8; 1428] = [
    15, 1, 7, 0, 112, 23, 0, 0, 160, 15, 0, 0, 220, 5, 100, 0, 20, 0, 30, 0, 44, 1, 50, 0, 14, 1,
    150, 0, 208, 7, 213, 7, 0, 0, 244, 1, 100, 0, 77, 1, 100, 0, 250, 0, 34, 1, 19, 0, 178, 12,
    126, 4, 164, 6, 34, 1, 18, 0, 184, 11, 232, 3, 64, 6, 40, 0, 2, 0, 250, 0, 166, 0, 100, 0, 0,
    0, 144, 1, 184, 11, 1, 0, 125, 0, 24, 0, 50, 0, 150, 0, 125, 0, 232, 3, 75, 0, 44, 1, 100, 0,
    80, 0, 224, 46, 12, 0, 64, 0, 196, 9, 10, 24, 5, 3, 3, 3, 3, 3, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0,
    72, 80, 181, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 112, 23, 0, 0, 160, 15, 0, 0,
    220, 5, 100, 0, 20, 0, 30, 0, 44, 1, 50, 0, 14, 1, 150, 0, 208, 7, 213, 7, 0, 0, 244, 1, 100,
    0, 77, 1, 100, 0, 250, 0, 230, 0, 17, 0, 166, 14, 126, 4, 164, 6, 200, 0, 17, 0, 116, 14, 232,
    3, 64, 6, 40, 0, 2, 0, 250, 0, 166, 0, 100, 0, 176, 4, 144, 1, 184, 11, 1, 0, 125, 0, 24, 0,
    50, 0, 150, 0, 125, 0, 232, 3, 75, 0, 44, 1, 100, 0, 80, 0, 224, 46, 12, 0, 64, 0, 196, 9, 10,
    24, 5, 3, 3, 3, 3, 3, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 72, 80, 189, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 112, 23, 0, 0, 160, 15, 0, 0, 220, 5, 100, 0, 20, 0, 30, 0, 44, 1, 50, 0,
    14, 1, 150, 0, 208, 7, 213, 7, 0, 0, 244, 1, 100, 0, 77, 1, 100, 0, 250, 0, 230, 0, 18, 0, 178,
    12, 126, 4, 164, 6, 200, 0, 17, 0, 184, 11, 76, 4, 64, 6, 40, 0, 2, 0, 250, 0, 166, 0, 100, 0,
    176, 4, 144, 1, 184, 11, 1, 0, 125, 0, 24, 0, 50, 0, 150, 0, 125, 0, 232, 3, 75, 0, 44, 1, 100,
    0, 80, 0, 224, 46, 12, 0, 64, 0, 196, 9, 10, 24, 5, 3, 3, 3, 3, 3, 3, 3, 0, 0, 0, 0, 0, 1, 0,
    0, 72, 100, 53, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 112, 23, 0, 0, 160, 15, 0,
    0, 220, 5, 100, 0, 20, 0, 30, 0, 44, 1, 50, 0, 14, 1, 150, 0, 208, 7, 213, 7, 0, 0, 244, 1,
    100, 0, 77, 1, 100, 0, 250, 0, 230, 0, 18, 0, 178, 12, 126, 4, 164, 6, 200, 0, 17, 0, 184, 11,
    232, 3, 64, 6, 40, 0, 2, 0, 250, 0, 166, 0, 100, 0, 176, 4, 144, 1, 184, 11, 1, 0, 125, 0, 24,
    0, 50, 0, 150, 0, 125, 0, 232, 3, 75, 0, 44, 1, 100, 0, 80, 0, 224, 46, 12, 0, 64, 0, 196, 9,
    10, 24, 5, 3, 3, 3, 3, 3, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 72, 80, 245, 3, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 112, 23, 0, 0, 160, 15, 0, 0, 220, 5, 100, 0, 20, 0, 30, 0, 44, 1, 50,
    0, 14, 1, 150, 0, 208, 7, 213, 7, 0, 0, 244, 1, 100, 0, 77, 1, 100, 0, 250, 0, 230, 0, 18, 0,
    178, 12, 126, 4, 164, 6, 200, 0, 17, 0, 184, 11, 232, 3, 64, 6, 40, 0, 2, 0, 250, 0, 166, 0,
    100, 0, 176, 4, 144, 1, 184, 11, 1, 0, 125, 0, 29, 0, 50, 0, 150, 0, 125, 0, 232, 3, 75, 0, 44,
    1, 100, 0, 80, 0, 224, 46, 12, 0, 64, 0, 196, 9, 10, 24, 5, 3, 3, 3, 3, 3, 3, 3, 0, 0, 0, 0, 0,
    0, 0, 0, 72, 80, 53, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 112, 23, 0, 0, 160, 15,
    0, 0, 220, 5, 100, 0, 20, 0, 30, 0, 44, 1, 50, 0, 14, 1, 150, 0, 208, 7, 213, 7, 0, 0, 244, 1,
    100, 0, 77, 1, 100, 0, 250, 0, 230, 0, 18, 0, 178, 12, 126, 4, 164, 6, 200, 0, 17, 0, 184, 11,
    232, 3, 64, 6, 40, 0, 2, 0, 250, 0, 166, 0, 100, 0, 176, 4, 144, 1, 184, 11, 1, 0, 125, 0, 24,
    0, 50, 0, 150, 0, 125, 0, 232, 3, 75, 0, 44, 1, 100, 0, 80, 0, 224, 46, 12, 0, 64, 0, 196, 9,
    10, 24, 5, 3, 3, 3, 3, 3, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 64, 80, 181, 26, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 112, 23, 0, 0, 160, 15, 0, 0, 220, 5, 100, 0, 20, 0, 30, 0, 44, 1,
    50, 0, 14, 1, 150, 0, 208, 7, 213, 7, 0, 0, 244, 1, 100, 0, 77, 1, 100, 0, 250, 0, 230, 0, 18,
    0, 178, 12, 126, 4, 164, 6, 200, 0, 17, 0, 184, 11, 232, 3, 64, 6, 40, 0, 2, 0, 250, 0, 166, 0,
    100, 0, 176, 4, 144, 1, 184, 11, 1, 0, 125, 0, 24, 0, 50, 0, 150, 0, 125, 0, 232, 3, 75, 0, 44,
    1, 100, 0, 80, 0, 224, 46, 12, 0, 64, 0, 196, 9, 10, 24, 5, 3, 3, 3, 3, 3, 3, 3, 0, 0, 0, 0, 0,
    0, 0, 2, 72, 80, 181, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 112, 23, 0, 0, 160,
    15, 0, 0, 220, 5, 100, 0, 20, 0, 30, 0, 44, 1, 50, 0, 44, 1, 150, 0, 208, 7, 213, 7, 1, 0, 244,
    1, 44, 1, 44, 1, 100, 0, 250, 0, 4, 1, 18, 0, 178, 12, 126, 4, 164, 6, 200, 0, 17, 0, 184, 11,
    232, 3, 64, 6, 40, 0, 2, 0, 250, 0, 166, 0, 100, 0, 176, 4, 144, 1, 184, 11, 1, 0, 125, 0, 16,
    0, 40, 0, 150, 0, 125, 0, 232, 3, 75, 0, 44, 1, 100, 0, 80, 0, 224, 46, 12, 0, 64, 0, 196, 9,
    10, 24, 5, 3, 3, 3, 3, 3, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 72, 80, 53, 2, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 112, 17, 1, 0, 176, 113, 11, 0, 38, 2, 0, 0, 112, 23, 0, 0, 184, 11, 0,
    0, 64, 105, 71, 0, 132, 3, 0, 0, 15, 39, 0, 0, 104, 16, 0, 0, 224, 46, 0, 0, 48, 87, 5, 0, 88,
    21, 1, 0, 80, 70, 0, 0, 112, 23, 0, 0, 25, 0, 0, 0, 148, 17, 0, 0, 184, 11, 0, 0, 232, 3, 0, 0,
    0, 0, 0, 0, 184, 11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 244, 1, 185, 0, 10,
    0, 80, 0, 32, 3, 72, 0, 200, 0, 188, 2, 3, 0, 20, 0, 22, 0, 10, 0, 15, 0, 0, 0, 0, 0, 39, 1, 0,
    2, 232, 3, 1, 0, 0, 0, 48, 12, 44, 1, 0, 1, 0, 0, 112, 23, 160, 15, 32, 78, 88, 2, 176, 4, 254,
    255, 200, 0, 200, 0, 10, 0, 224, 46, 192, 1, 144, 1, 232, 3, 128, 0, 112, 23, 0, 0, 232, 3,
    232, 3, 50, 0, 90, 0, 232, 3, 232, 3, 0, 0, 20, 0, 100, 0, 208, 7, 0, 0, 0, 0, 44, 1, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 3, 4, 2, 12, 1, 0, 0, 0, 1, 85, 1, 0, 1, 1, 0, 0, 0, 0, 1, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 70, 90, 50, 40, 30, 20, 5, 60, 60, 40, 80, 70, 60, 3, 30, 40, 5, 2, 60,
    10, 40, 5, 10, 15, 20, 10, 10, 30,
];

struct Tick {
    value: u32,
}

impl Tick {
    fn now() -> Self {
        let tick: u128 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let tick = (tick / 10) as u32;

        Self {
            value: tick & 0x7FFFFFFF,
        }
    }

    fn new(value: u32) -> Self {
        Self {
            value: value & 0x7FFFFFFF,
        }
    }

    fn diff(self, other: &Tick) -> i32 {
        let first: i32 = (self.value << 1) as i32;
        let second: i32 = (other.value << 1) as i32;

        (first - second) >> 1
    }

    fn gt(self, other: &Tick) -> bool {
        self.diff(other) > 0
    }

    fn gte(self, other: &Tick) -> bool {
        self.diff(other) >= 0
    }
}

struct ReliableMessage {
    id: u32,
    timestamp: Tick,
    size: usize,
    message: [u8; MAX_PACKET_SIZE],
}

impl ReliableMessage {
    fn new(id: u32, message: &[u8]) -> Self {
        println!("Reliable message len: {}", message.len());
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

struct OutboundChunkedPacket {
    data: Vec<u8>,
    index: usize,

    max_outbound: usize,
    // This is the list of outbound ids that we are waiting for acks on.
    // When we are sending the huge message, we want to wait for acks to come in before
    // continuing to send more, so we only have a limited number of outbound reliables.
    outbound_ids: Vec<u32>,
}

impl OutboundChunkedPacket {
    fn new(message: &[u8], max_outbound: usize) -> Self {
        Self {
            data: message.to_vec(),
            index: 0,
            max_outbound,
            outbound_ids: Vec::new(),
        }
    }

    fn get_remaining(&self) -> usize {
        self.data.len() - self.index
    }
}

struct PacketSequencer {
    next_process_id: u32,
    next_reliable_gen_id: u32,
    reliable_sent: Vec<ReliableMessage>,
    reliable_queue: Vec<ReliableMessage>,

    outbound_chunked: Option<OutboundChunkedPacket>,
}

struct Packet {
    data: [u8; MAX_PACKET_SIZE],
    size: usize,
}

impl Packet {
    fn new(message: &[u8]) -> Self {
        let size = message.len();
        let mut data = [0; MAX_PACKET_SIZE];

        data[..size].copy_from_slice(message);

        Self { data, size }
    }
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
    fn new() -> Self {
        Self {
            next_process_id: 0,
            next_reliable_gen_id: 0,
            reliable_sent: Vec::new(),
            reliable_queue: Vec::new(),

            outbound_chunked: None,
        }
    }

    fn pop_process_queue(&mut self) -> Option<ReliableMessage> {
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

    fn handle_ack(&mut self, id: u32) {
        if let Some(index) = self.reliable_sent.iter().position(|msg| msg.id == id) {
            println!("Found reliable ack");
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
        } else {
            println!("Didnt find reliable ack");
        }
    }

    fn increment_id(&mut self) {
        self.next_reliable_gen_id = self.next_reliable_gen_id.wrapping_add(1);
    }
}

struct PidSet {
    bits: [u64; 16],
}

impl PidSet {
    fn new() -> Self {
        Self { bits: [0u64; 16] }
    }

    fn clear(&mut self) {
        self.bits = [0u64; 16];
    }

    fn set(&mut self, id: u16) {
        if id >= 1024 {
            return;
        }

        let array_index = (id / 64) as usize;
        let bit_index = (id % 64) as usize;

        self.bits[array_index] |= 1 << bit_index;
    }

    fn test(&mut self, id: u16) -> bool {
        if id >= 1024 {
            return true;
        }

        let array_index = (id / 64) as usize;
        let bit_index = (id % 64) as usize;

        return self.bits[array_index] & (1 << bit_index) != 0;
    }

    fn get_and_set_free_pid(&mut self) -> Option<u16> {
        for i in 0..1024 {
            if self.test(i as u16) {
                self.set(i as u16);
                return Some(i);
            }
        }
        None
    }
}

struct Player {
    id: u16,
    name: [u8; 32],
}

impl Player {
    fn new(id: u16) -> Self {
        Self { id, name: [0; 32] }
    }
}

struct PlayerManager {
    players: HashMap<u16, Player>,
    pidset: PidSet,
}

impl PlayerManager {
    fn new() -> Self {
        Self {
            players: HashMap::new(),
            pidset: PidSet::new(),
        }
    }

    fn get_player_by_id(&mut self, id: u16) -> Option<&mut Player> {
        self.players.get_mut(&id)
    }

    fn create_player(&mut self) -> Option<&mut Player> {
        if let Some(pid) = self.pidset.get_and_set_free_pid() {
            let player = Player::new(pid);

            self.players.insert(pid, player);

            return self.get_player_by_id(pid);
        }
        None
    }
}

struct Connection {
    addr: SocketAddr,
    packet_sequencer: PacketSequencer,

    name: String,
}

impl Connection {
    fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            packet_sequencer: PacketSequencer::new(),
            name: String::new(),
        }
    }

    fn handle_packet(&mut self, game_socket: &mut UdpSocket, message: &[u8]) {
        if message.len() < 1 {
            return;
        }

        let size = message.len();
        let pkt_type = message[0];

        if pkt_type == 0 {
            // Special case core packet handling here
            if size <= 2 {
                // TODO: Send disconnect
                return;
            }

            let pkt_type = message[1];
            println!("Got core: {:?}", message);

            match pkt_type {
                1 => {}
                2 => {}
                3 => {
                    if let Err(e) = self.handle_reliable_message(game_socket, message) {
                        println!("Error handling reliable message: {}", e);
                    }
                }
                4 => {
                    if size < 6 {
                        return;
                    }

                    let id: u32 = u32::from_le_bytes(message[2..6].try_into().unwrap());
                    self.packet_sequencer.handle_ack(id);
                }
                5 => {
                    // Sync request
                    if size < 6 {
                        return;
                    }

                    let recv_timestamp = u32::from_le_bytes(message[2..6].try_into().unwrap());
                    let local_timestamp = Tick::now().value;

                    let mut data = [0; MAX_PACKET_SIZE];

                    data[0] = 0x00;
                    data[1] = 0x06;
                    data[2..6].copy_from_slice(&recv_timestamp.to_le_bytes());
                    data[6..10].copy_from_slice(&local_timestamp.to_le_bytes());

                    let sync_response = &data[..10];
                    if let Err(e) = self.send(game_socket, &sync_response) {
                        println!("Error sending sync response: {}", e);
                    }
                }
                14 => {
                    // Cluster
                    let mut cluster = &message[2..];

                    while !cluster.is_empty() {
                        let subsize: usize = cluster[0] as usize;

                        if cluster.len() >= subsize + 1 {
                            let subpkt = &cluster[1..subsize + 1];

                            self.handle_packet(game_socket, &subpkt);
                        } else {
                            break;
                        }

                        cluster = &cluster[subsize + 1..];
                    }
                }
                _ => {
                    return;
                }
            }
        } else {
            println!("Got pkt type {}", pkt_type);
            match pkt_type {
                1 => {
                    // ArenaLogin
                    let mut pid_pkt = [0; 3];
                    let pid: u16 = 0;

                    pid_pkt[0] = 0x01;
                    pid_pkt[1..3].copy_from_slice(&pid.to_le_bytes());
                    self.send_reliable_message(game_socket, &pid_pkt);

                    self.send_small_chunked_message(game_socket, &ARENA_SETTINGS);

                    let mut map_info_pkt = [0; 25];
                    let map_name = "pub.lvl";
                    let map_checksum: u32 = 1889723958;
                    let map_filesize: u32 = 58992;

                    map_info_pkt[0] = 0x29;
                    map_info_pkt[1..map_name.len() + 1].copy_from_slice(map_name.as_bytes());
                    map_info_pkt[17..21].copy_from_slice(&map_checksum.to_le_bytes());
                    map_info_pkt[21..25].copy_from_slice(&map_filesize.to_le_bytes());

                    self.send_reliable_message(game_socket, &map_info_pkt);

                    let mut enter_pkt = [0; 64];
                    enter_pkt[0] = 0x03;
                    enter_pkt[1] = 0x08;
                    enter_pkt[2] = 0x00;
                    enter_pkt[3..self.name.len() + 3].copy_from_slice(self.name.as_bytes());
                    enter_pkt[51..53].copy_from_slice(&pid.to_le_bytes());

                    self.send_reliable_message(game_socket, &enter_pkt[..]);

                    let mut data = [0; 1];
                    data[0] = 0x02;
                    self.send_reliable_message(game_socket, &data);
                }
                36 => {
                    // Password
                    if size < 66 {
                        return;
                    }

                    let name = std::str::from_utf8(&message[2..34]).unwrap();
                    let _ = std::str::from_utf8(&message[34..66]).unwrap(); // Password

                    self.name = name.into();
                    println!("Name: {}", name);

                    // Send version packet
                    let mut data = [0; MAX_PACKET_SIZE];

                    let checksum: u32 = 0xC9B61486;

                    data[0] = 0x34;
                    data[1] = 40;
                    data[2] = 0x00;
                    data[3..7].copy_from_slice(&checksum.to_le_bytes());

                    let version_pkt = &data[..7];
                    self.send_reliable_message(game_socket, &version_pkt);

                    let server_version: u32 = 134;
                    let subspace_checksum: u32 = 0;

                    data[0] = 0x0A;
                    data[1] = 0x00;
                    data[2..6].copy_from_slice(&server_version.to_le_bytes());
                    data[10..14].copy_from_slice(&subspace_checksum.to_le_bytes());

                    let password_response_pkt = &data[..36];
                    self.send_reliable_message(game_socket, &password_response_pkt);
                }
                _ => {
                    return;
                }
            }
        }
    }

    fn handle_reliable_message(
        &mut self,
        game_socket: &mut UdpSocket,
        message: &[u8],
    ) -> std::io::Result<()> {
        if message.len() < 7 {
            return Ok(());
        }

        println!("reliable_data: {:?}", message);

        let reliable_id: u32 = u32::from_le_bytes(message[2..6].try_into().unwrap());

        println!("Reliable id: {}", reliable_id);

        let mut ack_pkt = [0; 6];
        ack_pkt[0] = 0x00;
        ack_pkt[1] = 0x04;
        ack_pkt[2..6].copy_from_slice(&reliable_id.to_le_bytes());

        self.send(game_socket, &ack_pkt)?;
        println!("Sending reliable ack");
        println!("{:?}", ack_pkt);

        println!("Message size: {}", message.len());
        self.packet_sequencer
            .reliable_queue
            .push(ReliableMessage::new(reliable_id, &message[6..]));

        Ok(())
    }

    fn send(&mut self, game_socket: &mut UdpSocket, message: &[u8]) -> std::io::Result<()> {
        println!("Sending: {:?}", message);
        game_socket.send_to(message, &self.addr)?;
        Ok(())
    }

    fn send_packet(&mut self, game_socket: &mut UdpSocket, packet: &Packet) -> std::io::Result<()> {
        self.send(game_socket, &packet.data[..packet.size])?;
        Ok(())
    }

    fn send_reliable_message(&mut self, game_socket: &mut UdpSocket, message: &[u8]) -> bool {
        let size = message.len() + 6;
        if size > MAX_PACKET_SIZE {
            return false;
        }

        let rel_mesg = ReliableMessage::new(self.packet_sequencer.next_reliable_gen_id, message);

        let mut data = [0; MAX_PACKET_SIZE];
        let data_size: usize = message.len() + 6;

        data[0] = 0x00;
        data[1] = 0x03;
        data[2..6].copy_from_slice(&rel_mesg.id.to_le_bytes());
        data[6..data_size].copy_from_slice(&message);

        let data = &data[..data_size];

        println!("Sending reliable message size {}", data.len());

        if let Err(e) = self.send(game_socket, &data) {
            println!("Failed to send reliable message: {}", e);
            return false;
        }

        self.packet_sequencer.reliable_sent.push(rel_mesg);
        self.packet_sequencer.increment_id();

        return true;
    }

    fn send_small_chunked_message(&mut self, game_socket: &mut UdpSocket, message: &[u8]) {
        // Header size includes reliable message header and small chunk header size
        const HEADER_SIZE: usize = 2 + 6;

        let mut current = &message[..];

        while !current.is_empty() {
            let mut size = current.len();

            if size > MAX_PACKET_SIZE - HEADER_SIZE {
                size = MAX_PACKET_SIZE - HEADER_SIZE;
            }

            let mut data = [0; MAX_PACKET_SIZE];
            data[0] = 0x00;
            data[1] = 0x08;
            data[2..2 + size].copy_from_slice(&current[..size]);

            if size == current.len() {
                data[1] = 0x09;
            }

            self.send_reliable_message(game_socket, &data[..size + 2]);

            current = &current[size..];
        }
    }
}

struct Server {
    ping_socket: UdpSocket,
    game_socket: UdpSocket,

    connections: HashMap<SocketAddr, Connection>,
}

impl Server {
    fn new(port: u16) -> Result<Self, std::io::Error> {
        let game_socket = UdpSocket::bind(format!("0.0.0.0:{}", port))?;
        let ping_socket = UdpSocket::bind(format!("0.0.0.0:{}", port + 1))?;

        game_socket.set_nonblocking(true)?;
        ping_socket.set_nonblocking(true)?;

        Ok(Self {
            ping_socket,
            game_socket,
            connections: HashMap::new(),
        })
    }

    fn poll_game(&mut self) -> std::io::Result<()> {
        let mut buf = [0; MAX_PACKET_SIZE];
        let (size, src) = match self.game_socket.recv_from(&mut buf) {
            Ok(r) => Ok((r.0, r.1)),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    return Ok(());
                }
                Err(e)
            }
        }?;

        let buf = &buf[..size];

        let connection = match self.connections.get_mut(&src) {
            Some(connection) => connection,
            None => {
                let pkt_type = buf[0];
                if pkt_type == 0x00 && size >= 8 && buf[1] == 0x01 {
                    let key: u32 = u32::from_le_bytes(buf[2..6].try_into().unwrap()); // Key
                    let _: u16 = u16::from_le_bytes(buf[6..8].try_into().unwrap()); // Version

                    let mut response = [0; 7];
                    response[0] = 0x00;
                    response[1] = 0x02;
                    response[2..6].copy_from_slice(&key.to_le_bytes()); // Send key back to disable encryption
                    response[6] = 0x00; // No billing

                    self.game_socket.send_to(&response, &src)?;

                    self.connections.insert(src, Connection::new(src));
                    println!("Adding new connection");
                }
                return Ok(());
            }
        };

        println!("Found existing connection {}", buf.len());

        connection.handle_packet(&mut self.game_socket, buf);

        while let Some(rel_mesg) = connection.packet_sequencer.pop_process_queue() {
            println!("Processing reliable packet in connection");
            connection.handle_packet(&mut self.game_socket, &rel_mesg.message);
        }

        Ok(())
    }

    fn poll_ping(&self) -> std::io::Result<()> {
        let mut buf = [0; MAX_PACKET_SIZE];
        let (size, src) = match self.ping_socket.recv_from(&mut buf) {
            Ok(r) => Ok((r.0, r.1)),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    return Ok(());
                }
                Err(e)
            }
        }?;

        if size != 4 {
            return Ok(());
        }

        let buf = &mut buf[..4];
        let timestamp = u32::from_le_bytes(buf.try_into().unwrap());

        let mut pong_pkt = [0; 8];

        let player_count = 69u32;

        pong_pkt[..4].copy_from_slice(&player_count.to_le_bytes());
        pong_pkt[4..8].copy_from_slice(&timestamp.to_le_bytes());

        self.ping_socket.send_to(&pong_pkt, &src)?;
        Ok(())
    }
}

fn main() -> std::io::Result<()> {
    let mut server = Server::new(5000)?;

    loop {
        server.poll_ping()?;
        server.poll_game()?;
    }
}
