use std::collections::VecDeque;

use engine::{eval, search};
use position::position::{Position, Undo};
use ratatui::widgets::ListState;
use types::{MoveList, chess_move::Move, color::Color, piece::Piece, square::Square};

pub struct Model {
    pos: Position,

    legal_moves: MoveList,
    partial_move: Vec<char>,
    move_history: Vec<Move>,
    best_move: Option<Move>,
    eval: i32,
    undo: Vec<Undo>,

    logs: VecDeque<String>,
    log_state: ListState,

    exit: bool,
}

impl Model {
    pub fn new() -> Self {
        let mut pos = Position::new();

        let legal_moves = pos.legal_moves();
        let partial_move = Vec::with_capacity(5);
        let move_history = Vec::with_capacity(128);
        let best_move = search::negamax(&mut pos, 4, 0).1;
        let eval = i32::default();
        let undo = Vec::with_capacity(128);

        let logs = VecDeque::new();
        let log_state = ListState::default();

        let exit = false;

        Self {
            pos,
            legal_moves,
            partial_move,
            move_history,
            best_move,
            eval,
            undo,
            logs,
            log_state,
            exit,
        }
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

    pub fn should_exit(&self) -> bool {
        self.exit
    }

    pub fn info_log(&mut self, log: String) {
        self.logs.push_back(format!("[INFO] {log}"));
    }

    pub fn logs(&self) -> &[String] {
        self.logs.as_slices().0
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

    pub fn make_move<'a>(&mut self, raw_uci: &'a str) -> Option<&'a str> {
        let mv = self.legal_moves.iter().find(|mv| mv.to_uci() == raw_uci);

        let Some(mv) = mv else {
            return None;
        };

        let undo = self.pos.do_move_inner(*mv);

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

    fn update_best_move(&mut self) {
        let None = self.best_move else {
            return;
        };

        self.best_move = search::negamax(&mut self.pos, 4, 0).1;
    }
}
