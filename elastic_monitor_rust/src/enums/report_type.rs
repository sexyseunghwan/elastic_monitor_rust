#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportType {
    Day,
    Week,
    Month,
    Year,
}

pub fn get_days(report_type: ReportType) -> i64 {
    match report_type {
        ReportType::Day => 1,
        ReportType::Week => 7,
        ReportType::Month => 30,
        ReportType::Year => 365,
    }
}
