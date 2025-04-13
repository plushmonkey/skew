use crate::arena::ARENA_SETTINGS;
use crate::clock::Tick;
use crate::packet::sequencer::*;
use crate::packet::{MAX_PACKET_SIZE, Packet};
use crate::player::*;
use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};

pub mod arena;
pub mod clock;
pub mod packet;
pub mod player;

struct Connection {
    addr: SocketAddr,
    packet_sequencer: PacketSequencer,

    player_id: PlayerId,
    last_packet_time: Tick,

    connected: bool,
}

impl Connection {
    fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            packet_sequencer: PacketSequencer::new(),
            player_id: INVALID_PLAYER_ID,
            last_packet_time: Tick::now(),
            connected: true,
        }
    }

    fn send(&mut self, game_socket: &UdpSocket, packet: Packet) -> std::io::Result<()> {
        let buf = &packet.data[..packet.size];
        println!("Sending: {:?}", buf);
        game_socket.send_to(buf, &self.addr)?;
        Ok(())
    }

    fn send_reliable_message(&mut self, game_socket: &UdpSocket, message: &[u8]) -> bool {
        let size = message.len() + 6;
        if size > MAX_PACKET_SIZE {
            return false;
        }

        let rel_mesg = ReliableMessage::new(self.packet_sequencer.next_reliable_gen_id, message);
        let rel_mesg_packet = Packet::new_reliable(rel_mesg.id, message);

        if let Err(e) = self.send(game_socket, rel_mesg_packet) {
            println!("Failed to send reliable message: {}", e);
            return false;
        }

        self.packet_sequencer.reliable_sent.push(rel_mesg);
        self.packet_sequencer.increment_id();

        return true;
    }

    fn send_small_chunked_message(&mut self, game_socket: &UdpSocket, message: &[u8]) {
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

    fn send_disconnect(&mut self, game_socket: &UdpSocket) {
        let packet = Packet::empty().concat_u8(0x00).concat_u8(0x07);

        if let Err(e) = self.send(game_socket, packet) {
            println!("Failed to send disconnect packet: {}", e);
        }

        self.connected = false;
    }

    fn send_enter_list(&mut self, player_manager: &PlayerManager, game_socket: &UdpSocket) {
        const ENTER_PACKET_SIZE: usize = 64;

        let mut packet = Packet::empty();

        for (id, player) in &player_manager.players {
            if packet.remaining() < ENTER_PACKET_SIZE {
                self.send_reliable_message(game_socket, &packet.data[..packet.size]);

                packet = Packet::empty();
            }

            let mut name_len = player.name.len();
            if name_len > 20 {
                name_len = 20;
            }

            let slice = &mut packet.data[packet.size..packet.size + ENTER_PACKET_SIZE];

            slice[0] = 0x03;
            slice[1] = 0x08;
            slice[3..name_len + 3].copy_from_slice(&player.name.as_bytes()[..name_len]);
            slice[51..53].copy_from_slice(&id.to_le_bytes());
            packet.size += ENTER_PACKET_SIZE;
        }

        if packet.size > 0 {
            self.send_reliable_message(game_socket, &packet.data[..packet.size]);
        }
    }
}

// TODO: Arenas
struct Game {
    player_manager: PlayerManager,
}

impl Game {
    fn new() -> Self {
        Self {
            player_manager: PlayerManager::new(),
        }
    }

    fn on_data(
        &mut self,
        game_socket: &UdpSocket,
        connections: &mut HashMap<SocketAddr, Connection>,
        addr: SocketAddr,
        packet: Packet,
    ) -> bool {
        self.handle_packet(game_socket, connections, addr, packet);

        if let Some(conn) = connections.get_mut(&addr) {
            if let Some(rel_mesg) = conn.packet_sequencer.pop_process_queue() {
                let data = &rel_mesg.message[..rel_mesg.size];

                return self.handle_packet(game_socket, connections, addr, Packet::new(data));
            } else {
                return conn.connected;
            }
        }

        false
    }

