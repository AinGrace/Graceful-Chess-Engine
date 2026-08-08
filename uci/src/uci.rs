use std::io::BufRead;
use std::io::StdinLock;
use std::io::Write;
use std::io::stdin;
use std::io::stdout;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::{fmt::Display, io::Stdout, ops::ControlFlow, thread, time::Instant};

use engine::search;
use engine::time_control::TimeControl;
use engine::{
    eval,
    search::{SearchOptions, SearchResult, TTOptions},
    tt::TT,
};
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
    tt:          Arc<Mutex<TT>>,
    pos:         Position,
    writer:      Arc<Mutex<W>>,
    reader:      R,
    stop_flag:   Arc<AtomicBool>,
    is_thinking: Arc<AtomicBool>,
}

impl Uci<Stdout, StdinLock<'static>> {
    pub fn new_stdio() -> Self {
        Self::new(stdout().into(), stdin().lock())
    }
}

impl<W: Write + Send, R: BufRead> Uci<W, R> {
    pub fn new(writer: W, reader: R) -> Self {
        Self {
            pos: Position::new(),
            tt: Arc::new(TT::default().into()),
            is_thinking: Arc::new(AtomicBool::new(false)),
            stop_flag: Arc::new(AtomicBool::new(false)),
            writer: Arc::new(writer.into()),
            reader,
        }
    }

    pub fn run(&mut self) {
        loop {
            match uci_io::read_uci_command(&mut self.reader) {
                Ok(Some(cmd)) => {
                    let res = self.apply_command(cmd);
                    if res == ControlFlow::Break(()) {
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

        info!("sent {item}");
    }

    fn handle_d(&self) {
        let pos_str = format!("{:?}", self.pos);
        Self::send(pos_str, &mut *self.acquire_writer_lock());
    }

    fn handle_eval(&self) {
        let eval = eval::static_eval(&self.pos);
        Self::send(eval, &mut &mut *self.acquire_writer_lock());
    }

    fn handle_uci(&self) {
        for item in ID_VALUES {
            Self::send(*item, &mut *self.acquire_writer_lock());
        }

        // TODO: send supported options

        Self::send("uciok", &mut *self.acquire_writer_lock());
    }

    fn handle_stop(&self) {
        if self.is_thinking.load(Ordering::Relaxed) {
            self.stop_flag.store(true, Ordering::Relaxed);
        }
    }

    fn handle_isready(&self) {
        Self::send("readyok", &mut *self.acquire_writer_lock());
    }

    fn handle_ucinewgame(&mut self) {
        // self.stop_flag.store(false, Ordering::Relaxed);
        // self.is_thinking.store(false, Ordering::Relaxed);
        self.pos = Position::new();
    }

    fn handle_go(&self, cmd: GoCmd) {
        if !self.is_thinking.load(Ordering::Relaxed) {
            match cmd {
                GoCmd::Inf => self.handle_go_inner(Some(u8::MAX), GoTimeControlKind::Infinite),
                GoCmd::Base(time) => self.handle_go_inner(Some(search::MAX_DEPTH), time),
                GoCmd::Depth(depth, time) => self.handle_go_inner(Some(depth), time),
            }
        }
    }

    fn handle_go_inner(&self, depth: Option<u8>, time_control_kind: GoTimeControlKind) {
        let mut pos = self.pos.clone();

        let stop_flag = Arc::clone(&self.stop_flag);
        let thinking_flag = Arc::clone(&self.is_thinking);

        let tt = Arc::clone(&self.tt);
        let tt_options = TTOptions::Enabled(tt);

        let write_clone = Arc::clone(&self.writer);

        let time_control = TimeControl::new(time_control_kind.into(), pos.turn());

        thread::spawn(move || {
            let search_options = SearchOptions {
                pos: &mut pos,
                search_depth: depth,
                tt_opts: tt_options,
                stop_flag: &stop_flag,
                time_control: time_control,
            };

            let mut writer_handle = write_clone.lock().expect("FATAL error on acquiring lock");

            thinking_flag.store(true, Ordering::Relaxed);

            let SearchResult { best_move, .. } =
                search::search(search_options, |s| Self::send(s, &mut *writer_handle));
            thinking_flag.store(false, Ordering::Relaxed);

            if let Some(mv) = best_move {
                Self::send(format!("bestmove {}", mv.to_uci()), &mut *writer_handle);
            } else {
                Self::send("bestmove 0000", &mut *writer_handle);
            }
        });
    }

    fn handle_position(&mut self, cmd: PositionCmd) {
        match cmd {
            PositionCmd::Base => (),

            PositionCmd::Startpos(items) => {
                self.pos.reset();

                if let Some(moves) = items {
                    moves.iter().for_each(|mv| {
                        self.pos.do_move_inner(*mv);
                    });
                }
            }

            PositionCmd::Fen(fen, items) => {
                self.pos = fen.try_into_position().expect("fen is already valid");

                if let Some(moves) = items {
                    moves.iter().for_each(|mv| {
                        self.pos.do_move_inner(*mv);
                    });
                }
            }
        }
    }

    fn handle_setoption(&self, cmd: SetOptionCmd) {}
}
