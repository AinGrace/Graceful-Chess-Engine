use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, atomic::AtomicBool},
    time::{Duration, Instant},
};

use engine::{
    eval,
    search::{self, SearchResult, TTOptions},
    tt::TT,
};
use engine::{eval::Score, search::SearchOptions};
use position::{
    fen::Fen,
    position::{Position, Undo},
};
use ratatui::widgets::ListState;
use types::{MoveList, chess_move::Move, color::Color, piece::Piece, role::Role, square::Square};

use crate::copy_to_clipboard;

const DEFAULT_SEARCH_DEPTH: u8 = 4;

pub enum Scrolling {
    Up { col: u16, row: u16 },

    Down { col: u16, row: u16 },
}

impl Scrolling {
    pub fn col(&self) -> u16 {
        match self {
            Scrolling::Up { col, .. } | Scrolling::Down { col, .. } => *col,
        }
    }

    pub fn row(&self) -> u16 {
        match self {
            Scrolling::Up { row, .. } | Scrolling::Down { row, .. } => *row,
        }
    }
}

#[derive(Clone, Copy)]
pub enum FocusMode {
    MoveInput,
    FenInput,
}

pub struct EngineModel {
    pos: Position,
    legal_moves: MoveList,

    tt: Arc<Mutex<TT>>,

    best_move: Option<Move>,
    best_move_score: Score,
    static_evaluation_score: Score,

    undo: Vec<Undo>,

    search_time: Duration,
    search_depth: u8,
}

impl EngineModel {
    pub fn new() -> Self {
        let pos = Position::new();
        let legal_moves = pos.legal_moves();

        let tt = TT::new(TT::DEFAULT_SIZE_MB);
        let wrapped_tt = Arc::new(Mutex::new(tt));
        let static_evaluation_score = eval::static_eval(&pos);
        let undo = Vec::with_capacity(128);
        let search_depth = DEFAULT_SEARCH_DEPTH;

        let mut res = Self {
            pos,
            legal_moves,
            tt: wrapped_tt,
            best_move: None,
            best_move_score: Score::Centipawn(0),
            static_evaluation_score,
            undo,
            search_time: Duration::from_nanos(0),
            search_depth,
        };

        let tt_opts = TTOptions::Enabled(Arc::clone(&res.tt));

        let search_opts = SearchOptions {
            pos: &mut res.pos,
            search_depth: Some(6),
            tt: tt_opts,
            // FIXME: ui thread
            stop_flag: &Arc::new(AtomicBool::new(false)),
        };

        let before_search = Instant::now();
        let SearchResult { score, best_move } = search::search(search_opts);
        let after_search = Instant::now();

        let search_time = after_search.duration_since(before_search);
        res.search_time = search_time;
        res.best_move = best_move;
        res.best_move_score = score;

        res
    }

    pub fn make_move(&mut self, raw_uci: &str) -> Option<Move> {
        let Some(mv) = self
            .legal_moves
            .iter()
            .find(|mv| mv.to_uci() == raw_uci)
            .and_then(|mv| Some(*mv))
        else {
            return None;
        };

        let undo = self.pos.do_move_inner(mv);
        self.undo.push(undo);

        self.legal_moves = self.pos.legal_moves();

        self.init_search();

        Some(mv)
    }

    pub fn undo_move(&mut self) -> bool {
        let Some(undo) = self.undo.pop() else {
            return false;
        };

        self.pos.undo_move(undo);
        self.legal_moves = self.pos.legal_moves();
        self.init_search();

        true
    }

    pub fn apply_fen(&mut self, new_pos: Position) {
        self.pos = new_pos;
        self.legal_moves = self.pos.legal_moves();
        self.undo.clear();
        self.search_depth = DEFAULT_SEARCH_DEPTH;

        self.init_search();
    }

    fn init_search(&mut self) {
        let search_depth = self.search_depth;

        let tt_opts = TTOptions::Enabled(Arc::clone(&self.tt));

        let search_opts = SearchOptions {
            pos: &mut self.pos,
            search_depth: search_depth.into(),
            tt: tt_opts,
            // FIXME: ui thread
            stop_flag: &Arc::new(AtomicBool::new(false)),
        };
        let time_begin = Instant::now();
        let SearchResult { score, best_move } = search::search(search_opts);
        let time_end = Instant::now();

        self.best_move_score = score;
        self.best_move = best_move;
        self.search_time = time_end.duration_since(time_begin);
    }
}

