use position::position::{Position, Undo};
use ratatui::widgets::Widget;
use types::{MoveList, chess_move::Move, square::Square};

pub struct Model {
    pub pos: Position,
    pub legal_moves: MoveList,
    pub selected: Option<Square>,
    pub move_history: Vec<Move>,
    pub exit: bool,

    pub best_move: Option<Move>,
    pub eval: i32,

    pub undo: Vec<Undo>,
}

impl Model {
    pub fn new() -> Self {
        let pos = Position::new();
        let legal_moves = pos.legal_moves();
        let selected = None;
        let move_history = Vec::with_capacity(128);
        let exit = false;
        let best_move = None;
        let eval = i32::default();
        let undo = Vec::with_capacity(128);

        Self {
            pos,
            legal_moves,
            selected,
            move_history,
            exit,
            best_move,
            eval,
            undo
        }
    }

    pub fn quit(&mut self) {
        self.exit = true
    }
}

impl Widget for Model {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized {
        todo!()
    }
}
