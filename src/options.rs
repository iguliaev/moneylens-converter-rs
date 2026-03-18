use clap::Parser;

#[derive(Parser)]
#[command(name = "moneylens-converter-rs")]
#[command(version = "0.1.0")]
#[command(about = "A command-line tool for converting financial data formats", long_about = None)]
pub struct Options {
    #[arg(short, long)]
    pub input: std::path::PathBuf,
    #[arg(short, long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(short = 'm', long, value_parser = clap::value_parser!(u8).range(1..=12))]
    pub month: Option<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_month() {
        let opts = Options::try_parse_from([
            "moneylens-converter-rs",
            "--input",
            "input.ods",
            "--month",
            "3",
        ])
        .expect("month 3 should parse");

        assert_eq!(opts.month, Some(3));
    }

    #[test]
    fn rejects_month_out_of_range() {
        let result = Options::try_parse_from([
            "moneylens-converter-rs",
            "--input",
            "input.ods",
            "--month",
            "13",
        ]);

        assert!(result.is_err(), "month 13 should be rejected");

        assert!(result.err().unwrap().to_string().contains("1..=12"));
    }
}
