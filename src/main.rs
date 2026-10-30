use clap::{Parser, ValueEnum};

mod ast;
mod codegen;
mod instrument;
mod traits;
mod wit_util;

#[cfg(feature = "run")]
mod run;

#[cfg(test)]
mod tests {
    mod codegen;
}

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
    Fuzz,
    Dialog,
}

#[derive(Parser)]
enum Commands {
    /// Instrument a component with proxying capabilities.
    Instrument(instrument::InstrumentArgs),
    #[cfg(feature = "run")]
    /// Run a proxied component.
    Run(run::RunArgs),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Instrument(args) => instrument::run(args),
        #[cfg(feature = "run")]
        Commands::Run(args) => run::run(args),
    }
}

impl Mode {
    fn to_str(&self) -> &str {
        match self {
            Mode::Record => "record",
            Mode::Replay => "replay",
            Mode::Fuzz => "fuzz",
            Mode::Dialog => "dialog",
        }
    }
}
