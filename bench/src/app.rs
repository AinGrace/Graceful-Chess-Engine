use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio, abort, exit},
};

use anyhow::{Context, Result, bail};
use chrono::Local;

use crate::{Workspace, config::Config};

#[derive(Debug)]
pub struct App {
    config: Config,

    /// benchmark/
    benchmark_dir: PathBuf,

    /// Graceful-engine/
    repo_root: PathBuf,

    /// benchmark/results/
    results_dir: PathBuf,

    /// benchmark/fastchess
    fastchess: PathBuf,

    /// jj executable
    jj: String,

    /// stockfish executable
    stockfish: PathBuf,

    keep_workspaces: bool,
}

impl App {
    pub fn new(config_path: &Path, keep_workspaces: bool) -> Result<Self> {
        let config_path = if config_path.is_absolute() {
            config_path.to_path_buf()
        } else {
            std::env::current_dir()?.join(config_path)
        };

        let config_path = config_path
            .canonicalize()
            .with_context(|| format!("failed to resolve config: {}", config_path.display()))?;

        let benchmark_dir = config_path
            .parent()
            .context("config file has no parent directory")?
            .to_path_buf();

        let config_text = fs::read_to_string(&config_path)
            .with_context(|| format!("failed to read {}", config_path.display()))?;

        let config: Config = toml::from_str(&config_text).context("failed to parse config.toml")?;

        // benchmark/ is directly inside the engine repository.
        let repo_root = benchmark_dir
            .parent()
            .context("benchmark directory has no parent")?
            .to_path_buf();

        let results_dir = resolve_path(&benchmark_dir, &config.test.output_dir);

        let fastchess = resolve_tool(&benchmark_dir, &config.tools.fastchess);
        let stockfish = resolve_tool(&benchmark_dir, &config.tools.stockfish);

        Ok(Self {
            jj: config.tools.jj.clone(),
            stockfish,

            config,

            benchmark_dir,
            repo_root,
            results_dir,
            fastchess,

            keep_workspaces,
        })
    }

    pub fn check_requirements(&self) -> Result<()> {
        println!("Checking requirements...");

        self.check_command(&self.jj).context("jj is required")?;

        self.check_command("cargo").context("cargo is required")?;

        self.check_executable(&self.fastchess)
            .context("fastchess is required")?;

        self.check_executable(&self.stockfish)
            .context("stockfish is required")?;

        self.check_jj_repo()?;

        Ok(())
    }

    fn check_command(&self, command: &str) -> Result<()> {
        let status = Command::new(command)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        match status {
            Ok(status) if status.success() => Ok(()),

            Ok(status) => {
                bail!("`{command} --version` exited with {status}");
            }

            Err(error) => {
                bail!("command `{command}` not found: {error}");
            }
        }
    }

    fn check_executable(&self, path: &Path) -> Result<()> {
        if !path.is_file() {
            bail!("executable does not exist: {}", path.display());
        }

        Ok(())
    }

    fn check_jj_repo(&self) -> Result<()> {
        let status = Command::new(&self.jj)
            .current_dir(&self.repo_root)
            .arg("root")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .context("failed to execute jj")?;

        if !status.success() {
            bail!("{} is not a JJ repository", self.repo_root.display());
        }

        Ok(())
    }

