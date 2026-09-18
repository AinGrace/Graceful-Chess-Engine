use std::{
    sync::{
        Arc, LazyLock,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

use parking_lot::Mutex;
use position::position::Position;
use types::{chess_move::Move, score::Score};

use crate::{
    search::{SearchOptions, SearchResult},
    stats::EngineStats,
    time_control::TimeControl,
    tt::{TT, TTOptions},
};

mod eval;
mod mvv_lva;
mod search;
mod stats;
pub mod time_control;
mod tt;

const DEFAULT_SEARCH_DEPTH: u8 = search::MAX_DEPTH;

static STATISTICS: LazyLock<Mutex<Option<EngineStats>>> = LazyLock::new(|| Mutex::new(None));

pub fn stats() -> Option<EngineStats> {
    STATISTICS.lock().replace(EngineStats::new())
}

#[derive(Default)]
pub struct Engine {
    pos: Position,
    tt: TTOptions,
    stop_thinking: Arc<AtomicBool>,
    is_thinking: Arc<AtomicBool>,
}

impl Engine {
    pub fn new(pos: Position, tt_size_in_mb: u16) -> Self {
        let tt = if tt_size_in_mb != 0 {
            TTOptions::Enabled(Arc::new(TT::new(tt_size_in_mb).into()))
        } else {
            TTOptions::Disabled
        };

        Self {
            pos,
            tt: tt,
            stop_thinking: AtomicBool::default().into(),
            is_thinking: AtomicBool::default().into(),
        }
    }

    pub fn collect_stats(&mut self) {
        let mut guard = STATISTICS.lock();
        if guard.is_none() {
            let mut stats = EngineStats::new();
            stats.advance_new_game();
            guard.replace(stats);
        }
    }

    pub fn clear(&mut self) {
        self.pos = Position::default();
        self.stop_thinking.store(false, Ordering::Release);
        self.tt.clear();

        if let Some(ref mut stats) = *STATISTICS.lock() {
            stats.advance_new_game();
        }
    }

    pub fn search<F, U>(
        &self,
        depth: Option<u8>,
        time_control: TimeControl,
        mut intermediate_result_consumer: F,
        mut final_result_consumer: U,
    ) where
        F: FnMut(&SearchResult) + Send + 'static,
        U: FnMut(Option<Move>) + Send + 'static,
    {
        if self
            .is_thinking
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
        {
            let pos = self.pos.clone();
            let tt = self.tt.clone();
            let is_thinking = Arc::clone(&self.is_thinking);
            let stop_thinking = Arc::clone(&self.stop_thinking);

            if let Some(ref mut stats) = *STATISTICS.lock() {
                stats.push_time_control(time_control.clone());
            }

            thread::spawn(move || {
                let search_options = SearchOptions {
                    pos: &mut pos.clone(),
                    search_depth: depth.unwrap_or(DEFAULT_SEARCH_DEPTH),
                    tt_opts: tt,
                    stop_thinking: &stop_thinking,
                    time_control: time_control.clone(),
                };

                let SearchResult { best_move, .. } = search::search(search_options, |a| {
                    if let Some(ref mut stats) = *STATISTICS.lock() {
                        stats.push_search_res(a.clone());
                    }
                    intermediate_result_consumer(a)
                });

                is_thinking.store(false, Ordering::Release);

                final_result_consumer(best_move);
            });
        }
    }

    pub fn eval(&self) -> Score {
        eval::static_eval(&self.pos)
    }

    pub fn new_position(&mut self, new_pos: Position) {
        self.pos = new_pos;

        if let Some(ref mut stats) = *STATISTICS.lock() {
            stats.push_new_pos(self.pos.clone());
        }
    }

    pub fn pos(&self) -> &Position {
        &self.pos
    }

    pub fn stop_search(&self) {
        if self.is_thinking.load(Ordering::Relaxed) {
            self.stop_thinking.store(true, Ordering::Relaxed);
        }
    }
}
