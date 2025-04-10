use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_PACKET_SIZE: usize = 520;

fn get_current_tick() -> i32 {
    let now: u128 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    return (now / 10) as i32;
}

struct ReliableMessage {
    id: u32,
    timestamp: i32,
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
            timestamp: get_current_tick(),
            size: len,
            message: new_message,
        }
    }
}

struct PacketSequencer {
    next_process_id: u32,
    next_reliable_gen_id: u32,
    reliable_sent: Vec<ReliableMessage>,
    reliable_queue: Vec<ReliableMessage>,
}

impl PacketSequencer {
    fn new() -> Self {
        Self {
            next_process_id: 0,
            next_reliable_gen_id: 0,
            reliable_sent: Vec::new(),
            reliable_queue: Vec::new(),
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
}

impl Connection {
    fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            packet_sequencer: PacketSequencer::new(),
        }
    }

    fn send(&mut self, game_socket: &mut UdpSocket, message: &[u8]) -> std::io::Result<()> {
        game_socket.send_to(message, &self.addr)?;
        Ok(())
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

                    let timestamp = u32::from_le_bytes(message[2..6].try_into().unwrap());

                    let mut data = [0; MAX_PACKET_SIZE];

                    let packets_sent = 0u32;
                    let packets_recv = 0u32;

                    data[0] = 0x00;
                    data[1] = 0x05;
                    data[2..6].copy_from_slice(&timestamp.to_le_bytes());
                    data[6..10].copy_from_slice(&packets_sent.to_le_bytes());
                    data[10..14].copy_from_slice(&packets_recv.to_le_bytes());

                    let sync_response = &data[..14];
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
                }
                36 => {
                    // Password
                    if size < 66 {
                        return;
                    }

                    let name = std::str::from_utf8(&message[2..34]).unwrap();
                    let _ = std::str::from_utf8(&message[34..66]).unwrap(); // Password

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

    fn send_reliable_message(&mut self, game_socket: &mut UdpSocket, message: &[u8]) -> bool {
        let size = message.len() + 6;
        if size > MAX_PACKET_SIZE {
            return false;
        }

        let rel_mesg = ReliableMessage::new(self.packet_sequencer.next_reliable_gen_id, message);

        let mut data = [0; MAX_PACKET_SIZE];

        data[0] = 0x00;
        data[1] = 0x03;
        data[2..6].copy_from_slice(&rel_mesg.id.to_le_bytes());
        data[6..6 + message.len()].copy_from_slice(&message);

        let data = &data[..];

        println!("Sending reliable message size {}", data.len());

        if let Err(e) = game_socket.send_to(&data, &self.addr) {
            println!("Failed to send reliable message: {}", e);
            return false;
        }

        self.packet_sequencer.reliable_sent.push(rel_mesg);
        self.packet_sequencer.increment_id();

        return true;
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
        let mut buf = [0; 520];
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
                    let _: u32 = u32::from_le_bytes(buf[2..6].try_into().unwrap()); // Key
                    let _: u16 = u16::from_le_bytes(buf[6..8].try_into().unwrap()); // Version

                    let mut response = [0; 7];
                    response[0] = 0x00;
                    response[1] = 0x02;
                    response[2..6].copy_from_slice(&0u32.to_le_bytes());
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
        let mut buf = [0; 520];
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
