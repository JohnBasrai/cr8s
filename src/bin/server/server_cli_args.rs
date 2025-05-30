// src/bin/server/server_cli_args.rs

use clap::Parser;

// ---

/// cr8s backend server CLI arguments
#[derive(Parser, Debug)]
#[command(
    name = "cr8s",
    version,
    about = "Backend service for the cr8s project (Rocket + SQLx + Redis)",
    author = "John Basrai <john@basrai.dev>"
)]
pub struct Cli {
    // ---
    /// Dump route-to-State<T> trait table and exit
    #[arg(long)]
    pub dump_state_traits: bool,

    /// Output the table to a Markdown file
    #[arg(long, value_name = "PATH")]
    pub output: Option<std::path::PathBuf>,

    /// Enable CI mode: fail if manage()/State<T> mismatch is found
    #[arg(long)]
    pub check: bool,
}

// ---

#[cfg(test)]
mod tests {
    // ---

    use super::Cli;
    use anyhow::{ensure, Result};
    use clap::Parser;

    // ---

    #[test]
    fn parses_dump_flag() -> Result<()> {
        // ---

        let args = Cli::parse_from(["test", "--dump-state-traits"]);
        ensure!(args.dump_state_traits);
        ensure!(!args.check);
        ensure!(args.output.is_none());
        Ok(())
    }

    // ---

    #[test]
    fn parses_output_path() -> Result<()> {
        // ---

        let args = Cli::parse_from(["test", "--output", "output.md"]);
        ensure!(args.output.unwrap().to_str().unwrap() == "output.md");
        Ok(())
    }

    // ---

    #[test]
    fn parses_check_flag() -> Result<()> {
        // ---

        let args = Cli::parse_from(["test", "--check"]);
        ensure!(args.check);
        ensure!(!args.dump_state_traits);
        Ok(())
    }

    // ---

    #[test]
    fn parses_all_together() -> Result<()> {
        // ---

        let args = Cli::parse_from([
            "test",
            "--check",
            "--dump-state-traits",
            "--output",
            "state.md",
        ]);
        ensure!(args.check);
        ensure!(args.dump_state_traits);
        ensure!(args.output.unwrap().to_str().unwrap() == "state.md");
        Ok(())
    }
}
