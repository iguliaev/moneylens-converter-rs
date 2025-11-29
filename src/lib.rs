use std::error::Error;

pub mod options;
pub mod parsers;
pub mod payload;

use options::Options;
use spreadsheet_ods::{self};

pub fn run(opts: Options) -> Result<(), Box<dyn Error>> {
    let workbook = spreadsheet_ods::read_ods(&opts.input).expect("Failed to read ODS file");
    println!("Workbook has {} sheets", workbook.num_sheets());
    workbook.iter_sheets().for_each(|sheet| {
        println!("Sheet: {}", sheet.name());
    });
    let transactions = parsers::save::parse(workbook.sheet(1));

    let payload = payload::PayloadBuilder::default()
        .add_transactions(transactions)
        .build();

    let json = serde_json::to_string_pretty(&payload).expect("Failed to serialize");
    println!("{json}");

    Ok(())
}
