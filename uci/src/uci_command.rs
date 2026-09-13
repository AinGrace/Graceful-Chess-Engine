use engine::time_control::TimeControlKind;
use position::{fen::Fen, position::Position};
use std::str::FromStr;
use types::chess_move::Move;

#[derive(Debug)]
pub enum Command {
    // Non standart UCI commands
    D,
    Eval,

    // Standart UCI commands
    Uci,
    Stop,
    Quit,
    IsReady,
    UciNewGame,
    Go(GoCmd),
    Position(PositionCmd),
    SetOption(SetOptionCmd),
}

#[derive(Debug)]
pub enum GoCmd {
    Inf,                     // infinite search, until stop command
    Base(GoTimeControlKind), // search up to MAX_DEPTH defined somewhere in the depths of engine
    Depth(u8, GoTimeControlKind),
}

#[derive(Debug)]
pub enum GoTimeControlKind {
    Infinite,

    SuddenDeath {
        w_time: u32,
        b_time: u32,
    },

    Increment {
        w_time: u32,
        w_inc: u32,

        b_time: u32,
        b_inc: u32,
    },
}

#[derive(Debug)]
pub enum PositionCmd {
    Base,
    Startpos(Option<Vec<Move>>),
    Fen(Fen, Option<Vec<Move>>),
}

#[derive(Debug)]
pub enum SetOptionCmd {
    Name(String),
    NameAndValue(String, String),
}

#[rustfmt::skip]
impl FromStr for Command {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !(0..=1).contains(&s.matches('\n').count()) {
            return Err(format!("unknown command: {s}"));
        }

        let Some(whitespace_index) = s.find(' ') else {
            // at this point s == single word
            return match s.trim() {
                "d"          => Ok(Self::D),
                "go"         => Ok(Self::Go(GoCmd::Base(GoTimeControlKind::Infinite))),
                "uci"        => Ok(Self::Uci),
                "eval"       => Ok(Self::Eval),
                "stop"       => Ok(Self::Stop),
                "quit"       => Ok(Self::Quit),
                "isready"    => Ok(Self::IsReady),
                "ucinewgame" => Ok(Self::UciNewGame),

                unknown     => Err(format!("unknown command: {unknown}")),
            };
        };

        let (first_command, rest) = s.split_at(whitespace_index);
        let (first_command, rest) = (first_command.trim(), rest.trim());

        match first_command {
            "go"        => rest.parse::<GoCmd>().map(Self::Go),
            "position"  => rest.parse::<PositionCmd>().map(Self::Position),
            "setoption" => rest.parse::<SetOptionCmd>().map(Self::SetOption),

            unknown    => Err(format!("unknown command: {unknown}")),
        }
    }
}

impl FromStr for GoCmd {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(Self::Base(GoTimeControlKind::Infinite));
        }

        if s == "infinite" {
            return Ok(Self::Inf);
        }

        let raw_commands = s.split_ascii_whitespace().collect::<Vec<&str>>();

        // at the current moment longest go command supported by the engine
        // must have less than 11 elements
        //
        // range starts from 2 to filter out
        // incorrect command "go depth"
        // with no value for depth specified
        if !(2..=10).contains(&raw_commands.len()) {
            return Err(format!("unknown command, here: {s}"));
        }

        match raw_commands[0] {
            "depth" => {
                let depth = raw_commands[1]
                    .parse::<u8>()
                    .map_err(|_err| format!("unknown value for depth"))?;

                let time_control = (&raw_commands[2..])
                    .join(" ")
                    .parse::<GoTimeControlKind>()?;

                Ok(Self::Depth(depth, time_control))
            }

            _ => {
                let time_control = raw_commands.join(" ").parse::<GoTimeControlKind>()?;
                Ok(Self::Base(time_control))
            }
        }
    }
}

