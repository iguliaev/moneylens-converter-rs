use clap::Parser;
use spreadsheet_ods::{self};

mod parsers;
mod payload;

#[derive(Parser)]
#[command(name = "moneylens-converter-rs")]
#[command(version = "0.1.0")]
#[command(about = "A command-line tool for converting financial data formats", long_about = None)]



struct Options {
    // Define command-line options here
    #[arg(short, long)]
    input: std::path::PathBuf,
    #[arg(short, long)]
    output: Option<std::path::PathBuf>,
    // #[arg(short, long)]
    // format: Option<String>,
}


// fn parse_sheet(sheet: &Sheet) {
//     for (row, _d) in sheet.iter_rows((0,0)..(10,5)) {
//         println!("Row: {},{}", row.0, row.1);
//         println!("{:#?}", sheet.value(row.0, row.1));
//         // println!("{:#?}", sheet.annotation(row.0, row.1));
//     }
// }

#[derive(Debug, Default)]
struct Row {
    date: String,
    amount: f64,
    category: String,
    notes: Option<String>,
}

// fn extract_date(sheet: &Sheet, row: u32, col: u32) -> Option<String> {
//     match sheet.value(row, col) {
//         spreadsheet_ods::Value::Empty => None,
//         spreadsheet_ods::Value::DateTime(dt) => Some(dt.date().to_string()),
//         spreadsheet_ods::Value::Text(s) => Some(s.to_string()),
//         _ => None,
//     }
// }

// fn extract_amount(sheet: &Sheet, row: u32, col: u32) -> Option<f64> {
//     match sheet.value(row, col) {
//         spreadsheet_ods::Value::Empty => None,
//         spreadsheet_ods::Value::Number(f) => Some(*f),
//         spreadsheet_ods::Value::Currency(c, _) => Some(*c),
//         _ => None,
//     }
// }

// fn extract_category(sheet: &Sheet, row: u32, col: u32) -> Option<String> {
//     match sheet.value(row, col) {
//         spreadsheet_ods::Value::Empty => None,
//         spreadsheet_ods::Value::Text(s) => Some(s.to_string()),
//         _ => None,
//     }
// }

// fn extract_notes(sheet: &Sheet, row: u32, col: u32) -> Option<String> {
//     match sheet.value(row, col) {
//         spreadsheet_ods::Value::Empty => None,
//         spreadsheet_ods::Value::Text(s) => Some(s.to_string()),
//         _ => None,
//     }
// }

// fn parse_savings_sheet(sheet: &Sheet) {
//     assert!(sheet.name() == "Savings", "Expected sheet named 'Savings'");

//     //
//     //     Date	|Amount	   |Category |	Notes
//     // 2025-01-06	|150.00	   |Other	 |
//     // 2025-01-26	|151.71	   |Other	| Interest
//     // 2025-02-06	|150.00	   |Other	| Interest
//     // 2025-02-09	|1,000.00. |Other	|

//     const EMPTY_DATE_THRESHOLD: usize = 5;
//     let mut empty_date_count = 0;

//     for row_idx in 2..1000 {
//         let date = extract_date(sheet, row_idx, 2);
//         if date.is_none() {
//             empty_date_count += 1;
//             if empty_date_count >= EMPTY_DATE_THRESHOLD {
//                 break;
//             }
//             continue;
//        }
//        let amount = extract_amount(sheet, row_idx, 3).unwrap_or(0.0);
//        let category = extract_category(sheet, row_idx, 4).unwrap_or("".to_string());
//        let notes = extract_notes(sheet, row_idx, 5).unwrap_or("".to_string());
       
//        println!("Date: {}, Amount: {}, Category: {}, Notes: {}", date.unwrap(), amount, category, notes);
//     }
// }

fn main() {
    let opts = Options::parse();
    println!("Input: {}", opts.input.display());
    println!("Output: {:?}", opts.output.as_ref().map(|p| p.display().to_string()));
    // println!("Format: {:?}", opts.format);

    let workbook = spreadsheet_ods::read_ods(&opts.input).expect("Failed to read ODS file");
    println!("Workbook has {} sheets", workbook.num_sheets());
    workbook.iter_sheets().for_each(|sheet| {
        println!("Sheet: {}", sheet.name());
    });
    parsers::save::parse(workbook.sheet(1));
}
