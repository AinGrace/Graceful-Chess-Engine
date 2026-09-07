use std::{
    cmp,
    fmt::{self, Display},
    time::{Duration, Instant},
};

use position::position::Position;
use types::color::Color;

const AVG_MOVES_PER_GAME: u32 = 40;

/// Buffer to account for IO overhead
const LAG_BUFFER_MILLIS: u32 = 30;

/// Min possible value for the remaining time
const FLOOR_MS: u32 = 100;

#[derive(Debug)]
pub struct TimeControl {
    pub started: Instant,
    pub soft_limit: Duration,
    pub hard_limit: Duration,

    pub remaining_millis: u32,
    pub increment: u32,

    pub is_infinite: bool,
}

impl TimeControl {
    pub fn new(kind: TimeControlKind, pos: &Position) -> Self {
        let (remaining_millis, increment) = match (&kind, pos.turn()) {
            (TimeControlKind::Infinite, _) => return Self::new_infinite(),

            (TimeControlKind::SuddenDeath { w_time, .. }, Color::White) => (*w_time, 0),
            (TimeControlKind::SuddenDeath { b_time, .. }, Color::Black) => (*b_time, 0),

            (TimeControlKind::Increment { w_time, w_inc, .. }, Color::White) => (*w_time, *w_inc),
            (TimeControlKind::Increment { b_time, b_inc, .. }, Color::Black) => (*b_time, *b_inc),
        };

        let remaining_millis = remaining_millis.saturating_sub(LAG_BUFFER_MILLIS);

        let soft_limit = if remaining_millis < 1000 {
            let base = cmp::max(remaining_millis / 20, FLOOR_MS);
            Duration::from_millis((base + increment * 3 / 4) as u64)
        } else {
            let move_count = AVG_MOVES_PER_GAME.saturating_sub(pos.full_moves()).max(10);
            let base = (remaining_millis / move_count) as u64;
            Duration::from_millis(base + increment as u64 * 3 / 4)
        };

        let hard_limit = Duration::from_millis(
            (remaining_millis as u64 / 20).min(soft_limit.as_millis() as u64 * 4),
        )
        .max(soft_limit);

        let res = Self {
            started: Instant::now(),
            soft_limit,
            hard_limit,
            remaining_millis,
            increment,
            is_infinite: false,
        };

        res
    }

    pub fn new_infinite() -> Self {
        Self {
            started: Instant::now(),
            soft_limit: Duration::MAX,
            hard_limit: Duration::MAX,
            remaining_millis: u32::MAX,
            increment: u32::MAX,
            is_infinite: true,
        }
    }

    pub fn soft_expired(&self) -> bool {
        let elapsed = self.started.elapsed();
        elapsed > self.soft_limit
    }

    pub fn hard_expired(&self) -> bool {
        let elapsed = self.started.elapsed();
        elapsed > self.hard_limit
    }

    pub fn elapsed_secs_f64(&self) -> f64 {
        self.started.elapsed().as_secs_f64()
    }

    pub fn elapsed_from_start(&self) -> Duration {
        self.started.elapsed()
    }

    pub fn increase_soft_by_factor(&mut self, factor: f64) {
        if self.is_infinite {
            return;
        }

        self.soft_limit = self.soft_limit.mul_f64(factor);
    }
}

impl Display for TimeControl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Time control config: ")?;
        writeln!(
            f,
            "    total time: {:<10}",
            format!("{:?}", self.remaining_millis),
        )?;
        writeln!(f, "    increment:  {:<10}", format!("{:?}", self.increment),)?;
        writeln!(
            f,
            "    soft:       {:<10}",
            format!("{:?}", self.soft_limit),
        )?;
        writeln!(
            f,
            "    hard:       {:<10}",
            format!("{:?}", self.hard_limit),
        )?;
        writeln!(f, "---------------------")
    }
}

#[derive(Debug)]
pub enum TimeControlKind {
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

fn format_nps(nps: f64) -> String {
    const UNITS: [&str; 4] = ["", "K", "M", "G"];
    let mut value = nps;
    let mut unit_idx = 0;

    while value >= 1000.0 && unit_idx < UNITS.len() - 1 {
        value /= 1000.0;
        unit_idx += 1;
    }

    format!("{:.2}{}", value, UNITS[unit_idx])
}
