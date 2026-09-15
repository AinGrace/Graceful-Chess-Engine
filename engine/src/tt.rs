use std::sync::Arc;

use parking_lot::Mutex;
use types::chess_move::Move;

use crate::{eval::Score, search::Bound, tt::TTOptions::Enabled};

#[derive(Clone)]
pub enum TTOptions {
    Enabled(Arc<Mutex<TT>>),
    Disabled,
}

impl TTOptions {
    pub fn probe(&self, z_hash: u64, depth: u8) -> Option<TTEntry> {
        if let Enabled(tt) = self {
            let tt = tt.lock();
            tt.get(z_hash, depth).cloned()
        } else {
            None
        }
    }

    pub fn insert(
        &self,
        z_hash: u64,
        depth: u8,
        score: Score,
        best_move: Option<Move>,
        bound: Bound,
    ) {
        if let Enabled(tt) = self {
            let mut tt = tt.lock();
            tt.insert(z_hash, depth, score, best_move, bound);
        }
    }

    pub fn clear(&self) {
        if let Enabled(tt) = self {
            let mut tt = tt.lock();
            tt.clear();
        }
    }
}

impl Default for TTOptions {
    fn default() -> Self {
        Self::Enabled(Arc::new(TT::default().into()))
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct TTEntry {
    pub hash: u64,
    pub depth: u8,
    pub score: Score,
    pub best_move: Option<Move>,
    pub bound: Bound,
}

pub struct TT {
    entries: Box<[TTEntry]>,
    size: usize,
}

impl TT {
    pub const DEFAULT_SIZE_MB: u16 = 256;

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
            && ((entry.depth == 0 && depth == 0) || (entry.depth >= depth && depth > 0))
        {
            Some(entry)
        } else {
            None
        }
    }

    pub fn insert(&mut self, hash: u64, depth: u8, score: Score, mv: Option<Move>, bound: Bound) {
        if let Some(entry) = self.get(hash, depth)
            && (entry.depth > depth)
        {
            return;
        }

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

    fn clear(&mut self) {
        self.entries.fill(TTEntry::default());
    }
}

impl Default for TT {
    fn default() -> Self {
        Self::new(Self::DEFAULT_SIZE_MB)
    }
}
