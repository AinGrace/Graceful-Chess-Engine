use color_eyre::eyre::Ok;
use ratatui::crossterm::{
    self,
    event::{Event, KeyCode},
};

use crate::model::Model;

#[derive(PartialEq, Debug)]
pub enum Message {
    PartialMove(char),
    RemoveChar,
    ConfirmMove,
    UndoMove,
    SearchIncrement,
    SearchDecrement,
    Search,
    Quit,
    AcceptBestMove,
}

pub fn update(model: &mut Model, msg: Message) {
    match msg {
        Message::PartialMove(chr) => {
            model.push_partial_move(chr);
            model.info_log(format!("pushed [{chr}] to partial_move stack"));
        }

        Message::RemoveChar => {
            if let Some(chr) = model.pop_partial_move() {
                model.info_log(format!("removed [{chr}] from partial_move stack"));
            }
        }

        Message::ConfirmMove => {
            if let Some(applied_move) = model.make_move(&model.collect_partial_move_to_str()) {
                model.info_log(format!("applied move [{applied_move}]"));
            }
        }

        Message::UndoMove => {
            if let Some(unmade_move) = model.undo_move() {
                model.info_log(format!("reversed the move {unmade_move}"));
            }
        }

        Message::AcceptBestMove => {
            if let Some(best_move) = model.play_best_move() {
                model.info_log(format!("applied engine suggested move [{best_move}]"));
            }
        }
        Message::Quit => model.quit(),
        Message::SearchIncrement => model.set_search_depth(model.search_depth().saturating_add(1)),
        Message::SearchDecrement => model.set_search_depth(model.search_depth().saturating_sub(1)),
        Message::Search => model.update_best_move(),
    }
}

pub fn handle_event() -> color_eyre::Result<Option<Message>> {
    if let Event::Key(key) = crossterm::event::read()?
        && key.is_press()
    {
        match key.code {
            KeyCode::Esc => return Ok(Some(Message::Quit)),
            KeyCode::Enter => return Ok(Some(Message::ConfirmMove)),
            KeyCode::Backspace => return Ok(Some(Message::RemoveChar)),
            KeyCode::Char(chr) => return handle_char(chr),
            KeyCode::Tab => return Ok(Some(Message::AcceptBestMove)),
            _ => Ok(None),
        }
    } else {
        Ok(None)
    }
}

fn handle_char(chr: char) -> color_eyre::Result<Option<Message>> {
    match chr {
        'u' => Ok(Some(Message::UndoMove)),
        'a'..='h' | '1'..='8' => Ok(Some(Message::PartialMove(chr))),
        '+' => Ok(Some(Message::SearchIncrement)),
        '-' => Ok(Some(Message::SearchDecrement)),
        's' => Ok(Some(Message::Search)),
        _rest => Ok(None),
    }
}
