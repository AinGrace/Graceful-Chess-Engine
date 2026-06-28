use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use engine::{eval, search};
use position::{
    fen::Fen,
    position::{Position, Undo},
};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    widgets::ListState,
};
use types::{MoveList, chess_move::Move, color::Color, piece::Piece, square::Square};

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

#[derive(Default)]
pub struct UiAreas {
    log_area: Rect,
    chessboard_area: Rect,
    history_area: Rect,
}

impl UiAreas {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn log_area(&self) -> Rect {
        self.log_area
    }

    pub fn chessboard_area(&self) -> Rect {
        self.chessboard_area
    }

    pub fn history_area(&self) -> Rect {
        self.history_area
    }

    pub fn set_log_area(&mut self, area: Rect) {
        self.log_area = area;
    }

    pub fn set_chessboard_area(&mut self, area: Rect) {
        self.chessboard_area = area;
    }

    pub fn set_history_area(&mut self, area: Rect) {
        self.history_area = area;
    }
}

pub struct Model {
    pos: Position,

    legal_moves: MoveList,

    partial_move: Vec<char>,
    partial_fen: Vec<char>,
    move_history: Vec<Move>,
    best_move: Option<Move>,
    undo: Vec<Undo>,

    logs: VecDeque<String>,
    log_state: ListState,

    search_time: Duration,
    search_depth: u8,

    scrolling: Option<Scrolling>,

    focus: FocusMode,

    /// column and row
    mouse_hover: Option<(u16, u16)>,
    selected_square: Option<Square>,

    ui_areas: UiAreas,

    exit: bool,
}

impl Model {
    pub fn new() -> Self {
        let mut pos = Position::new();

        let legal_moves = pos.legal_moves();
        let partial_move = Vec::with_capacity(5);
        let partial_fen = Vec::with_capacity(20);
        let move_history = Vec::with_capacity(128);
        let undo = Vec::with_capacity(128);

        let logs = VecDeque::new();
        let log_state = ListState::default();

        let time_begin = Instant::now();
        let best_move = search::negamax(&mut pos, 4, 0).1;
        let time_end = Instant::now();

        let search_time = time_end.duration_since(time_begin);
        let search_depth = 4;

        let focus = FocusMode::MoveInput;

        let scrolling = None;
        let mouse_hover = None;
        let selected_square = None;

        let ui_areas = UiAreas::default();

        let exit = false;

        Self {
            pos,
            legal_moves,
            partial_move,
            partial_fen,
            move_history,
            best_move,
            undo,
            logs,
            log_state,
            search_time,
            search_depth,
            focus,
            scrolling,
            mouse_hover,
            selected_square,
            ui_areas,
            exit,
        }
    }

    pub fn ui_areas(&self) -> &UiAreas {
        &self.ui_areas
    }

    pub fn set_ui_areas(&mut self, frame: &Frame) {
        let left_middle_right =
            Layout::horizontal(Constraint::from_percentages([35, 30, 35])).split(frame.area());

        let middle_top_bottom =
            Layout::vertical([Constraint::Fill(1), Constraint::Max(3)]).split(left_middle_right[1]);

        let right_top_bottom =
            Layout::vertical(Constraint::from_percentages([60, 40])).split(left_middle_right[2]);

        todo!()
    }

    pub fn should_exit(&self) -> bool {
        self.exit
    }

    pub fn logs(&self) -> &[String] {
        self.logs.as_slices().0
    }

    pub fn is_legal_origin(&self, from: Square) -> bool {
        self.legal_moves.iter().any(|mv| mv.from() == from)
    }

    pub fn is_legal_dest(&self, to: Square) -> bool {
        self.legal_moves.iter().any(|mv| mv.to() == to)
    }

    pub fn is_legal_orig_dest_for(&self, from: Square, to: Square) -> bool {
        self.legal_moves
            .iter()
            .any(|mv| mv.from() == from && mv.to() == to)
    }

    pub fn legal_moves_of(&self, from: Square) -> Option<Vec<String>> {
        self.legal_moves
            .iter()
            .cloned()
            .filter(|mv| mv.from() == from)
            .map(|mv| mv.to_uci())
            .map(Some)
            .collect()
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
        self.best_move
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
        self.pos
            .ep_square()
            .map(|sqr| sqr.to_string())
            .unwrap_or("None".into())
    }

    pub fn move_history_iter(&self) -> impl Iterator<Item = String> {
        self.move_history.iter().map(|mv| mv.to_uci())
    }

    pub fn turn(&self) -> Color {
        self.pos.turn()
    }

