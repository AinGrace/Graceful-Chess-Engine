use std::fs::File;
use std::io::BufRead;
use std::io::StdinLock;
use std::io::Write;
use std::io::stdin;
use std::io::stdout;
use std::path::Component::CurDir;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::{fmt::Display, io::Stdout, ops::ControlFlow};

use engine::Engine;
use engine::time_control::TimeControl;
use position::position::Position;
use tracing::info;

use crate::uci_command::GoTimeControlKind;
use crate::{
    uci_command::{Command, GoCmd, PositionCmd, SetOptionCmd},
    uci_io,
};

static ID_VALUES: &[&str] = &["id name Graceful", "id author AinGrace"];

#[rustfmt::skip]
pub struct Uci<W: Write + Send + 'static, R: BufRead> {
    engine:      Engine,
    writer:      Arc<Mutex<W>>,
    reader:      R,
}

impl Uci<Stdout, StdinLock<'static>> {
    pub fn new_stdio() -> Self {
        Self::new(stdout().into(), stdin().lock())
    }
}

impl<W: Write + Send, R: BufRead> Uci<W, R> {
    pub fn new(writer: W, reader: R) -> Self {
        let mut engine = Engine::default();
        engine.collect_stats();
        Self {
            engine: engine,
            writer: Arc::new(writer.into()),
            reader,
        }
    }

    pub fn run(&mut self) {
        loop {
            match uci_io::read_uci_command(&mut self.reader) {
                Ok(Some(cmd)) => {
                    if self.apply_command(cmd) == ControlFlow::Break(()) {
                        write_stats_to_file();
                        break;
                    }
                }

                Ok(None) => (),
                Err(_e) => {
                    // TODO: proper error handling
                    println!("{_e}");
                }
            }
        }
    }


    #[rustfmt::skip]
    fn apply_command(&mut self, cmd: Command) -> ControlFlow<()> {
        match cmd {
            Command::D              => self.handle_d(),
            Command::Eval           => self.handle_eval(),
            Command::Uci            => self.handle_uci(),
            Command::Stop           => self.handle_stop(),
            Command::IsReady        => self.handle_isready(),
            Command::UciNewGame     => self.handle_ucinewgame(),
            Command::Go(cmd)        => self.handle_go(cmd),
            Command::Position(cmd)  => self.handle_position(cmd),
            Command::SetOption(cmd) => self.handle_setoption(cmd),

            Command::Quit           => return ControlFlow::Break(())
        }

        ControlFlow::Continue(())
    }

    fn acquire_writer_lock(&self) -> MutexGuard<'_, W> {
        self.writer.lock().expect("FATAL")
    }

    fn send(item: impl Display, w: &mut impl Write) {
        if let Err(e) = uci_io::send(&item, w) {
            info!("err while sending: {e}")
        }

        info!("-> {item}");
    }

    fn handle_d(&self) {
        let pos_str = format!("{:?}", self.engine.pos());
        Self::send(pos_str, &mut *self.acquire_writer_lock());
    }

    fn handle_eval(&self) {
        let eval = self.engine.eval();
        Self::send(eval, &mut *self.acquire_writer_lock());
    }

    fn handle_uci(&self) {
        for item in ID_VALUES {
            Self::send(*item, &mut *self.acquire_writer_lock());
        }

        // TODO: send supported options

        Self::send("uciok", &mut *self.acquire_writer_lock());
    }

    fn handle_stop(&self) {
        self.engine.stop_search();
    }

    fn handle_isready(&self) {
        Self::send("readyok", &mut *self.acquire_writer_lock());
    }

    fn handle_ucinewgame(&mut self) {
        write_stats_to_file();
        self.engine.clear();
    }

    fn handle_go(&self, cmd: GoCmd) {
        match cmd {
            GoCmd::Inf => self.handle_go_inner(Some(u8::MAX), GoTimeControlKind::Infinite),
            GoCmd::Base(time) => self.handle_go_inner(None, time),
            GoCmd::Depth(depth, time) => self.handle_go_inner(Some(depth), time),
        }
    }

    fn handle_go_inner(&self, depth: Option<u8>, time_control_kind: GoTimeControlKind) {
        let intermediate_writer = Arc::clone(&self.writer);
        let final_writer = Arc::clone(&self.writer);
        let time_control = TimeControl::new(time_control_kind.into(), self.engine.pos());

        self.engine.search(
            depth,
            time_control,
            move |res| {
                let mut writer = intermediate_writer.lock().expect("FATAL");

                Self::send(
                    format!(
                        "info depth {} score {} nodes {} nps {} time {}",
                        res.depth, res.score, res.nodes, res.nps, res.elapsed_millis
                    ),
                    &mut *writer,
                );
            },
            move |res| {
                let mut writer = final_writer.lock().expect("FATAL");
                if let Some(res) = res {
                    Self::send(format!("bestmove {}", res.to_uci()), &mut *writer)
                } else {
                    Self::send("bestmove 0000", &mut *writer);
                }
            },
        );
    }

    fn handle_position(&mut self, cmd: PositionCmd) {
        match cmd {
            PositionCmd::Base => (),

            PositionCmd::Startpos(items) => {
                let mut pos = Position::new();

                if let Some(moves) = items {
                    moves.iter().for_each(|mv| {
                        pos.do_move_inner(*mv);
                    });
                }
                self.engine.new_position(pos);
            }

            PositionCmd::Fen(fen, items) => {
                let mut pos = fen.try_to_position().expect("fen is already valid");

                if let Some(moves) = items {
                    moves.iter().for_each(|mv| {
                        pos.do_move_inner(*mv);
                    });
                }

                self.engine.new_position(pos);
            }
        }
    }

    fn handle_setoption(&self, cmd: SetOptionCmd) {}
}

fn write_stats_to_file() {
    let log_dir = option_env!("LOG_DIR");
    if let Some(stats) = engine::stats()
        && let Some(log_dir) = log_dir
    {
        let file = File::create(Path::new(log_dir).join("stats.txt"));
        file.iter().for_each(|mut f| {
            writeln!(f, "{stats}").unwrap();
        });
    }
}
