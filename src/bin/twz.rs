use clap::Parser;
use tower_zip::Cli;
fn main() {
    let cli = Cli::parse();
    cli.run();
}
