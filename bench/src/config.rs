use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub engine: EngineConfig,
    pub tools: ToolsConfig,
    pub test: TestConfig,
    pub sprt: SprtConfig,
    pub gauntlet: GauntletConfig,
}

#[derive(Debug, Deserialize)]
pub struct EngineConfig {
    pub binary: String,

    #[serde(default)]
    pub package: String,
}

/// stores paths to tools used by benchmark
#[derive(Debug, Deserialize)]
pub struct ToolsConfig {
    pub fastchess: String,
    pub jj: String,
    pub stockfish: String,
}

#[derive(Debug, Deserialize)]
pub struct TestConfig {
    pub time_control: String,

    pub threads: u32,
    pub hash_mb: u32,

    pub concurrency: u32,
    pub games: u32,

    pub sprt_rounds: u32,
    pub gauntlet_rounds: u32,

    pub opening_book: String,
    pub opening_format: String,
    pub opening_order: String,
    pub opening_plies: u32,

    pub output_dir: String,
}

#[derive(Debug, Deserialize)]
pub struct SprtConfig {
    pub elo0: f64,
    pub elo1: f64,
    pub alpha: f64,
    pub beta: f64,
}

#[derive(Debug, Deserialize)]
pub struct GauntletConfig {
    pub stockfish_elos: Vec<u32>,
}
