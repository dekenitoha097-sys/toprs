mod battery;
mod console;
mod cpu;
mod disk;
mod memory;
mod network;
mod processes;
mod system;
mod temperature;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    name = "toprs",
    version,
    about = "Moniteur système Linux en terminal",
    arg_required_else_help = true
)]
struct Cli {
    /// Lance le tableau de bord interactif dans le terminal
    #[arg(short = 'c', long = "console")]
    console: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.console {
        console::run()?;
    }

    Ok(())
}
