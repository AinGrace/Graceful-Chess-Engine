use position::{fen::Fen, position::Position};
use std::str::FromStr;
use tracing::info;
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
    Inf,  // infinite search, until stop command
    Base, // search up to MAX_DEPTH defined somewhere in the depths of engine
    Subcommands(Vec<GoSubCmd>),
}

#[derive(Debug)]
pub enum GoSubCmd {
    Depth(u8),
    Winc(u32),
    Binc(u32),
    Wtime(u32),
    Btime(u32),
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
            return Ok(Self::Base);
        }

        if s == "infinite" {
            return Ok(Self::Inf);
        }

        let mut res = Vec::new();
        let raw_commands = s.split_ascii_whitespace().collect::<Vec<&str>>();

        if raw_commands.len() == 1 {
            return Ok(Self::Base); // fallback value
        }

        for i in (0..raw_commands.len()).step_by(2) {
            let sub_cmd = raw_commands[i..].join(" ").parse::<GoSubCmd>()?;
            res.push(sub_cmd);
        }

        Ok(Self::Subcommands(res))
    }
}

impl FromStr for GoSubCmd {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse_cmd_value_pair<T, F>(
            raw_cmds: &[&str],
            raw_cmd: &str,
            value_idx: usize,
            f: F,
        ) -> Result<GoSubCmd, String>
        where
            T: FromStr,
            F: Fn(T) -> GoSubCmd,
        {
            let Some(val) = raw_cmds.get(value_idx).map(|v| v.trim()) else {
                return Err(format!("unknown command: missing value for {raw_cmd}"));
            };

            val.parse::<T>()
                .map_or_else(|_| Err(format!("unknown command: {val}")), |val| Ok(f(val)))
        }

        let raw_cmds: Vec<&str> = s.split_ascii_whitespace().collect();

        if let Some(i) = (0..raw_cmds.len()).next() {
            return match raw_cmds[i] {
                raw_cmd @ "depth" => parse_cmd_value_pair(&raw_cmds, raw_cmd, i + 1, Self::Depth),

                raw_cmd @ "winc" => parse_cmd_value_pair(&raw_cmds, raw_cmd, i + 1, Self::Winc),

                raw_cmd @ "binc" => parse_cmd_value_pair(&raw_cmds, raw_cmd, i + 1, Self::Binc),

                raw_cmd @ "wtime" => parse_cmd_value_pair(&raw_cmds, raw_cmd, i + 1, Self::Wtime),

                raw_cmd @ "btime" => parse_cmd_value_pair(&raw_cmds, raw_cmd, i + 1, Self::Btime),

                _unknown => Err(format!("unknown command: {s}")),
            };
        }

        Err("unknown command: invalid input for go".into())
    }
}

impl FromStr for PositionCmd {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cmds = s.split_ascii_whitespace().collect::<Vec<&str>>();

        if cmds.is_empty() {
            return Ok(Self::Base);
        }

        let sub_cmd = cmds[0];
        let sub_cmd = sub_cmd.trim();

        match sub_cmd {
            "startpos" => Ok(Self::parse_startpos(&cmds[1..])),
            "fen" => {
                Self::parse_fen(&cmds[1..]).map_or_else(|| Err(format!("unknown command: {s}")), Ok)
            }
            unknown => Err(format!("unknown command: {unknown}")),
        }
    }
}

impl PositionCmd {
    fn parse_startpos(cmds: &[&str]) -> Self {
        let Some(moves_idx) = cmds.iter().position(|cmd| *cmd == "moves") else {
            return Self::Startpos(None);
        };

        let mut pos = Position::new();
        let mut buff = Vec::with_capacity(cmds.len() - (moves_idx - 1));

        info!("raw cmds: {cmds:?}");

        for raw_uci_move in &cmds[moves_idx + 1..cmds.len()] {
            info!("trying to parse {raw_uci_move}");
            if let Some(uci_move) = pos.parse_uci(raw_uci_move) {
                pos.do_move_inner(uci_move);
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

        let Ok(pos) = fen.clone().try_into_position() else {
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