    fn create_workspace(&self, revision: &str, name: &str) -> Result<Workspace> {
        let workspace_root = self.repo_root.join(".bench-workspaces").join(name);

        if workspace_root.exists() {
            println!(
                "workspace already exists: {}, remove? [Y/n]: ",
                workspace_root.display()
            );

            let mut buf = String::with_capacity(2);

            io::stdin()
                .read_line(&mut buf)
                .context("failed to read user input")?;

            if buf.len() > 2 {
                bail!(
                    "invalid input {:?}",
                    buf.chars().map(|chr| chr as u8).collect::<Vec<u8>>()
                );
            }

            if buf.len() == 1 {
                let a = buf.pop().unwrap();
                if a as u8 != 10 {
                    bail!("invalid input {a} | numeric -> {}", a as u8)
                }
            }

            if buf.len() == 2 && buf != "n\x0A" {
                bail!(
                    "invalid input {:?}",
                    buf.chars().map(|chr| chr as u8).collect::<Vec<u8>>()
                );
            } else if buf.len() == 2 && buf == "n\x0A" {
                exit(1)
            }

            let existing_workspace = Workspace {
                name: name.into(),
                path: workspace_root.clone(),
            };

            self.forget_workspace(&existing_workspace)?
        }

        if let Some(parent) = workspace_root.parent() {
            fs::create_dir_all(parent)?;
        }

        println!("\n==> Creating workspace `{name}` at `{revision}`");

        run(Command::new(&self.jj)
            .current_dir(&self.repo_root)
            .args(["workspace", "add", "--name", name, "--revision", revision])
            .arg(&workspace_root))?;

        Ok(Workspace {
            name: name.to_owned(),
            path: workspace_root,
        })
    }

    fn forget_workspace(&self, workspace: &Workspace) -> Result<()> {
        println!("Cleaning workspace `{}`", workspace.name);

        let status = Command::new(&self.jj)
            .current_dir(&self.repo_root)
            .args(["workspace", "forget", &workspace.name])
            .status()
            .context("failed to execute jj workspace forget")?;

        if !status.success() {
            bail!("jj workspace forget failed for `{}`", workspace.name);
        }

        if workspace.path.exists() {
            fs::remove_dir_all(&workspace.path)
                .with_context(|| format!("failed to remove {}", workspace.path.display()))?;
        }

        Ok(())
    }

    fn build_engine(&self, workspace: &Workspace, revision: &str) -> Result<PathBuf> {
        println!("\n==> Building `{revision}`");

        let mut command = Command::new("cargo");

        command
            .current_dir(&workspace.path)
            .args(["build", "-q", "--release"]);

        if !self.config.engine.package.is_empty() {
            command.args(["--package", &self.config.engine.package]);
        }

        run(&mut command)?;

        let binary = workspace
            .path
            .join("target")
            .join("release")
            .join(&self.config.engine.binary);

        if !binary.is_file() {
            bail!("built engine was not found: {}", binary.display());
        }

        Ok(binary)
    }

    pub fn run_sprt(&self, new_revision: &str, base_revision: &str) -> Result<()> {
        let new_name = sanitize_name(new_revision);
        let base_name = sanitize_name(base_revision);

        let new_workspace_name = format!("bench-new-{new_name}");

        let base_workspace_name = format!("bench-base-{base_name}");

        let new_workspace = self.create_workspace(new_revision, &new_workspace_name)?;

        let base_workspace = match self.create_workspace(base_revision, &base_workspace_name) {
            Ok(workspace) => workspace,

            Err(error) => {
                let _ = self.forget_workspace(&new_workspace);
                return Err(error);
            }
        };

        let result =
            self.run_sprt_inner(new_revision, base_revision, &new_workspace, &base_workspace);

        if !self.keep_workspaces {
            let _ = self.forget_workspace(&new_workspace);
            let _ = self.forget_workspace(&base_workspace);
        } else {
            println!(
                "\nTemporary workspaces retained under {}",
                self.repo_root.join(".bench-workspaces").display()
            );
        }

        result
    }