    pub fn static_eval(&self) -> i32 {
        eval::static_eval(&self.pos)
    }

    pub fn peek_piece_at(&self, sqr: Square) -> Option<Piece> {
        self.pos.board().peek(sqr)
    }

    pub fn search_time(&self) -> Duration {
        self.search_time
    }

    pub fn search_depth(&self) -> u8 {
        self.search_depth
    }

    pub fn half_moves(&self) -> u32 {
        self.pos.half_moves()
    }

    pub fn full_moves(&self) -> u32 {
        self.pos.full_moves()
    }

    pub fn z_hash(&self) -> u64 {
        self.pos.zobrist_hash()
    }

    pub fn focus(&self) -> FocusMode {
        self.focus
    }

    pub fn log_state(&mut self) -> &mut ListState {
        &mut self.log_state
    }

    pub fn make_move(&mut self, raw_uci: &str) {
        let mv = self.legal_moves.iter().find(|mv| mv.to_uci() == raw_uci);

        let Some(mv) = mv else {
            return;
        };

        let undo = self.pos.do_move_inner(*mv);

        self.move_history.push(*mv);
        self.legal_moves = self.pos.legal_moves();
        self.undo.push(undo);
        self.partial_move.clear();

        self.update_best_move();
        self.info_log(format!("applied move [{raw_uci}]"));
    }

    pub fn undo_move(&mut self) {
        if let Some(undo) = self.undo.pop() {
            self.pos.undo_move(undo);
            self.legal_moves = self.pos.legal_moves();

            let unmade_move = self
                .move_history
                .pop()
                .expect("move history is not empty")
                .to_uci();

            self.update_best_move();
            self.info_log(format!("reversed the move {unmade_move}"));
        }
    }

    pub fn play_best_move(&mut self) {
        if let Some(best_move) = self.best_move.map(|mv| mv.to_uci()).take() {
            self.make_move(&best_move);
            self.info_log(format!("applied engine suggested move [{best_move}]"));
        }
    }

    pub fn update_best_move(&mut self) {
        let time_begin = Instant::now();
        self.best_move = search::negamax(&mut self.pos, self.search_depth, 0).1;
        let time_end = Instant::now();

        self.search_time = time_end.duration_since(time_begin)
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
            self.info_log(format!("pushed [{chr}] to partial_move stack"));
        }
    }

    pub fn pop_partial_move(&mut self) {
        if matches!(self.focus, FocusMode::MoveInput)
            && let Some(chr) = self.partial_move.pop()
        {
            self.info_log(format!("removed [{chr}] from partial_move stack"));
        }
    }

    pub fn push_partial_fen(&mut self, chr: char) {
        if matches!(self.focus, FocusMode::FenInput) {
            self.partial_fen.push(chr);
            self.info_log(format!("pushed [{chr}] to partial_fen stack"));
        }
    }

    pub fn pop_partial_fen(&mut self) {
        if matches!(self.focus, FocusMode::FenInput)
            && let Some(chr) = self.partial_fen.pop()
        {
            self.info_log(format!("removed [{chr}] from partial_fen stack"));
        }
    }

    pub fn pop_char(&mut self) {
        match self.focus {
            FocusMode::MoveInput => self.pop_partial_move(),
            FocusMode::FenInput => self.pop_partial_fen(),
        }
    }

    pub fn set_search_depth(&mut self, depth: u8) {
        self.search_depth = depth;
    }

    pub fn confirm_action(&mut self) {
        match self.focus {
            FocusMode::MoveInput => self.make_move(&self.partial_move_to_string()),
            FocusMode::FenInput => self.apply_fen(),
        }
    }

    pub fn ui_areas_mut(&mut self) -> &mut UiAreas {
        &mut self.ui_areas
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
            Ok(fen_struct) => {
                let maybe_pos = fen_struct.into_position();
                match maybe_pos {
                    Ok(pos) => {
                        self.pos = pos;
                        self.legal_moves = self.pos.legal_moves();
                        self.partial_move.clear();
                        self.partial_fen.clear();
                        self.move_history.clear();
                        self.undo.clear();

                        let time_begin = Instant::now();
                        let best_move = search::negamax(&mut self.pos, 4, 0).1;
                        let time_end = Instant::now();

                        self.search_time = time_end.duration_since(time_begin);
                        self.search_depth = 4;
                        self.best_move = best_move
                    }
                    Err(e) => {
                        self.info_log(e.to_string());
                    }
                }
            }
            Err(e) => self.info_log(e.to_string()),
        }
    }
}
