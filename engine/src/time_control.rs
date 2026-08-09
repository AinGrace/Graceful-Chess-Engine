use std::{
    fmt::{self, Display},
    time::{Duration, Instant},
};

use types::color::Color;

#[derive(Debug)]
pub struct TimeControl {
    pub started: Instant,
    pub soft_limit: Duration,
    pub hard_limit: Duration,

    pub total_allocation: u32, // minutes
    pub increment: u32,
}

impl TimeControl {
    pub fn new(kind: TimeControlKind, us: Color) -> Self {
        println!("{kind:?}");
        let (total_allocation, increment) = match (&kind, us) {
            (TimeControlKind::Infinite, _) => (u32::MAX, u32::MAX),

            (TimeControlKind::SuddenDeath { w_time, .. }, Color::White) => (*w_time, 0),
            (TimeControlKind::SuddenDeath { b_time, .. }, Color::Black) => (*b_time, 0),

            (TimeControlKind::Increment { w_time, w_inc, .. }, Color::White) => (*w_time, *w_inc),
            (TimeControlKind::Increment { b_time, b_inc, .. }, Color::Black) => (*b_time, *b_inc),
        };

        if total_allocation == u32::MAX && increment == u32::MAX {
            return Self {
                started: Instant::now(),
                soft_limit: Duration::MAX,
                hard_limit: Duration::MAX,
                total_allocation: u32::MAX,
                increment: u32::MAX,
            };
        }

        println!("total: {total_allocation}");

        let soft_limit = Duration::from_millis((total_allocation / 30 + increment / 2) as u64);
        let hard_limit = soft_limit.mul_f32(1.4);

        Self {
            started: Instant::now(),
            soft_limit,
            hard_limit,
            total_allocation,
            increment,
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

    pub fn increase_soft(&mut self, amount: Duration) {}

    pub fn increase_hard(&mut self, amount: Duration) {}
}

impl Display for TimeControl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Time control config: ")?;
        writeln!(
            f,
            "    total time: {:<10}",
            format!("{:?}", self.total_allocation),
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
