use clap::{Parser, ValueEnum};

mod ast;
mod codegen;
mod instrument;
mod traits;
mod util;

#[cfg(feature = "run")]
mod run;

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
    /// Generate necessary trait implementation from a wit-bindgen binding
    Generate(codegen::GenerateArgs),
    #[cfg(feature = "run")]
    /// Run a proxied component.
    Run(run::RunArgs),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Instrument(args) => instrument::run(args),
        Commands::Generate(args) => args.generate(),
        #[cfg(feature = "run")]
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
