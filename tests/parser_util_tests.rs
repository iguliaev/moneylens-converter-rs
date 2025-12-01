use moneylens_converter_rs::parsers;
use spreadsheet_ods;

#[test]
fn test_extract_annotation_text() {
    let workbook = spreadsheet_ods::read_ods("tests/data/spend_earn_transactions_example.ods")
        .expect("Failed to read ODS file");
    let sheet = workbook.sheet(0);
    let annotation = parsers::utils::extract_annotation(sheet, 15, 1);

    assert_eq!(
        Some("Earn Annotation".to_string()),
        annotation,
        "Expected annotation text at row 1, col 3"
    );
}
