use clap::Parser;

use moneylens_converter_rs::{options::Options, run};

fn main() {
    let opts = Options::parse();
    println!("Input: {}", opts.input.display());
    println!(
        "Output: {:?}",
        opts.output.as_ref().map(|p| p.display().to_string())
    );

    if let Err(e) = run(opts) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