    fn handle_packet(
        &mut self,
        game_socket: &UdpSocket,
        connections: &mut HashMap<SocketAddr, Connection>,
        addr: SocketAddr,
        packet: Packet,
    ) -> bool {
        let buf = &packet.data[..packet.size];
        let packet_type = buf[0];

        if packet_type == 0x00 {
            if packet.size < 2 {
                return false;
            }

            let packet_type = buf[1];

            match packet_type {
                3 => {
                    // Reliable message
                    if buf.len() < 7 {
                        if let Some(conn) = connections.get_mut(&addr) {
                            conn.send_disconnect(game_socket);
                        }
                        return false;
                    }

                    let reliable_id: u32 = u32::from_le_bytes(buf[2..6].try_into().unwrap());
                    let ack_pkt = Packet::new_reliable_ack(reliable_id);

                    if let Some(conn) = connections.get_mut(&addr) {
                        if let Err(e) = conn.send(game_socket, ack_pkt) {
                            println!("Failed to send reliable ack: {}", e);
                            conn.send_disconnect(game_socket);
                        }

                        conn.packet_sequencer
                            .reliable_queue
                            .push(ReliableMessage::new(reliable_id, &buf[6..]));
                    }
                }
                4 => {
                    // Reliable message ack
                    if buf.len() < 6 {
                        if let Some(conn) = connections.get_mut(&addr) {
                            conn.send_disconnect(game_socket);
                        }
                    }

                    let id: u32 = u32::from_le_bytes(buf[2..6].try_into().unwrap());

                    if let Some(conn) = connections.get_mut(&addr) {
                        conn.packet_sequencer.handle_ack(id);
                    }
                }

                5 => {
                    // Sync request
                    if buf.len() < 6 {
                        return false;
                    }

                    let recv_timestamp = u32::from_le_bytes(buf[2..6].try_into().unwrap());
                    let sync_response_packet = Packet::new_sync_response(Tick::new(recv_timestamp));

                    if let Some(conn) = connections.get_mut(&addr) {
                        if let Err(e) = conn.send(game_socket, sync_response_packet) {
                            println!("Error sending sync response: {}", e);
                        }
                    }
                }
                7 => {
                    if let Some(conn) = connections.get_mut(&addr) {
                        conn.connected = false;
                        println!("Received disconnect packet");
                    }
                }
                14 => {
                    // Cluster
                    let mut cluster = &buf[2..];

                    while !cluster.is_empty() {
                        let subsize: usize = cluster[0] as usize;

                        if cluster.len() >= subsize + 1 {
                            let subpkt = &cluster[1..subsize + 1];

                            self.handle_packet(
                                game_socket,
                                connections,
                                addr,
                                Packet::new(&subpkt),
                            );
                        } else {
                            break;
                        }

                        cluster = &cluster[subsize + 1..];
                    }
                }
                _ => {}
            }
        } else {
            match packet_type {
                1 => {
                    // ArenaLogin
                    let Some(conn) = connections.get_mut(&addr) else {
                        return false;
                    };

                    let Some(player) = self.player_manager.get_player_by_id(conn.player_id) else {
                        conn.send_disconnect(game_socket);
                        return false;
                    };

                    let mut pid_pkt = [0; 3];
                    let pid: u16 = player.id;

                    pid_pkt[0] = 0x01;
                    pid_pkt[1..3].copy_from_slice(&pid.to_le_bytes());
                    conn.send_reliable_message(game_socket, &pid_pkt);

                    conn.send_small_chunked_message(game_socket, &ARENA_SETTINGS);

                    let mut map_info_pkt = [0; 25];
                    let map_name = "pub.lvl";
                    let map_checksum: u32 = 1889723958;
                    let map_filesize: u32 = 58992;

                    map_info_pkt[0] = 0x29;
                    map_info_pkt[1..map_name.len() + 1].copy_from_slice(map_name.as_bytes());
                    map_info_pkt[17..21].copy_from_slice(&map_checksum.to_le_bytes());
                    map_info_pkt[21..25].copy_from_slice(&map_filesize.to_le_bytes());

                    conn.send_reliable_message(game_socket, &map_info_pkt);

                    conn.send_enter_list(&self.player_manager, game_socket);

                    let mut data = [0; 1];
                    data[0] = 0x02;
                    conn.send_reliable_message(game_socket, &data);
                    self.broadcast_player_enter(game_socket, pid);
                }
                36 => {
                    // Password
                    let Some(conn) = connections.get_mut(&addr) else {
                        return false;
                    };
                    if buf.len() < 66 {
                        conn.send_disconnect(game_socket);
                        return false;
                    }

                    let name = std::str::from_utf8(&buf[2..34]).unwrap();
                    let _ = std::str::from_utf8(&buf[34..66]).unwrap(); // Password

                    println!("Name: {}", name);

                    if let Some(player) = self.player_manager.create_player(addr) {
                        player.name = name.into();

                        conn.player_id = player.id;
                    } else {
                        println!("Failed to create player for: {:?}", name);
                        conn.send_disconnect(game_socket);
                        return false;
                    }

                    // Send version packet
                    let mut data = [0; MAX_PACKET_SIZE];

                    let checksum: u32 = 0xC9B61486;

                    data[0] = 0x34;
                    data[1] = 40;
                    data[2] = 0x00;
                    data[3..7].copy_from_slice(&checksum.to_le_bytes());

                    let version_pkt = &data[..7];
                    conn.send_reliable_message(game_socket, &version_pkt);

                    let server_version: u32 = 134;
                    let subspace_checksum: u32 = 0;

                    data[0] = 0x0A;
                    data[1] = 0x00;
                    data[2..6].copy_from_slice(&server_version.to_le_bytes());
                    data[10..14].copy_from_slice(&subspace_checksum.to_le_bytes());

                    let password_response_pkt = &data[..36];
                    conn.send_reliable_message(game_socket, &password_response_pkt);
                }
                _ => {}
            }
        }

        if let Some(conn) = connections.get(&addr) {
            return conn.connected;
        }
        return false;
    }

