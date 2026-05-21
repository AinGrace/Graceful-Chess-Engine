use color_eyre::eyre::Ok;
use engine::eval;
use position::position::Undo;
use ratatui::crossterm::{
    self,
    event::{Event, KeyCode},
};
use types::{chess_move::Move, square::Square};

use crate::model::Model;

#[derive(PartialEq, Debug)]
pub enum Message {
    MakeMove(Move),
    UndoMove,
    Highlight(Square),
    Eval,
    Search(u8),
    Quit,
}

pub fn update(model: &mut Model, msg: Message) {
    match msg {
        Message::MakeMove(m) => {
            let undo = model.pos.do_move(m).expect("illegal move");
            model.undo.push(undo);
        }
        Message::UndoMove => {
            model.pos.undo_move(model.undo.pop().expect("no undo"));
        }
        Message::Highlight(square) => todo!(),
        Message::Eval => {
            let eval = eval::static_eval(&model.pos);
            model.eval = eval;
        }
        Message::Search(_) => todo!(),
        Message::Quit => model.quit(),
    }
}

pub fn handle_event() -> color_eyre::Result<Option<Message>> {
    if let Event::Key(key) = crossterm::event::read()? && key.is_press() {
        match key.code {
            KeyCode::Char('q') => return Ok(Some(Message::Quit)),
            _ => todo!(),
        }
    } else {
        Ok(None)
    }
}