pub struct Model {
    engine: EngineModel,
    partial_move: Vec<char>,
    partial_fen: Vec<char>,
    move_history: Vec<Move>,
    logs: VecDeque<String>,
    log_state: ListState,
    scrolling: Option<Scrolling>,
    focus: FocusMode,
    exit: bool,
}

impl Model {
    pub fn new() -> Self {
        let engine = EngineModel::new();
        let partial_move = Vec::with_capacity(5);
        let partial_fen = Vec::with_capacity(20);
        let move_history = Vec::with_capacity(128);

        let logs = VecDeque::new();
        let log_state = ListState::default();

        let focus = FocusMode::MoveInput;

        let scrolling = None;

        let exit = false;

        Self {
            engine,
            partial_move,
            partial_fen,
            move_history,
            logs,
            log_state,
            focus,
            scrolling,
            exit,
        }
    }

    pub fn should_exit(&self) -> bool {
        self.exit
    }

    pub fn logs(&self) -> &[String] {
        self.logs.as_slices().0
    }

    pub fn legal_moves(&self) -> &MoveList {
        &self.engine.legal_moves
    }

    pub fn position(&self) -> &Position {
        &self.engine.pos
    }

    pub fn position_mut(&mut self) -> &mut Position {
        &mut self.engine.pos
    }

    pub fn best_move(&self) -> Option<Move> {
        self.engine.best_move
    }

    pub fn best_move_score(&self) -> Score {
        self.engine.best_move_score
    }

    pub fn static_eval(&self) -> Score {
        self.engine.static_evaluation_score
    }

    pub fn search_time(&self) -> Duration {
        self.engine.search_time
    }

    pub fn search_depth(&self) -> u8 {
        self.engine.search_depth
    }

    pub fn is_legal_origin(&self, from: Square) -> bool {
        self.legal_moves().iter().any(|mv| mv.from() == from)
    }

    pub fn is_legal_dest(&self, role: Role, to: Square) -> bool {
        self.legal_moves()
            .iter()
            .any(|mv| mv.role() == role && mv.to() == to)
    }

    pub fn copy_fen_to_clipboard(&mut self) {
        match copy_to_clipboard(&self.position_fen()) {
            Ok(_) => (),
            Err(e) => self.info_log(e.to_string()),
        }
    }

    pub fn has_legal_origin_and_destination(&self, from: Square, to: Square) -> bool {
        self.legal_moves()
            .iter()
            .any(|mv| mv.from() == from && mv.to() == to)
    }

    pub fn legal_moves_of(&self, from: Square) -> Option<Vec<String>> {
        self.legal_moves()
            .iter()
            .cloned()
            .filter(|mv| mv.from() == from)
            .map(|mv| mv.to_uci())
            .map(Some)
            .collect()
    }

    pub fn position_fen(&self) -> String {
        self.position().into_fen().to_string()
    }

    pub fn left_partial_move(&self) -> Option<String> {
        if self.partial_move.len() < 2 {
            None
        } else {
            Some(self.partial_move_to_string()[..2].into())
        }
    }

    pub fn right_partial_move(&self) -> Option<String> {
        if self.partial_move.len() < 4 {
            None
        } else {
            Some(self.partial_move_to_string()[2..=3].into())
        }
    }

    pub fn best_move_to_uci(&self) -> String {
        self.best_move()
            .map(|mv| mv.to_uci())
            .unwrap_or("None".into())
    }

    pub fn partial_move_to_string(&self) -> String {
        self.partial_move.iter().collect()
    }

    pub fn partial_fen_to_string(&self) -> String {
        self.partial_fen.iter().collect()
    }

    pub fn ep_square_to_str(&self) -> String {
        self.position()
            .ep_square()
            .map(|sqr| sqr.to_string())
            .unwrap_or("None".into())
    }

    pub fn move_history_iter(&self) -> impl Iterator<Item = String> {
        self.move_history.iter().map(|mv| mv.to_uci())
    }

    pub fn turn(&self) -> Color {
        self.position().turn()
    }

