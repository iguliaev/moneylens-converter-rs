use clap::Parser;

use moneylens_converter_rs::{options::Options, run};

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let opts = Options::parse();
    log::info!("Input: {}", opts.input.display());
    log::info!(
        "Output: {:?}",
        opts.output.as_ref().map(|p| p.display().to_string())
    );
    if let Some(month) = opts.month {
        log::info!("Month: {month}");
    }

    if let Err(e) = run(opts) {
        log::error!("{e}");
        std::process::exit(1);
    }
}