    fn broadcast_player_enter(&mut self, game_socket: &UdpSocket, player_id: PlayerId) {
        const ENTER_PACKET_SIZE: usize = 64;
        let mut packet = Packet::empty();

        let Some(join_player) = self.player_manager.get_player_by_id(player_id) else {
            return;
        };

        let mut name_len = join_player.name.len();
        if name_len > 20 {
            name_len = 20;
        }

        packet.data[0] = 0x03;
        packet.data[1] = 0x08;
        packet.data[3..name_len + 3].copy_from_slice(&join_player.name.as_bytes()[..name_len]);
        packet.data[51..53].copy_from_slice(&join_player.id.to_le_bytes());
        packet.size = ENTER_PACKET_SIZE;

        for (id, player) in self.player_manager.players.iter() {
            if *id == player_id {
                continue;
            }

            if let Err(e) = game_socket.send_to(&packet.data[..packet.size], player.addr) {
                println!("Error sending player enter: {}", e);
            }
        }
    }

    fn broadcast_player_leave(&mut self, game_socket: &UdpSocket, player_id: PlayerId) {
        self.player_manager.remove_player(player_id);

        let packet = Packet::empty().concat_u8(0x04).concat_u16(player_id);

        // TODO: Check arena and player in valid state to receive
        for (_, player) in &self.player_manager.players {
            if let Err(e) = game_socket.send_to(&packet.data[..packet.size], &player.addr) {
                println!("Failed to send player leave: {}", e);
            }
        }
    }
}

struct Server {
    ping_socket: UdpSocket,
    game_socket: UdpSocket,

    connections: HashMap<SocketAddr, Connection>,
    game: Game,
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
            game: Game::new(),
        })
    }

    fn remove_connection(&mut self, addr: SocketAddr) {
        let Some(player_id) = self
            .connections
            .get(&addr)
            .and_then(|conn| Some(conn.player_id))
        else {
            return;
        };

        if let Some(conn) = self.connections.remove(&addr).as_mut() {
            conn.send_disconnect(&mut self.game_socket);
        }

        self.game
            .broadcast_player_leave(&self.game_socket, player_id);
    }

    fn timeout_connection(&mut self) {
        const TIMEOUT_TICKS: i32 = 1000;
        let mut remove_addr = None;

        let now = Tick::now();

        // Times out the first connection that hasn't sent data recently.
        for (addr, connection) in &mut self.connections {
            let ticks_since_data = now.diff(&connection.last_packet_time);

            if ticks_since_data >= TIMEOUT_TICKS {
                println!("Timing out {:?}", addr);
                remove_addr = Some(addr.to_owned());
                break;
            }
        }

        if let Some(addr) = remove_addr {
            self.remove_connection(addr);
        }
    }

    fn poll_game(&mut self) -> std::io::Result<()> {
        self.timeout_connection();

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

        println!("Recv: {:?}", buf);

        connection.last_packet_time = Tick::now();

        let addr = connection.addr.clone();

        if !self.game.on_data(
            &self.game_socket,
            &mut self.connections,
            addr,
            Packet::new(buf),
        ) {
            self.remove_connection(addr);
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
