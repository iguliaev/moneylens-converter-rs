use std::error::Error;

pub mod options;
pub mod parsers;
pub mod payload;

use options::Options;
use spreadsheet_ods::{self};

pub fn run(opts: Options) -> Result<(), Box<dyn Error>> {
    let workbook = spreadsheet_ods::read_ods(&opts.input).expect("Failed to read ODS file");
    println!("Workbook has {} sheets", workbook.num_sheets());

    let mut payload_builder = payload::PayloadBuilder::default();

    for sheet in workbook.iter_sheets() {
        if parsers::save::can_parse(sheet) {
            payload_builder = payload_builder.add_transactions(parsers::save::parse(sheet));
        }
        if parsers::earn::can_parse(sheet) {
            payload_builder = payload_builder.add_transactions(parsers::earn::parse(sheet));
        }
        if parsers::spend::can_parse(sheet) {
            payload_builder = payload_builder.add_transactions(parsers::spend::parse(sheet));
        }
        println!("Sheet: {}", sheet.name());
    }

    let payload = payload_builder.build();

    let json = serde_json::to_string_pretty(&payload).expect("Failed to serialize");

    match opts.output {
        Some(ref path) => std::fs::write(path, &json).expect("Failed to write output file"),
        None => println!("{json}"),
    };

    Ok(())
}
