use std::collections::HashMap;

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

pub struct Player {
    pub id: u16,
    pub name: [u8; 32],
}

impl Player {
    pub fn new(id: u16) -> Self {
        Self { id, name: [0; 32] }
    }
}

pub struct PlayerManager {
    pub players: HashMap<u16, Player>,
    pidset: PidSet,
}

impl PlayerManager {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
            pidset: PidSet::new(),
        }
    }

    pub fn get_player_by_id(&mut self, id: u16) -> Option<&mut Player> {
        self.players.get_mut(&id)
    }

    pub fn create_player(&mut self) -> Option<&mut Player> {
        if let Some(pid) = self.pidset.get_and_set_free_pid() {
            let player = Player::new(pid);

            self.players.insert(pid, player);

            return self.get_player_by_id(pid);
        }
        None
    }
}