impl FromStr for GoTimeControlKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let raw_cmds: Vec<&str> = s.split_ascii_whitespace().collect();

        if raw_cmds.len() % 2 != 0 {
            return Err(format!("unknown command: {s}"));
        }

        let mut w_time = None;
        let mut b_time = None;
        let mut w_inc = None;
        let mut b_inc = None;

        for pair in raw_cmds.chunks_exact(2) {
            let cmd = pair[0];
            let val = pair[1]
                .parse::<u32>()
                .map_err(|_| format!("unknown command: {s}"))?;

            match cmd {
                "wtime" => w_time = Some(val),
                "btime" => b_time = Some(val),
                "winc" => w_inc = Some(val),
                "binc" => b_inc = Some(val),
                _ => return Err(format!("unknown command: {s}")),
            }
        }

        match (w_time, b_time, w_inc, b_inc) {
            (None, None, None, None) => Ok(Self::Infinite),
            (Some(w_time), Some(b_time), None, None) => Ok(Self::SuddenDeath { w_time, b_time }),
            (Some(w_time), Some(b_time), Some(w_inc), Some(b_inc)) => Ok(Self::Increment {
                w_time,
                w_inc,
                b_time,
                b_inc,
            }),

            _ => Err(format!("invalid time controls: {s}")),
        }
    }
}

impl FromStr for PositionCmd {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cmds = s.split_ascii_whitespace().collect::<Vec<&str>>();

        match cmds.as_slice() {
            [] => Ok(Self::Base),
            ["startpos", rest @ ..] => Ok(Self::parse_startpos(rest)),
            ["fen", rest @ ..] => {
                Self::parse_fen(rest).map_or_else(|| Err(format!("unknown command: {s}")), Ok)
            }
            [unknown, ..] => Err(format!("unknown command: {unknown}")),
        }
    }
}

impl PositionCmd {
    fn parse_startpos(cmds: &[&str]) -> Self {
        let Some(moves_idx) = cmds.iter().position(|cmd| *cmd == "moves") else {
            return Self::Startpos(None);
        };

        let mut pos = Position::new();
        let mut buff = Vec::with_capacity(cmds[moves_idx + 1..].len());

        for raw_uci_move in &cmds[moves_idx + 1..cmds.len()] {
            if let Some(uci_move) = pos.parse_uci(raw_uci_move) {
                //SAFETY: parsed uci_move is quaranteed to be safe
                unsafe { pos.do_move_unchecked(uci_move) };
                buff.push(uci_move);
            } else {
                // TODO: return Err instead of silent failure
                break;
            }
        }

        if buff.is_empty() {
            Self::Startpos(None)
        } else {
            Self::Startpos(Some(buff))
        }
    }

    fn parse_fen(cmds: &[&str]) -> Option<Self> {
        // FEN string consists of 6 parts
        //
        //                     1                         2  3   4  5 6
        // rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1
        const NUM_OF_FEN_PARTS: usize = 6;

        if cmds.len() < NUM_OF_FEN_PARTS {
            return None;
        }

        let Ok(fen) = Fen::from_str(&cmds[..=6].join(" ")) else {
            return None; // TODO: return Err with short decription of what went wrong
        };

        let Ok(pos) = fen.clone().try_to_position() else {
            return None; // TODO: return Err with short decription of what went wrong
        };

        let mut buff = Vec::with_capacity(cmds.len() - (NUM_OF_FEN_PARTS + 1));

        for raw_uci_move in &cmds[NUM_OF_FEN_PARTS + 2..cmds.len()] {
            if let Some(uci_move) = pos.parse_uci(raw_uci_move) {
                buff.push(uci_move);
            } else {
                // TODO: return Err instead of silent failure
                break;
            }
        }

        if buff.is_empty() {
            Some(Self::Fen(fen, None))
        } else {
            Some(Self::Fen(fen, Some(buff)))
        }
    }
}

impl FromStr for SetOptionCmd {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

impl From<GoTimeControlKind> for TimeControlKind {
    fn from(value: GoTimeControlKind) -> Self {
        match value {
            GoTimeControlKind::SuddenDeath { w_time, b_time } => {
                TimeControlKind::SuddenDeath { w_time, b_time }
            }
            GoTimeControlKind::Increment {
                w_time,
                w_inc,
                b_time,
                b_inc,
            } => TimeControlKind::Increment {
                w_time,
                w_inc,
                b_time,
                b_inc,
            },
            GoTimeControlKind::Infinite => TimeControlKind::Infinite,
        }
    }
}
