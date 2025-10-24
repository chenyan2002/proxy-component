use clap::{Parser, ValueEnum};

mod analyze;
mod ast;
mod instrument;
mod run;
mod util;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(ValueEnum, Clone)]
pub enum Mode {
    Record,
    Replay,
}

#[derive(Parser)]
enum Commands {
    /// Instrument a component with proxying capabilities.
    Instrument(instrument::InstrumentArgs),
    /// Run a proxied component.
    Run(run::RunArgs),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Instrument(args) => instrument::run(args),
        Commands::Run(args) => run::run(args),
    }
}

impl Mode {
    fn to_str(&self) -> &str {
        match self {
            Mode::Record => "record",
            Mode::Replay => "replay",
        }
    }
}