    fn run_sprt_inner(
        &self,
        new_revision: &str,
        base_revision: &str,
        new_workspace: &Workspace,
        base_workspace: &Workspace,
    ) -> Result<()> {
        let new_engine = self.build_engine(new_workspace, new_revision)?;

        let base_engine = self.build_engine(base_workspace, base_revision)?;

        let timestamp = Local::now().format("%Y%m%d-%H%M%S");

        let result_dir = self.results_dir.join(format!(
            "sprt-{}-vs-{}-{timestamp}",
            sanitize_name(base_revision),
            sanitize_name(new_revision),
        ));

        fs::create_dir_all(&result_dir)?;

        let pgn = result_dir.join("result.pgn");
        let log = result_dir.join("fastchess.log");

        println!();
        println!("========================================");
        println!("             SPRT TEST");
        println!("========================================");
        println!("New:          {new_revision}");
        println!("Base:         {base_revision}");
        println!("Time control: {}", self.config.test.time_control);
        println!(
            "SPRT:         H0={} H1={}",
            self.config.sprt.elo0, self.config.sprt.elo1
        );
        println!();

        let mut args = Vec::<String>::new();

        // New engine.
        args.extend([
            "-engine".into(),
            format!("name={new_revision}"),
            format!("cmd={}", new_engine.display()),
        ]);

        // Base engine.
        args.extend([
            "-engine".into(),
            format!("name={base_revision}"),
            format!("cmd={}", base_engine.display()),
        ]);

        self.add_common_fastchess_args(&mut args);

        args.extend([
            "-rounds".into(),
            self.config.test.sprt_rounds.to_string(),
            "-games".into(),
            self.config.test.games.to_string(),
            "-repeat".into(),
            "-concurrency".into(),
            self.config.test.concurrency.to_string(),
            "-recover".into(),
            "-sprt".into(),
            format!("elo0={}", self.config.sprt.elo0),
            format!("elo1={}", self.config.sprt.elo1),
            format!("alpha={}", self.config.sprt.alpha),
            format!("beta={}", self.config.sprt.beta),
            "-pgnout".into(),
            pgn.display().to_string(),
        ]);

        self.run_fastchess(&args, &log)?;

        self.write_sprt_info(&result_dir, new_revision, base_revision)?;

        println!("\nResults: {}", result_dir.display());

        Ok(())
    }

    pub fn run_gauntlet(&self, revision: &str) -> Result<()> {
        let name = sanitize_name(revision);
        let workspace_name = format!("bench-{name}");

        let workspace = self.create_workspace(revision, &workspace_name)?;

        let result = self.run_gauntlet_inner(revision, &workspace);

        if !self.keep_workspaces {
            let _ = self.forget_workspace(&workspace);
        }

        result
    }

    fn run_gauntlet_inner(&self, revision: &str, workspace: &Workspace) -> Result<()> {
        let engine = self.build_engine(workspace, revision)?;

        let timestamp = Local::now().format("%Y%m%d-%H%M%S");

        let result_dir = self.results_dir.join(format!(
            "gauntlet-{revision}-{timestamp}",
            revision = sanitize_name(revision),
        ));

        fs::create_dir_all(&result_dir)?;

        println!();
        println!("========================================");
        println!("         STOCKFISH GAUNTLET");
        println!("========================================");
        println!("Engine:       {revision}");
        println!("Time control: {}", self.config.test.time_control);
        println!(
            "Opponents:    {}",
            self.config.gauntlet.stockfish_elos.len()
        );
        println!();

        for elo in &self.config.gauntlet.stockfish_elos {
            self.run_gauntlet_match(revision, &engine, *elo, &result_dir)?;
        }

        println!();
        println!("Gauntlet complete: {}", result_dir.display());

        Ok(())
    }

