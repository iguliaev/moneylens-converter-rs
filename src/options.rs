use clap::Parser;

#[derive(Parser)]
#[command(name = "moneylens-converter-rs")]
#[command(version = "0.1.0")]
#[command(about = "A command-line tool for converting financial data formats", long_about = None)]

pub struct Options {
    // Define command-line options here
    #[arg(short, long)]
    pub input: std::path::PathBuf,
    #[arg(short, long)]
    pub output: Option<std::path::PathBuf>,
    // #[arg(short, long)]
    // format: Option<String>,
}
