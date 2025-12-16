use chrono::Timelike;

use crate::common::*;

use crate::enums::report_range::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportType {
    Day,
    Week,
    Month,
    Year,
}

impl ReportType {
    pub fn range(&self) -> ReportRange {
        /* Stawndard -> UTC time */
        let now: DateTime<Utc> = Utc::now();

        match self {
            ReportType::Day => ReportRange {
                from: now - ChronoDuration::days(1),
                to: now,
            },
            ReportType::Week => ReportRange {
                from: now - ChronoDuration::days(7),
                to: now,
            },
            ReportType::Month => ReportRange {
                from: now - Months::new(1),
                to: now,
            },
            ReportType::Year => ReportRange {
                from: now - Months::new(12),
                to: now,
            },
        }
    }

    pub fn get_name(&self) -> String {
        match self {
            ReportType::Day => "Daily",
            ReportType::Week => "Weekly",
            ReportType::Month => "Monthly",
            ReportType::Year => "Yearly",
        }
        .to_string()
    }
}
