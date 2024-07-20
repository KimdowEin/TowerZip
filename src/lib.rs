use clap::{command, Parser, Subcommand};
mod unzip;
mod enzip;

#[derive(Parser)]
#[command(name = "tower_zip", author = "Ein", version = "0.1.0")]
#[command(about="一个多次解压||压缩的工具", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    Unzip(unzip::UnZipCli),

    ///建设中
    Zip (enzip::ZipCli),
}
impl Cli {
    pub fn run(self) {
        match self.command {
            Commands::Unzip(cli) => cli.run(),
            Commands::Zip(cli) => cli.run(),
        }
    }
}
