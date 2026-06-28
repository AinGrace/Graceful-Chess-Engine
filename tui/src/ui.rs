use std::{fmt::Display, str::FromStr};

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Position},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, List, Paragraph, Wrap},
};
use types::{file::File, piece::Piece, rank::Rank, square::Square};

use crate::model::{FocusMode, Model};

pub fn global_render(frame: &mut Frame, model: &mut Model) {
    let left_middle_right =
        Layout::horizontal(Constraint::from_percentages([30, 40, 30])).split(frame.area());

    let middle_top_bottom =
        Layout::vertical([Constraint::Fill(1), Constraint::Max(3), Constraint::Max(3)])
            .split(left_middle_right[1]);

    let right_top_bottom =
        Layout::vertical(Constraint::from_percentages([60, 40])).split(left_middle_right[2]);

    let log_box = build_logs(model.logs().to_vec());
    if let Some(scroll) = model.take_scrolling()
        && left_middle_right[0].contains(Position::new(scroll.col(), scroll.row()))
    {
        match scroll {
            crate::model::Scrolling::Up { .. } => model.log_state().select_previous(),
            crate::model::Scrolling::Down { .. } => model.log_state().select_next(),
        }
    }

    frame.render_stateful_widget(log_box, left_middle_right[0], model.log_state());

    let board_box = build_chessboard(model);
    frame.render_widget(board_box, middle_top_bottom[0]);

    let info_box = build_info(model);
    frame.render_widget(info_box, right_top_bottom[0]);

    let (move_input_box, fen_input_box) = build_input_boxes(model);
    frame.render_widget(move_input_box, middle_top_bottom[1]);
    frame.render_widget(fen_input_box, middle_top_bottom[2]);

    let history_box = build_history(model);
    frame.render_widget(history_box, right_top_bottom[1]);
}

fn build_history(model: &Model) -> Paragraph<'_> {
    let line = Line::from_iter(
        model
            .move_history_iter()
            .enumerate()
            .map(|(i, mv)| format!("{i}. {mv} ")),
    );

    Paragraph::new(line)
        .block(
            Block::bordered()
                .title("Move history")
                .border_type(BorderType::Rounded)
                .border_style(Style::new().light_cyan()),
        )
        .wrap(Wrap { trim: true })
}

fn build_input_boxes(model: &Model) -> (Paragraph<'static>, Paragraph<'static>) {
    let mut move_input_block = Block::bordered()
        .title("Move")
        .border_style(Style::new().yellow());

    let mut fen_input_block = Block::bordered()
        .title("Fen")
        .border_style(Style::new().yellow());

    match model.focus() {
        FocusMode::MoveInput => {
            fen_input_block = fen_input_block.border_style(Style::new().dark_gray())
        }
        FocusMode::FenInput => {
            move_input_block = move_input_block.border_style(Style::new().dark_gray())
        }
    };

    let move_input = Paragraph::new(model.partial_move_to_string())
        .block(move_input_block)
        .left_aligned();
    
    let fen_input = Paragraph::new(model.partial_fen_to_string())
        .block(fen_input_block)
        .left_aligned();

    (move_input, fen_input)
}

fn build_info(model: &Model) -> Paragraph<'_> {
    let line = |label: &str, val: &dyn Display| Line::from(format!("{label}: {val}"));

    let legal_moves = model
        .left_partial_move()
        .and_then(|raw| Square::from_str(&raw).ok())
        .and_then(|sq| model.legal_moves_of(sq))
        .map(|moves| {
            moves
                .iter()
                .map(|mv| mv.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_else(|| "None".to_string());

    let search_time = model.search_time();

    Paragraph::new(vec![
        line("Eval", &model.static_eval()),
        line("Turn", &model.turn().char()),
        line("Ep square", &model.ep_square_to_str()),
        line("Best move", &model.best_move_to_uci()),
        line("Search depth", &model.search_depth()),
        Line::from(format!(
            "Search time: millis → {} | micros → {}",
            search_time.as_millis(),
            search_time.as_micros(),
        )),
        line("Half moves", &model.half_moves()),
        line("Full moves", &model.full_moves()),
        line("Zobrist hash", &model.z_hash()),
        Line::from(format!("Legal moves for selection: {legal_moves}")),
    ])
    .block(Block::bordered().title("Info"))
    .wrap(Wrap { trim: true })
}

fn build_chessboard(model: &mut Model) -> Paragraph<'_> {
    const LIGHT: Color = Color::Rgb(240, 217, 181);
    const DARK: Color = Color::Rgb(181, 136, 99);

    let partial_move_from = Square::from_str(&model.left_partial_move().unwrap_or_default()).ok();
    let partial_move_to = Square::from_str(&model.right_partial_move().unwrap_or_default()).ok();

    let mut lines: Vec<Line> = (0..8)
        .rev()
        .map(|rank| {
            let mut spans = vec![Span::raw(format!("{} ", rank + 1))];

            for file in 0..8 {
                let square = Square::of(File::from_u32_checked(file), Rank::from_u32_checked(rank));
                let piece = model.peek_piece_at(square);
                let symbol = piece.map(piece_to_unicode).unwrap_or(" ");

                let bg = match () {
                    _ if partial_move_to == Some(square) => {
                        if model.is_legal_dest(square) {
                            Color::Green
                        } else {
                            Color::LightRed
                        }
                    }
                    _ if partial_move_from == Some(square) => {
                        if model.is_legal_origin(square) {
                            Color::LightGreen
                        } else {
                            Color::LightRed
                        }
                    }
                    _ if partial_move_from
                        .is_some_and(|from| model.is_legal_orig_dest_for(from, square)) =>
                    {
                        Color::Yellow
                    }
                    _ => {
                        if square.is_dark_square() {
                            DARK
                        } else {
                            LIGHT
                        }
                    }
                };

                let fg = piece.map(|p| match p.color() {
                    types::color::Color::White => Color::Rgb(255, 255, 255),
                    types::color::Color::Black => Color::Rgb(0, 0, 0),
                });

                let style = match fg {
                    Some(fg) => Style::new().bg(bg).fg(fg),
                    None => Style::new().bg(bg),
                };

                spans.push(Span::styled(format!("  {symbol}  "), style));
            }

            Line::from(spans)
        })
        .collect();

    lines.push(Line::raw("    A    B    C    D    E    F    G    H"));

    Paragraph::new(lines).block(
        Block::bordered()
            .title("Graceful chess")
            .border_type(BorderType::Rounded)
            .border_style(Style::new().green()),
    )
}

fn build_logs(logs: Vec<String>) -> List<'static> {
    List::new(logs)
        .block(
            Block::bordered()
                .title("Logs")
                .border_type(BorderType::Rounded)
                .border_style(Style::new().blue()),
        )
        .highlight_symbol("=> ")
}

pub fn piece_to_unicode(piece: Piece) -> &'static str {
    match piece {
        Piece::WPawn | Piece::BPawn => "♟",
        Piece::WKnight | Piece::BKnight => "♞",
        Piece::WBishop | Piece::BBishop => "♝",
        Piece::WRook | Piece::BRook => "♜",
        Piece::WQueen | Piece::BQueen => "♛",
        Piece::WKing | Piece::BKing => "♚",
    }
}
