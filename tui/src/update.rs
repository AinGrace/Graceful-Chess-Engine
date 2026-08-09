use color_eyre::eyre::Ok;
use ratatui::crossterm::{
    self,
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
};

use crate::model::{Model, Scrolling};

#[derive(PartialEq, Debug)]
pub enum Message {
    PushChar(char),
    RemoveChar,
    Confirm,
    UndoMove,
    SearchIncrement,
    SearchDecrement,
    MouseMove { col: u16, row: u16 },
    MouseScrollDown { col: u16, row: u16 },
    MouseScrollUp { col: u16, row: u16 },
    CopyFenToClipboard,
    Search,
    Quit,
    AcceptBestMove,
    ChangeFocus,
}

// pub fn update(model: &mut Model, msg: Message) {
//     match msg {
//         Message::PushChar(chr) => {
//             model.push_char(chr);
//         }
//         Message::RemoveChar => {
//             model.pop_char();
//         }
//         Message::Confirm => model.confirm_action(),
//         Message::UndoMove => model.undo_move(),
//         Message::AcceptBestMove => model.play_best_move(),
//         Message::Quit => model.quit(),
//         Message::SearchIncrement => model.set_search_depth(model.search_depth().saturating_add(1)),
//         Message::SearchDecrement => model.set_search_depth(model.search_depth().saturating_sub(1)),
//         Message::Search => model.init_search(),
//         Message::MouseScrollDown { col, row } => {
//             model.set_scrolling(Scrolling::Down { col, row });
//         }
//         Message::MouseScrollUp { col, row } => {
//             model.set_scrolling(Scrolling::Up { col, row });
//         }
//         Message::MouseMove { .. } => {}
//         Message::ChangeFocus => model.change_focus(),
//         Message::CopyFenToClipboard => model.copy_fen_to_clipboard(),
//     }
// }

pub fn handle_event() -> color_eyre::Result<Option<Message>> {
    match crossterm::event::read()? {
        Event::FocusGained => Ok(None),
        Event::FocusLost => Ok(None),
        Event::Key(key_event) => handle_key(key_event),
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

fn handle_key(key_event: KeyEvent) -> color_eyre::Result<Option<Message>> {
    match key_event.code {
        KeyCode::Esc => return Ok(Some(Message::Quit)),
        KeyCode::Enter => return Ok(Some(Message::Confirm)),
        KeyCode::Backspace => return Ok(Some(Message::RemoveChar)),
        KeyCode::Char(chr) if key_event.modifiers.is_empty() => return handle_plain_char(chr),
        KeyCode::Char(chr) => return handle_modified_char(key_event.modifiers, chr),
        KeyCode::Tab => return Ok(Some(Message::ChangeFocus)),
        _ => Ok(None),
    }
}

fn handle_modified_char(modifiers: KeyModifiers, chr: char) -> color_eyre::Result<Option<Message>> {
    if modifiers == KeyModifiers::ALT {
        return match chr {
            's' => Ok(Some(Message::Search)),
            'e' => Ok(Some(Message::AcceptBestMove)),
            'u' => Ok(Some(Message::UndoMove)),
            'n' => Ok(Some(Message::SearchIncrement)),
            'p' => Ok(Some(Message::SearchDecrement)),
            'c' => Ok(Some(Message::CopyFenToClipboard)),
            _rest => Ok(None),
        };
    }

    if modifiers == KeyModifiers::SHIFT {
        return Ok(Some(Message::PushChar(chr.to_ascii_uppercase())));
    }

    Ok(None)
}

fn handle_plain_char(chr: char) -> color_eyre::Result<Option<Message>> {
    match chr {
        chr if chr.is_ascii() => Ok(Some(Message::PushChar(chr))),
        _rest => Ok(None),
    }
}
