use core::fmt;
use std::fmt::Display;

use position::position::Position;

use crate::{search::SearchResult, time_control::TimeControl};

#[derive(Clone)]
pub struct EngineStats {
    game: Option<GameStats>,
}

#[derive(Clone)]
pub struct GameStats {
    positions: Vec<PositionStats>,
}

#[derive(Clone)]
pub struct PositionStats {
    pos: Position,
    time_control: Option<TimeControl>,
    search_results: Vec<SearchResult>,
}

impl EngineStats {
    pub fn new() -> Self {
        Self { game: None }
    }

    fn game_mut(&mut self) -> Option<&mut GameStats> {
        self.game.as_mut()
    }

    pub fn last_pos_mut(&mut self) -> Option<&mut PositionStats> {
        self.game_mut().and_then(|game| game.positions.last_mut())
    }

    pub fn push_time_control(&mut self, time_control: TimeControl) {
        if let Some(pos_stats) = self.last_pos_mut() {
            pos_stats.time_control = Some(time_control);
        }
    }

    pub fn push_search_res(&mut self, search_res: SearchResult) {
        if let Some(pos_stats) = self.last_pos_mut() {
            pos_stats.search_results.push(search_res);
        }
    }

    pub fn advance_new_game(&mut self) {
        self.game.replace(GameStats { positions: vec![] });
    }

    pub fn push_new_pos(&mut self, pos: Position) {
        if let Some(game) = self.game_mut() {
            game.positions.push(PositionStats {
                pos,
                time_control: None,
                search_results: vec![],
            });
        }
    }

    pub fn is_empty(&self) -> bool {
        self.game.is_none()
    }
}

impl Display for EngineStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for game in self.game.iter() {
            writeln!(f)?;
            writeln!(f, "--- Game ---")?;
            write!(f, "{}", indent(&game.to_string(), 1))?;
        }

        Ok(())
    }
}

impl fmt::Display for GameStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "positions: {}", self.positions.len())?;

        let results = self.positions.iter().flat_map(|p| p.search_results.iter());
        if let Some(agg) = SearchAggregate::from_iter(results) {
            writeln!(f, "{agg}")?;
        }

        for (i, pos) in self.positions.iter().enumerate() {
            writeln!(f)?;
            writeln!(f, "[position {}]", i + 1)?;
            write!(f, "{}", indent(&pos.to_string(), 1))?;
        }

        Ok(())
    }
}

impl fmt::Display for PositionStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "fen:       {:<10}", self.pos.to_fen())?;
        writeln!(f, "turn:      {:<10?}", self.pos.turn())?;

        match &self.time_control {
            Some(tc) => writeln!(f, "--time control--\n{tc}")?,
            None => writeln!(f, "--time control--\nNONE")?,
        }

        writeln!(f, "search results: {}", self.search_results.len())?;
        for sr in &self.search_results {
            writeln!(f, "  {sr}")?;
        }

        if let Some(agg) = SearchAggregate::from_iter(self.search_results.iter()) {
            write!(f, "{agg}")?;
        }

        Ok(())
    }
}

impl fmt::Display for SearchResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let best = self
            .best_move
            .as_ref()
            .map(|m| m.to_uci())
            .unwrap_or_else(|| "-".to_string());

        let score_str = self.score.to_string();
        let space_idx = score_str.find(' ').expect("space always exists");
        let (prefix, value) = score_str.split_at(space_idx);

        write!(
            f,
            "depth {:>2} | score {} {:>5} | best {:>6} | nodes {:>10} | {:>12} nps | {:>6} ms",
            self.depth, prefix, value, best, self.nodes, self.nps, self.elapsed_millis,
        )
    }
}

/// Aggregate stats over a slice of `SearchResult`s.
struct SearchAggregate {
    count: usize,
    max_depth: u8,
    total_nodes: u64,
    total_elapsed_ms: u128,
    avg_nps: u64,
}

impl SearchAggregate {
    fn from_iter<'a>(results: impl Iterator<Item = &'a SearchResult>) -> Option<Self> {
        let results: Vec<&SearchResult> = results.collect();
        if results.is_empty() {
            return None;
        }

        let count = results.len();
        let max_depth = results.iter().map(|r| r.depth).max().unwrap_or(0);
        let total_nodes = results.iter().map(|r| r.nodes).sum();
        let total_elapsed_ms = results.iter().map(|r| r.elapsed_millis).sum();
        let avg_nps = results.iter().map(|r| r.nps as u128).sum::<u128>() / count as u128;

        Some(Self {
            count,
            max_depth,
            total_nodes,
            total_elapsed_ms,
            avg_nps: avg_nps as u64,
        })
    }
}

impl fmt::Display for SearchAggregate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "  [agg] searches={} max_depth={} total_nodes={} avg_nps={} total_time={}ms",
            self.count, self.max_depth, self.total_nodes, self.avg_nps, self.total_elapsed_ms,
        )
    }
}

/// Indents every line of `s` by `levels * 2` spaces.
fn indent(s: &str, levels: usize) -> String {
    let pad = "  ".repeat(levels);
    s.lines().map(|line| format!("{pad}{line}\n")).collect()
}
