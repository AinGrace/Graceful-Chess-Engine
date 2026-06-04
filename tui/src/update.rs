use color_eyre::eyre::Ok;
use ratatui::crossterm::{
    self,
    event::{Event, KeyCode},
};

use crate::model::{Model, Scrolling};

#[derive(PartialEq, Debug)]
pub enum Message {
    PartialMove(char),
    RemoveChar,
    ConfirmMove,
    UndoMove,
    SearchIncrement,
    SearchDecrement,
    MouseMove { col: u16, row: u16 },
    MouseScrollDown { col: u16, row: u16 },
    MouseScrollUp { col: u16, row: u16 },
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
        Message::MouseScrollDown { col, row } => {
            model.set_scrolling(Scrolling::Down { col, row });
        }
        Message::MouseScrollUp { col, row } => {
            model.set_scrolling(Scrolling::Up { col, row });
        }
        Message::MouseMove { col, row } => {
        }
    }
}

pub fn handle_event() -> color_eyre::Result<Option<Message>> {
    match crossterm::event::read()? {
        Event::FocusGained => Ok(None),
        Event::FocusLost => Ok(None),
        Event::Key(key_event) => handle_key(key_event.code),
        Event::Mouse(mouse_event) => handle_mouse(mouse_event),
        Event::Paste(_) => Ok(None),
        Event::Resize(_, _) => Ok(None),
    }
}

fn handle_mouse(mouse_event: crossterm::event::MouseEvent) -> color_eyre::Result<Option<Message>> {
    match mouse_event.kind {
        crossterm::event::MouseEventKind::Down(_mouse_button) => Ok(None),
        crossterm::event::MouseEventKind::Up(_mouse_button) => Ok(None),
        crossterm::event::MouseEventKind::Drag(_mouse_button) => Ok(None),
        crossterm::event::MouseEventKind::Moved => Ok(Some(Message::MouseMove {
            col: mouse_event.column,
            row: mouse_event.row,
        })),
        crossterm::event::MouseEventKind::ScrollDown => {
            while crossterm::event::poll(std::time::Duration::from_millis(0))? {
                let _ = crossterm::event::read()?; // discard
            }

            Ok(Some(Message::MouseScrollDown {
                col: mouse_event.column,
                row: mouse_event.row,
            }))
        }
        crossterm::event::MouseEventKind::ScrollUp => {
            while crossterm::event::poll(std::time::Duration::from_millis(0))? {
                let _ = crossterm::event::read()?; // discard
            }

            Ok(Some(Message::MouseScrollUp {
                col: mouse_event.column,
                row: mouse_event.row,
            }))
        }
        crossterm::event::MouseEventKind::ScrollLeft => Ok(None),
        crossterm::event::MouseEventKind::ScrollRight => Ok(None),
    }
}

fn handle_key(key_code: KeyCode) -> color_eyre::Result<Option<Message>> {
    match key_code {
        KeyCode::Esc => return Ok(Some(Message::Quit)),
        KeyCode::Enter => return Ok(Some(Message::ConfirmMove)),
        KeyCode::Backspace => return Ok(Some(Message::RemoveChar)),
        KeyCode::Char(chr) => return handle_char(chr),
        KeyCode::Tab => return Ok(Some(Message::AcceptBestMove)),
        _ => Ok(None),
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