    pub fn peek_piece_at(&self, sqr: Square) -> Option<Piece> {
        self.position().board().peek(sqr)
    }

    pub fn half_moves(&self) -> u32 {
        self.position().half_moves()
    }

    pub fn full_moves(&self) -> u32 {
        self.position().full_moves()
    }

    pub fn z_hash(&self) -> u64 {
        self.position().zobrist_hash()
    }

    pub fn focus(&self) -> FocusMode {
        self.focus
    }

    pub fn log_state(&mut self) -> &mut ListState {
        &mut self.log_state
    }

    pub fn make_move(&mut self, raw_uci: &str) {
        let Some(applied_move) = self.engine.make_move(raw_uci) else {
            return;
        };

        self.move_history.push(applied_move);
        self.partial_move.clear();

        self.info_log(format!("applied move [{raw_uci}]"));
    }

    pub fn undo_move(&mut self) {
        if !self.engine.undo_move() {
            return;
        }

        let unmade_move = self
            .move_history
            .pop()
            .expect("move history is not empty")
            .to_uci();

        self.info_log(format!("reversed the move {unmade_move}"));
    }

    pub fn init_search(&mut self) {
        self.engine.init_search();
    }

    pub fn play_best_move(&mut self) {
        if let Some(best_move) = self.engine.best_move.map(|mv| mv.to_uci()).take() {
            self.make_move(&best_move);
        }
    }

    pub fn quit(&mut self) {
        self.exit = true
    }

    pub fn push_char(&mut self, chr: char) {
        match self.focus {
            FocusMode::MoveInput => self.push_partial_move(chr),
            FocusMode::FenInput => self.push_partial_fen(chr),
        }
    }

    pub fn push_partial_move(&mut self, chr: char) {
        let valid_chars = matches!(chr, 'a'..='h' | '1'..='8');
        if matches!(self.focus, FocusMode::MoveInput) && valid_chars {
            self.partial_move.push(chr);
        }
    }

    pub fn pop_partial_move(&mut self) {
        if matches!(self.focus, FocusMode::MoveInput) {
            self.partial_move.pop();
        }
    }

    pub fn push_partial_fen(&mut self, chr: char) {
        if matches!(self.focus, FocusMode::FenInput) {
            self.partial_fen.push(chr);
        }
    }

    pub fn pop_partial_fen(&mut self) {
        if matches!(self.focus, FocusMode::FenInput) {
            self.partial_fen.pop();
        }
    }

    pub fn pop_char(&mut self) {
        match self.focus {
            FocusMode::MoveInput => self.pop_partial_move(),
            FocusMode::FenInput => self.pop_partial_fen(),
        }
    }

    pub fn set_search_depth(&mut self, depth: u8) {
        self.engine.search_depth = depth;
    }

    pub fn confirm_action(&mut self) {
        match self.focus {
            FocusMode::MoveInput => self.make_move(&self.partial_move_to_string()),
            FocusMode::FenInput => self.apply_fen(),
        }
    }

    pub fn info_log(&mut self, log: String) {
        self.logs.push_back(format!("{log}"));
        self.log_state().scroll_down_by(1);
        self.log_state().select_previous();
    }

    pub fn set_scrolling(&mut self, scrolling: Scrolling) {
        self.scrolling.replace(scrolling);
    }

    pub fn take_scrolling(&mut self) -> Option<Scrolling> {
        self.scrolling.take()
    }

    pub fn change_focus(&mut self) {
        self.focus = match self.focus {
            FocusMode::MoveInput => FocusMode::FenInput,
            FocusMode::FenInput => FocusMode::MoveInput,
        };
    }

    fn apply_fen(&mut self) {
        let raw_fen = self.partial_fen_to_string();
        let maybe_fen = Fen::new(&raw_fen);

        match maybe_fen {
            Err(e) => self.info_log(e.to_string()),
            Ok(fen_struct) => match fen_struct.try_into_position() {
                Ok(pos) => {
                    self.engine.apply_fen(pos);
                    self.partial_move.clear();
                    self.partial_fen.clear();
                    self.move_history.clear();
                }
                Err(e) => {
                    self.info_log(e.to_string());
                }
            },
        }
    }
}
