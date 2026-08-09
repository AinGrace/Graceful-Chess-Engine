use types::chess_move::Move;

use crate::{eval::Score, search::Bound};

#[derive(Default, Clone, Debug)]
pub struct TTEntry {
    pub hash: u64,
    pub depth: u8,
    pub score: Score,
    pub best_move: Option<Move>,
    pub bound: Bound, // TODO
}

pub struct TT {
    entries: Box<[TTEntry]>,
    size: usize,
}

impl TT {
    pub const DEFAULT_SIZE_MB: u16 = 64;

    pub fn new(size_in_mb: u16) -> Self {
        let size = ((size_in_mb as usize) * 1024 * 1024) / size_of::<TTEntry>();
        let normalized_size = size.next_power_of_two();

        Self {
            entries: vec![TTEntry::default(); normalized_size].into_boxed_slice(),
            size: normalized_size,
        }
    }

    pub fn get(&self, hash: u64, depth: u8) -> Option<&TTEntry> {
        if let Some(entry) = self.entries.get(self.index(hash))
            && entry.hash == hash
            && entry.depth == depth
        {
            Some(entry)
        } else {
            None
        }
    }

    pub fn insert(&mut self, hash: u64, depth: u8, score: Score, mv: Option<Move>, bound: Bound) {
        let idx = self.index(hash);
        self.entries[idx] = TTEntry {
            hash,
            depth,
            score,
            best_move: mv,
            bound,
        }
    }

    fn index(&self, hash: u64) -> usize {
        hash as usize & (self.size.next_power_of_two() - 1)
    }
}

impl Default for TT {
    fn default() -> Self {
        Self::new(Self::DEFAULT_SIZE_MB)
    }
}
