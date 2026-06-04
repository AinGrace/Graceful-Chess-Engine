use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use engine::{eval, search};
use position::position::{Position, Undo};
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
    move_history: Vec<Move>,
    best_move: Option<Move>,
    undo: Vec<Undo>,

    logs: VecDeque<String>,
    log_state: ListState,

    search_time: Duration,
    search_depth: u8,

    scrolling: Option<Scrolling>,

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
        let move_history = Vec::with_capacity(128);
        let undo = Vec::with_capacity(128);

        let logs = VecDeque::new();
        let log_state = ListState::default();

        let time_begin = Instant::now();
        let best_move = search::negamax(&mut pos, 4, 0).1;
        let time_end = Instant::now();

        let search_time = time_end.duration_since(time_begin);
        let search_depth = 4;

        let scrolling = None;
        let mouse_hover = None;
        let selected_square = None;

        let ui_areas = UiAreas::default();

        let exit = false;

        Self {
            pos,
            legal_moves,
            partial_move,
            move_history,
            best_move,
            undo,
            logs,
            log_state,
            search_time,
            search_depth,
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

    pub fn left_partial_move(&self) -> Option<String> {
        if self.partial_move.len() < 2 {
            None
        } else {
            Some(self.collect_partial_move_to_str())
        }
    }

    pub fn right_partial_move(&self) -> Option<String> {
        if self.partial_move.len() < 4 {
            None
        } else {
            Some(self.collect_partial_move_to_str()[1..3].to_string())
        }
    }

    pub fn best_move_to_uci(&self) -> String {
        self.best_move
            .map(|mv| mv.to_uci())
            .unwrap_or("None".into())
    }

    pub fn collect_partial_move_to_str(&self) -> String {
        self.partial_move.iter().collect()
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

    pub fn log_state(&mut self) -> &mut ListState {
        &mut self.log_state
    }

    pub fn make_move<'a>(&mut self, raw_uci: &'a str) -> Option<&'a str> {
        let mv = self.legal_moves.iter().find(|mv| mv.to_uci() == raw_uci);

        let Some(mv) = mv else {
            return None;
        };

        let undo = self.pos.do_move_inner(*mv);

        self.move_history.push(*mv);
        self.legal_moves = self.pos.legal_moves();
        self.undo.push(undo);
        self.partial_move.clear();

        self.update_best_move();

        Some(raw_uci)
    }

    pub fn undo_move(&mut self) -> Option<String> {
        if let Some(undo) = self.undo.pop() {
            self.pos.undo_move(undo);
            self.legal_moves = self.pos.legal_moves();

            let unmade_move = self
                .move_history
                .pop()
                .expect("move history is not empty")
                .to_uci();

            self.update_best_move();

            Some(unmade_move)
        } else {
            None
        }
    }

    pub fn play_best_move(&mut self) -> Option<String> {
        if let Some(best_move) = self.best_move.map(|mv| mv.to_uci()).take() {
            let _ = self.make_move(&best_move);
            Some(best_move)
        } else {
            None
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

    pub fn push_partial_move(&mut self, chr: char) {
        self.partial_move.push(chr);
    }

    pub fn pop_partial_move(&mut self) -> Option<char> {
        self.partial_move.pop()
    }

    pub fn set_search_depth(&mut self, depth: u8) {
        self.search_depth = depth;
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
}
