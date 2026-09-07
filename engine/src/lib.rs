use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

use position::position::{InvalidMoveError, Position, Undo};
use types::chess_move::Move;

use crate::{
    eval::Score,
    search::{SearchOptions, SearchResult},
    time_control::TimeControl,
    tt::{TT, TTOptions},
};

mod eval;
mod mvv_lva;
mod search;
pub mod time_control;
mod tt;

const DEFAULT_SEARCH_DEPTH: u8 = search::MAX_DEPTH;

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

    pub fn search<F, U>(
        &self,
        depth: Option<u8>,
        time_control: TimeControl,
        intermediate_result_consumer: F,
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
            let mut pos = self.pos.clone();
            let tt = self.tt.clone();
            let is_thinking = Arc::clone(&self.is_thinking);
            let stop_thinking = Arc::clone(&self.stop_thinking);

            thread::spawn(move || {
                let search_options = SearchOptions {
                    pos: &mut pos,
                    search_depth: depth.unwrap_or(DEFAULT_SEARCH_DEPTH),
                    tt_opts: tt,
                    stop_thinking: &stop_thinking,
                    time_control: time_control,
                };

                let SearchResult { best_move, .. } =
                    search::search(search_options, intermediate_result_consumer);

                is_thinking.store(false, Ordering::Release);

                final_result_consumer(best_move);
            });
        }
    }

    pub fn eval(&self) -> Score {
        eval::static_eval(&self.pos)
    }

    pub fn set_pos(&mut self, new_pos: Position) {
        self.pos = new_pos;
    }

    pub fn pos(&self) -> &Position {
        &self.pos
    }

    pub fn make_move(&mut self, mv: Move) -> Result<Undo, InvalidMoveError> {
        self.pos.do_move(mv)
    }

    pub fn stop_search(&self) {
        if self.is_thinking.load(Ordering::Relaxed) {
            self.stop_thinking.store(true, Ordering::Relaxed);
        }
    }
}
