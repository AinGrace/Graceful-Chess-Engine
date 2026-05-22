use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, List, ListItem, Paragraph},
};
use types::{file::File, piece::Piece, rank::Rank, square::Square};

use crate::model::Model;

pub fn global_render(frame: &mut Frame, model: &Model) {
    let left_middle_right =
        Layout::horizontal(Constraint::from_percentages([35, 30, 35])).split(frame.area());

    let middle_top_bottom =
        Layout::vertical([Constraint::Fill(1), Constraint::Max(3)]).split(left_middle_right[1]);

    let right_top_bottom =
        Layout::vertical(Constraint::from_percentages([60, 40])).split(left_middle_right[2]);

    let board_box = build_chessboard(model);
    let info_box = build_info(model);
    let input_box = build_input_box(model);
    let log_box = build_logs(model);
    let history_box = build_history(model);

    frame.render_widget(board_box, middle_top_bottom[0]);
    frame.render_widget(input_box, middle_top_bottom[1]);

    frame.render_widget(info_box, right_top_bottom[0]);

    frame.render_widget(history_box, right_top_bottom[1]);
    frame.render_widget(log_box, left_middle_right[0]);
}

fn build_history(model: &Model) -> Paragraph<'_> {
    let line = Line::from_iter(model.move_history_iter());

    Paragraph::new(line).block(
        Block::bordered()
            .title("Move history")
            .border_type(BorderType::Rounded)
            .border_style(Style::new().light_cyan()),
    )
}

fn build_input_box(model: &Model) -> Paragraph<'static> {
    let input_line = Line::from(format!("Input: {}", model.collect_partial_move_to_str()));

    Paragraph::new(vec![input_line])
        .block(Block::bordered().border_style(Style::new().yellow()))
        .left_aligned()
}

fn build_info(model: &Model) -> Paragraph<'_> {
    let eval_line = Line::from(format!("Eval: {}", model.static_eval()));
    let turn_line = Line::from(format!("Turn: {}", model.turn().char()));
    let ep_line = Line::from(format!("Ep square: {}", model.ep_square_to_str()));
    let best_move_line = Line::from(format!("Best move: {}", model.best_move_to_uci()));

    Paragraph::new(vec![eval_line, turn_line, ep_line, best_move_line])
        .block(Block::bordered().title("Info"))
}

fn build_chessboard(model: &Model) -> Paragraph<'_> {
    let mut lines = vec![];

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
            let square_style = Style::new().bg(if square.is_dark_square() { dark } else { light });

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

fn build_logs(model: &Model) -> List<'_> {
    let items: Vec<ListItem> = model
        .logs()
        .iter()
        .map(|log| ListItem::new(log.as_str()))
        .collect();

    List::new(items).block(
        Block::bordered()
            .title("Logs")
            .border_type(BorderType::Rounded)
            .border_style(Style::new().blue()),
    )
}

fn piece_to_unicode(piece: Piece) -> &'static str {
    match piece {
        Piece::WPawn | Piece::BPawn => "♟",
        Piece::WKnight | Piece::BKnight => "♞",
        Piece::WBishop | Piece::BBishop => "♝",
        Piece::WRook | Piece::BRook => "♜",
        Piece::WQueen | Piece::BQueen => "♛",
        Piece::WKing | Piece::BKing => "♚",
    }
}
