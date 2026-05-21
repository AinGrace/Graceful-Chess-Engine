use engine::{eval, search};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use types::{file::File, piece::Piece, rank::Rank, square::Square};

use crate::model::Model;

pub fn render(frame: &mut Frame, model: &Model) {
    let left_right = Layout::horizontal(Constraint::from_percentages([50, 50])).split(frame.area());
    let board_paragraph = build_chessboard(model);
    let info_paragraph = build_info(model);

    frame.render_widget(board_paragraph, left_right[0]);
    frame.render_widget(info_paragraph, left_right[1]);
}

fn build_info(model: &Model) -> Paragraph<'_> {
    let eval_line = Line::from(format!("Eval: {}", eval::static_eval(&model.pos)));
    let turn_line = Line::from(format!("Turn: {}", model.pos.turn().char()));
    let ep_line = Line::from(format!(
        "Ep square: {}",
        model
            .pos
            .ep_square()
            .map(|sqr| sqr.to_string())
            .unwrap_or("None".into())
    ));

    let best_move_line = Line::from(format!(
        "Best move: {}",
        search::negamax(&mut model.pos.clone(), 4, 0)
            .1
            .map(|mv| mv.to_uci())
            .unwrap_or("None".into())
    ));

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

            let piece = model.pos.board().peek(square);

            let symbol = piece.map(piece_to_unicode).unwrap_or(" ");

            let mut style = Style::default();

            // board colors
            let light = Color::Rgb(240, 217, 181);
            let dark = Color::Rgb(181, 136, 99);

            style = style.bg(if square.is_dark_square() { dark } else { light });

            // selected square
            if model.selected == Some(square) {
                style = style.bg(Color::Yellow);
            }

            // legal move highlight
            // if is_legal_target(model, square) {
            //     style = style.bg(Color::Green);
            // }

            spans.push(Span::styled(format!(" {} ", symbol), style));
        }

        lines.push(Line::from(spans));
    }

    // file labels
    lines.push(Line::raw("   a  b  c  d  e  f  g  h"));
    Paragraph::new(lines).block(Block::bordered().title("Graceful chess"))
}

fn piece_to_unicode(piece: Piece) -> &'static str {
    match piece {
        Piece::BPawn => "♙",
        Piece::BKnight => "♘",
        Piece::BBishop => "♗",
        Piece::BRook => "♖",
        Piece::BQueen => "♕",
        Piece::BKing => "♔",

        Piece::WPawn => "♟",
        Piece::WKnight => "♞",
        Piece::WBishop => "♝",
        Piece::WRook => "♜",
        Piece::WQueen => "♛",
        Piece::WKing => "♚",
    }
}