    fn run_gauntlet_match(
        &self,
        revision: &str,
        engine: &Path,
        elo: u32,
        result_dir: &Path,
    ) -> Result<()> {
        println!("\n==> {} vs Stockfish {}", revision, elo);

        let pgn = result_dir.join(format!("sf-{elo}.pgn"));

        let log = result_dir.join(format!("sf-{elo}.log"));

        let mut args = Vec::<String>::new();

        args.extend([
            "-engine".into(),
            format!("name={revision}"),
            format!("cmd={}", engine.display()),
            "-engine".into(),
            format!("name=SF-{elo}"),
            format!("cmd={}", self.stockfish.to_str().unwrap()),
            "option.UCI_LimitStrength=true".into(),
            format!("option.UCI_Elo={elo}"),
        ]);

        self.add_common_fastchess_args(&mut args);

        args.extend([
            "-rounds".into(),
            self.config.test.gauntlet_rounds.to_string(),
            "-games".into(),
            self.config.test.games.to_string(),
            "-repeat".into(),
            "-concurrency".into(),
            self.config.test.concurrency.to_string(),
            "-recover".into(),
            "-pgnout".into(),
            format!("file={}", pgn.display().to_string()),
        ]);

        self.run_fastchess(&args, &log)
    }

    fn add_common_fastchess_args(&self, args: &mut Vec<String>) {
        args.extend([
            "-each".into(),
            "proto=uci".into(),
            format!("tc={}", self.config.test.time_control),
            format!("option.Threads={}", self.config.test.threads),
            format!("option.Hash={}", self.config.test.hash_mb),
        ]);

        if !self.config.test.opening_book.is_empty() {
            let book = resolve_path(&self.benchmark_dir, &self.config.test.opening_book);

            args.extend([
                "-openings".into(),
                format!("file={}", book.display()),
                format!("format={}", self.config.test.opening_format),
                format!("order={}", self.config.test.opening_order),
                format!("plies={}", self.config.test.opening_plies),
            ]);
        }
    }

    fn run_fastchess(&self, args: &[String], log_path: &Path) -> Result<()> {
        println!("\n$ {} {}", self.fastchess.display(), shell_join(args));

        let output = Command::new(&self.fastchess)
            .args(args)
            .output()
            .context("failed to execute fastchess")?;

        fs::write(log_path, &output.stdout)?;

        print!("{}", String::from_utf8_lossy(&output.stdout));
        eprint!("{}", String::from_utf8_lossy(&output.stderr));

        if !output.status.success() {
            bail!("fastchess exited with {}", output.status);
        }

        Ok(())
    }

    fn write_sprt_info(
        &self,
        result_dir: &Path,
        new_revision: &str,
        base_revision: &str,
    ) -> Result<()> {
        let text = format!(
            r#"New:          {new_revision}
Base:         {base_revision}

Time control: {time_control}
Threads:      {threads}
Hash:         {hash} MB
Concurrency:  {concurrency}
Games/round:  {games}
Max rounds:   {rounds}

SPRT:
  elo0:       {elo0}
  elo1:       {elo1}
  alpha:      {alpha}
  beta:       {beta}

Opening book: {opening}
"#,
            time_control = self.config.test.time_control,
            threads = self.config.test.threads,
            hash = self.config.test.hash_mb,
            concurrency = self.config.test.concurrency,
            games = self.config.test.games,
            rounds = self.config.test.sprt_rounds,
            elo0 = self.config.sprt.elo0,
            elo1 = self.config.sprt.elo1,
            alpha = self.config.sprt.alpha,
            beta = self.config.sprt.beta,
            opening = self.config.test.opening_book,
        );

        fs::write(result_dir.join("test-info.txt"), text)?;

        Ok(())
    }
}

fn resolve_path(base: &Path, value: &str) -> PathBuf {
    let path = Path::new(value);

    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

fn resolve_tool(base: &Path, value: &str) -> PathBuf {
    if value.starts_with("./") {
        base.join(&value[2..])
    } else {
        PathBuf::from(value)
    }
}

fn sanitize_name(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn run(command: &mut Command) -> Result<ExitStatus> {
    let status = command
        .status()
        .context("failed to execute external command")?;

    if !status.success() {
        bail!("command exited with {status}");
    }

    Ok(status)
}

fn shell_join(args: &[String]) -> String {
    args.iter()
        .map(|arg| {
            if arg
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "-_./=:+".contains(c))
            {
                arg.clone()
            } else {
                format!("'{}'", arg.replace('\'', "'\\''"))
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
