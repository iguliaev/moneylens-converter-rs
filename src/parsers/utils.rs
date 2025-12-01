use spreadsheet_ods::Sheet;

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

pub(super) fn extract_date(sheet: &Sheet, row: u32, col: u32) -> Option<String> {
    match sheet.value(row, col) {
        spreadsheet_ods::Value::Empty => None,
        spreadsheet_ods::Value::DateTime(dt) => Some(dt.date().to_string()),
        spreadsheet_ods::Value::Text(s) => Some(s.to_string()),
        _ => None,
    }
}

pub(super) fn extract_amount(sheet: &Sheet, row: u32, col: u32) -> Option<f64> {
    match sheet.value(row, col) {
        spreadsheet_ods::Value::Empty => None,
        spreadsheet_ods::Value::Number(f) => Some(*f),
        spreadsheet_ods::Value::Currency(c, _) => Some(*c),
        _ => None,
    }
}

pub(super) fn extract_text(sheet: &Sheet, row: u32, col: u32) -> Option<String> {
    match sheet.value(row, col) {
        spreadsheet_ods::Value::Empty => None,
        spreadsheet_ods::Value::Text(s) if !s.is_empty() => Some(s.to_string()),
        _ => None,
    }
}

pub fn extract_annotation(sheet: &Sheet, row: u32, col: u32) -> Option<String> {
    match sheet.annotation(row, col) {
        Some(annotation) => {
            let mut text: String = String::new();

            annotation.text().iter().for_each(|content| {
                content.extract_text(&mut text);
            });
            Some(text)
        }
        _ => None,
    }
}

pub(super) fn is_month(value: &str) -> bool {
    MONTH_NAMES.contains(&value)
}
