use once_cell::sync::Lazy;
use spreadsheet_ods::Sheet;
use std::collections::HashMap;

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

static BANK_ACCOUNT_MAP: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    HashMap::from([
        ("X", "AmEx"),
        ("B", "Barclays"),
        ("W", "Wise Virtual"),
        ("M", "Monzo"),
        ("A", "Wise Physical"),
    ])
});
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

pub fn month_name(month: u8) -> Option<&'static str> {
    month
        .checked_sub(1)
        .and_then(|month_idx| MONTH_NAMES.get(month_idx as usize))
        .copied()
}

pub fn sheet_matches_month_selection(sheet_name: &str, selected_month: Option<u8>) -> bool {
    match selected_month {
        Some(month) => month_name(month) == Some(sheet_name),
        None => is_month(sheet_name),
    }
}

pub fn date_matches_month(date: &str, selected_month: Option<u8>) -> bool {
    match selected_month {
        Some(month) => extract_month_number(date) == Some(month),
        None => true,
    }
}

fn extract_month_number(date: &str) -> Option<u8> {
    let mut parts = date.split('-');
    let _year = parts.next()?;
    let month = parts.next()?;
    let _day = parts.next()?;

    if parts.next().is_some() {
        return None;
    }

    let month_num = month.parse::<u8>().ok()?;
    if (1..=12).contains(&month_num) {
        Some(month_num)
    } else {
        None
    }
}

pub(super) fn bank_account_symbol_to_name(symbol: Option<String>) -> String {
    match symbol {
        Some(ref name) => BANK_ACCOUNT_MAP
            .get(name.as_str())
            .unwrap_or(&"Unknown")
            .to_string(),
        None => "NatWest".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_month_name_from_number() {
        assert_eq!(month_name(1), Some("January"));
        assert_eq!(month_name(12), Some("December"));
        assert_eq!(month_name(13), None);
    }

    #[test]
    fn matches_sheet_name_against_selected_month() {
        assert!(sheet_matches_month_selection("January", Some(1)));
        assert!(!sheet_matches_month_selection("February", Some(1)));
        assert!(sheet_matches_month_selection("March", None));
        assert!(!sheet_matches_month_selection("Savings", None));
    }

    #[test]
    fn matches_iso_date_against_selected_month() {
        assert!(date_matches_month("2025-02-15", Some(2)));
        assert!(!date_matches_month("2025-03-15", Some(2)));
        assert!(date_matches_month("2025-03-15", None));
        assert!(!date_matches_month("not-a-date", Some(2)));
    }
}
