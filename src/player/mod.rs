use std::collections::HashMap;
use std::net::SocketAddr;

pub type PlayerId = u16;
pub const INVALID_PLAYER_ID: PlayerId = 65535;

#[derive(Debug)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,

    pub addr: SocketAddr,
}

impl Player {
    pub fn new(addr: SocketAddr, id: PlayerId) -> Self {
        Self {
            id,
            name: String::new(),
            addr,
        }
    }
}

pub struct PlayerManager {
    pub players: HashMap<PlayerId, Player>,
    pidset: PidSet,
}

impl PlayerManager {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
            pidset: PidSet::new(),
        }
    }

    pub fn get_player_by_id(&mut self, id: PlayerId) -> Option<&mut Player> {
        if !self.pidset.test(id) {
            println!("Pid {} is not set in pidset.", id);
            return None;
        }
        println!("getting player from pid {}", id);
        self.players.get_mut(&id)
    }

    pub fn create_player(&mut self, addr: SocketAddr) -> Option<&mut Player> {
        if let Some(pid) = self.pidset.get_and_set_free_pid() {
            let player = Player::new(addr, pid);

            self.players.insert(pid, player);

            return self.get_player_by_id(pid);
        }
        None
    }

    pub fn remove_player(&mut self, pid: PlayerId) {
        self.players.remove(&pid);
    }
}

struct PidSet {
    bits: [u64; 16],
}

impl PidSet {
    fn new() -> Self {
        Self { bits: [0u64; 16] }
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
            if !self.test(i as u16) {
                self.set(i as u16);
                return Some(i);
            }
        }
        None
    }
}
