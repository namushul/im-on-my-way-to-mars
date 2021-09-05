use std::time::Duration;

const SECONDS_IN_A_MINUTE: u64 = 60;
const SECONDS_IN_A_HOUR: u64 = SECONDS_IN_A_MINUTE * 60;
const SECONDS_IN_A_DAY: u64 = SECONDS_IN_A_HOUR * 24;
const SECONDS_IN_A_WEEK: u64 = SECONDS_IN_A_DAY * 7;
const SECONDS_IN_A_MONTH: u64 = SECONDS_IN_A_DAY * 30;
const SECONDS_IN_A_YEAR: u64 = SECONDS_IN_A_DAY * 365;

/// Formats the object as human friendly `String`
pub trait Humanize {
    /// Formats the object as human friendly `String`
    fn humanize(&self) -> String;
}

impl Humanize for Duration {
    fn humanize(&self) -> String {
        let years = self.as_secs() / SECONDS_IN_A_YEAR;
        if years == 1 { return "a year".to_owned(); }
        if years > 0 { return format!("{} years", years); }

        let months = self.as_secs() / SECONDS_IN_A_MONTH;
        if months == 1 { return "a month".to_owned(); }
        if months > 0 { return format!("{} months", months); }

        let weeks = self.as_secs() / SECONDS_IN_A_WEEK;
        if weeks == 1 { return "a week".to_owned(); }
        if weeks > 0 { return format!("{} weeks", weeks); }

        let days = self.as_secs() / SECONDS_IN_A_DAY;
        if days == 1 { return "a day".to_owned(); }
        if days > 0 { return format!("{} days", days); }

        let hours = self.as_secs() / SECONDS_IN_A_HOUR;
        if hours == 1 { return "an hour".to_owned(); }
        if hours > 0 { return format!("{} hours", hours); }

        let minutes = self.as_secs() / SECONDS_IN_A_MINUTE;
        if minutes == 1 { return "a minute".to_owned(); }
        if minutes > 0 { return format!("{} minutes", minutes); }

        if self.as_secs() == 1 { return "a second".to_owned(); }
        if self.as_secs() > 0 { return format!("{} seconds", self.as_secs()); }
        return "less than a second".to_owned();
    }
}

#[test]
fn test_humanize() {
    assert_eq!(Duration::from_micros(17).humanize(), "less than a second");
    assert_eq!(Duration::from_millis(17).humanize(), "less than a second");
    assert_eq!(Duration::from_secs(0).humanize(), "less than a second");
    assert_eq!(Duration::from_secs(1).humanize(), "a second");
    assert_eq!(Duration::from_secs(17).humanize(), "17 seconds");
    assert_eq!(Duration::from_secs(59).humanize(), "59 seconds");
    assert_eq!(Duration::from_secs(60).humanize(), "a minute");
    assert_eq!(Duration::from_secs(61).humanize(), "a minute");
    assert_eq!(Duration::from_secs(119).humanize(), "a minute");
    assert_eq!(Duration::from_secs(120).humanize(), "2 minutes");
    assert_eq!(Duration::from_secs(17 * SECONDS_IN_A_MINUTE).humanize(), "17 minutes");
    assert_eq!(Duration::from_secs(1 * SECONDS_IN_A_HOUR).humanize(), "an hour");
    assert_eq!(Duration::from_secs(17 * SECONDS_IN_A_HOUR).humanize(), "17 hours");
    assert_eq!(Duration::from_secs(1 * SECONDS_IN_A_DAY).humanize(), "a day");
    assert_eq!(Duration::from_secs(5 * SECONDS_IN_A_DAY).humanize(), "5 days");
    assert_eq!(Duration::from_secs(1 * SECONDS_IN_A_WEEK).humanize(), "a week");
    assert_eq!(Duration::from_secs(3 * SECONDS_IN_A_WEEK).humanize(), "3 weeks");
    assert_eq!(Duration::from_secs(1 * SECONDS_IN_A_MONTH).humanize(), "a month");
    assert_eq!(Duration::from_secs(3 * SECONDS_IN_A_MONTH).humanize(), "3 months");
    assert_eq!(Duration::from_secs(1 * SECONDS_IN_A_YEAR).humanize(), "a year");
    assert_eq!(Duration::from_secs(3 * SECONDS_IN_A_YEAR).humanize(), "3 years");
}