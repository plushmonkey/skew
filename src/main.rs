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
            if size <= 2 {
                return;
            }

            let pkt_type = message[1];

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
                    let sync_response_packet = Packet::new_sync_response(Tick::new(recv_timestamp));

                    if let Err(e) = self.send(game_socket, sync_response_packet) {
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

        let reliable_id: u32 = u32::from_le_bytes(message[2..6].try_into().unwrap());
        let ack_pkt = Packet::new_reliable_ack(reliable_id);

        self.send(game_socket, ack_pkt)?;

        self.packet_sequencer
            .reliable_queue
            .push(ReliableMessage::new(reliable_id, &message[6..]));

        Ok(())
    }

    fn send(&mut self, game_socket: &mut UdpSocket, packet: Packet) -> std::io::Result<()> {
        let buf = &packet.data[..packet.size];
        println!("Sending: {:?}", buf);
        game_socket.send_to(buf, &self.addr)?;
        Ok(())
    }

    fn send_reliable_message(&mut self, game_socket: &mut UdpSocket, message: &[u8]) -> bool {
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

    player_manager: PlayerManager,
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
            player_manager: PlayerManager::new(),
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

        println!("Recv: {:?}", buf);

        connection.handle_packet(&mut self.game_socket, buf);

        while let Some(rel_mesg) = connection.packet_sequencer.pop_process_queue() {
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
