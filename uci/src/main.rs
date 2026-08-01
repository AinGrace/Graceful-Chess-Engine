use std::{
    fs::OpenOptions,
    io::{self, BufRead, Write, stdin},
    mem,
};

use engine::{eval, search};
use position::{fen::Fen, position::Position};
use tracing::info;
use tracing_subscriber::fmt;

static ENGINE_NAME: &str = "Graceful";
static AUTHOR: &str = "AinGrace";

fn main() {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/home/aingrace/Graceful-Engine/bench/results/engine.log")
        .unwrap();

    fmt().with_writer(file).init();

    let stdin = stdin();
    let mut pos = Position::new();

    for line in stdin.lock().lines() {
        let line = line.expect(
            "encountered invalid UTF-8 or something else while trying to parse stdin for uci",
        );

        info!("engine got command -> {line}");

        match line.as_str() {
            "uci" => {
                send(format!("id name {ENGINE_NAME}"));
                send(format!("id author {AUTHOR}"));
                send(format!("uciok"));
            }

            "isready" => {
                send(format!("readyok"));
                info!("sent [readyok] to harness")
            }

            "ucinewgame" => {
                pos = Position::new();
            }

            "quit" => {
                break;
            }

            "eval" => {
                send(format!("{}", eval::static_eval(&pos)));
            }

            "d" => {
                send(format!("{:#?}", pos));
            }

            _ => {
                if line.starts_with("position") {
                    handle_position(&mut pos, &line);
                }

                if line.starts_with("go") {
                    handle_go(&mut pos, &line);
                }
            }
        }
    }
}

fn handle_go(pos: &mut Position, line: &str) {
    let mut commands = line.split_whitespace();

    let _ = commands.nth(0);

    for sub_cmd in commands {
        match sub_cmd {
            "infinite" => {
                info!("received unsupported command [infinite]")
            }
            _ => (),
        }
    }

    info!("starting search");
    let (_score, maybe_best_move) = search::negamax(pos, 4);
    info!("search finished");

    if let Some(best_move) = maybe_best_move {
        send(format!("info score cp {_score}"));
        send(format!("bestmove {}", best_move.to_uci()));
        info!("sent [bestmove {}] to harness", best_move.to_uci());
    } else {
        send(format!("bestmove 0000"));
        info!("sent [bestmove 0000] to harness")
    }
}

fn handle_position(pos: &mut Position, line: &str) {
    let mut commands = line.split_whitespace();

    // drop first word
    let _ = commands.next();

    let Some(second_part) = commands.next() else {
        return;
    };

    match second_part {
        "startpos" => {
            *pos = Position::new();
            handle_moves(pos, commands);
            info!("pos after the moves {pos:#?}");
        }
        "fen" => {
            let Some(fen_str) = commands.next() else {
                return;
            };

            let Ok(fen) = Fen::new(fen_str) else { return };

            if let Ok(new_pos) = fen.into_position() {
                let _old_pos = mem::replace(pos, new_pos);
            } else {
                return;
            }

            handle_moves(pos, commands);
        }

        _unknown => (),
    }
}

fn handle_moves(pos: &mut Position, mut commands: std::str::SplitWhitespace<'_>) {
    if let Some(moves_cmd) = commands.next()
        && moves_cmd == "moves"
    {
        for raw_uci in commands.into_iter() {
            let Ok(_undo) = pos.uci_move(raw_uci) else {
                return;
            };
        }
    }
}

fn send(s: String) {
    println!("{s}");
    io::stdout().flush().expect("can't flush to stdout")
}
