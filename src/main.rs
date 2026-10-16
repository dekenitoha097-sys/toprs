mod battery;
mod console;
mod cpu;
mod disk;
mod gui;
mod memory;
mod network;
mod processes;
mod system;
mod temperature;
mod web;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    name = "toprs",
    version,
    about = "Moniteur système avec interfaces console et web",
    arg_required_else_help = true
)]
struct Cli {
    /// Lance le tableau de bord interactif dans le terminal
    #[arg(short = 'c', long = "console", conflicts_with_all = ["web", "gui"])]
    console: bool,

    /// Sert les fichiers du dossier public dans un serveur web local
    #[arg(short = 'w', long = "web", conflicts_with = "gui")]
    web: bool,

    /// Ouvre l’interface graphique native
    #[arg(short = 'g', long = "gui")]
    gui: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.console {
        console::run()?;
    } else if cli.web {
        web::run().await?;
    } else if cli.gui {
        gui::run()?;
    }

    Ok(())
}
