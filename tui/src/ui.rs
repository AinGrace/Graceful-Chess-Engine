use std::{fmt::Display, str::FromStr};

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Position},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, List, Paragraph, Wrap},
};
use types::{chess_move::Move, file::File, piece::Piece, rank::Rank, square::Square};

use crate::model::Model;

// TODO move into model
pub fn global_render(frame: &mut Frame, model: &mut Model) {
    let main_area_and_bottom_status_bar =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(2)]).split(frame.area());

    let left_middle_right = Layout::horizontal(Constraint::from_percentages([35, 30, 35]))
        .split(main_area_and_bottom_status_bar[0]);

    let middle_top_bottom =
        Layout::vertical([Constraint::Fill(1), Constraint::Max(3)]).split(left_middle_right[1]);

    let right_top_bottom =
        Layout::vertical(Constraint::from_percentages([60, 40])).split(left_middle_right[2]);

    // {
    //     let ui_areas = model.ui_areas_mut();
    //     ui_areas.set_chessboard_area(middle_top_bottom[0]);
    //     ui_areas.set_history_area(right_top_bottom[1]);
    //     ui_areas.
    // }

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

    let input_box = build_input_box(model);
    frame.render_widget(input_box, middle_top_bottom[1]);

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

fn build_input_box(model: &Model) -> Paragraph<'static> {
    let input_line = Line::from(format!("Input: {}", model.partial_move_to_string()));

    Paragraph::new(vec![input_line])
        .block(Block::bordered().border_style(Style::new().yellow()))
        .left_aligned()
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
    let mut lines = vec![];

    let partial_move_from = Square::from_str(&model.left_partial_move().unwrap_or_default());
    let partial_move_to = Square::from_str(&model.right_partial_move().unwrap_or_default());

    for rank in (0..8).rev() {
        let mut spans = Vec::new();

        // rank label
        spans.push(Span::raw(format!("{} ", rank + 1)));

        for file in 0..8 {
            let square = Square::of(File::from_u32_checked(file), Rank::from_u32_checked(rank));

            let piece = model.peek_piece_at(square);

            let symbol = piece.map(piece_to_unicode).unwrap_or(" ");

            // board colors
            let light = Color::Rgb(240, 217, 181);
            let dark = Color::Rgb(181, 136, 99);

            let piece_span = Span::raw(format!("  {symbol}  "));
            let mut square_style =
                Style::new().bg(if square.is_dark_square() { dark } else { light });

            if let Ok(move_from) = partial_move_from {
                // highlight leval squares for a piece at `move_from`
                if model.is_legal_orig_dest_for(move_from, square) {
                    square_style = square_style.bg(Color::Yellow);
                }

                if move_from == square {
                    if model.is_legal_origin(move_from) {
                        square_style = square_style.bg(Color::LightGreen);
                    } else {
                        square_style = square_style.bg(Color::LightRed);
                    }
                }
            }

            if let Ok(move_to) = partial_move_to
                && move_to == square
            {
                if model.is_legal_dest(move_to) {
                    square_style = square_style.bg(Color::Green);
                } else {
                    square_style = square_style.bg(Color::LightRed);
                }
            }

            if let Some(piece) = piece {
                let piece_color = match piece.color() {
                    types::color::Color::White => Color::Rgb(255, 255, 255),
                    types::color::Color::Black => Color::Rgb(0, 0, 0),
                };
                spans.push(piece_span.style(square_style.fg(piece_color)));
            } else {
                spans.push(piece_span.style(square_style));
            }
        }

        lines.push(Line::from(spans));
    }

    // file labels
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
