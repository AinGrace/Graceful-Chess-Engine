use std::str::FromStr;

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Position},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, List, Paragraph, Wrap},
};
use types::{file::File, piece::Piece, rank::Rank, square::Square};

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
    let eval_line = Line::from(format!("Eval: {}", model.static_eval()));
    let turn_line = Line::from(format!("Turn: {}", model.turn().char()));
    let ep_line = Line::from(format!("Ep square: {}", model.ep_square_to_str()));
    let best_move_line = Line::from(format!("Best move: {}", model.best_move_to_uci()));
    let half_moves_line = Line::from(format!("Half moves: {}", model.half_moves()));
    let full_moves_line = Line::from(format!("Full moves: {}", model.full_moves()));
    let z_hash_line = Line::from(format!("Zobrist hash: {}", model.z_hash()));
    let search_depth = Line::from(format!("Search depth: {}", model.search_depth()));
    let search_time_line = Line::from(format!(
        "Search time: millis -> {} | micros -> {}",
        model.search_time().as_millis(),
        model.search_time().as_micros()
    ));

    Paragraph::new(vec![
        eval_line,
        turn_line,
        ep_line,
        best_move_line,
        search_depth,
        search_time_line,
        half_moves_line,
        full_moves_line,
        z_hash_line,
    ])
    .block(Block::bordered().title("Info"))
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
