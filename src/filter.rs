use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, Duration, Local, NaiveTime, TimeZone};

use crate::models::Bookmark;

pub struct FilteredCounts {
    pub day: usize,
    pub week: usize,
    pub month: usize,
}

/// Write last_day, last_week, and last_month bookmark files into the same
/// directory as the main bookmarks.json.
pub fn write_filtered_files(bookmarks: &[Bookmark], dir: &Path) -> Result<FilteredCounts> {
    let day_items = filter_by_cutoff(bookmarks, &start_of_today());
    let week_items = filter_by_cutoff(bookmarks, &start_of_week());
    let month_items = filter_by_cutoff(bookmarks, &start_of_month());

    write_json(dir.join("last_day_bookmarks.json"), &day_items)?;
    write_json(dir.join("last_week_bookmarks.json"), &week_items)?;
    write_json(dir.join("last_month_bookmarks.json"), &month_items)?;

    Ok(FilteredCounts {
        day: day_items.len(),
        week: week_items.len(),
        month: month_items.len(),
    })
}

pub fn filter_by_cutoff<'a>(bookmarks: &'a [Bookmark], cutoff: &DateTime<Local>) -> Vec<&'a Bookmark> {
    bookmarks
        .iter()
        .filter(|b| {
            parse_timestamp(&b.last_update)
                .map(|t| t >= *cutoff)
                .unwrap_or(false)
        })
        .collect()
}

fn start_of_today() -> DateTime<Local> {
    let today = Local::now().date_naive();
    Local
        .from_local_datetime(&today.and_time(NaiveTime::MIN))
        .unwrap()
}

fn start_of_week() -> DateTime<Local> {
    let today = Local::now().date_naive();
    let days_since_monday = today.weekday().num_days_from_monday() as i64;
    let monday = today - Duration::days(days_since_monday);
    Local
        .from_local_datetime(&monday.and_time(NaiveTime::MIN))
        .unwrap()
}

fn start_of_month() -> DateTime<Local> {
    let today = Local::now().date_naive();
    let first = chrono::NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();
    Local
        .from_local_datetime(&first.and_time(NaiveTime::MIN))
        .unwrap()
}

fn parse_timestamp(s: &str) -> Option<DateTime<Local>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Local))
}

fn write_json(path: std::path::PathBuf, items: &[&Bookmark]) -> Result<()> {
    let json = serde_json::to_string_pretty(items).context("failed to serialize bookmarks")?;
    std::fs::write(&path, json)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_bookmark(last_update: &str) -> Bookmark {
        Bookmark {
            id: 1,
            title: "Test".to_string(),
            link: "https://example.com".to_string(),
            tags: vec![],
            collection_id: 1,
            collection: "Test".to_string(),
            created: last_update.to_string(),
            last_update: last_update.to_string(),
            excerpt: String::new(),
            kind: "link".to_string(),
        }
    }

    #[test]
    fn test_filter_includes_items_after_cutoff() {
        let cutoff = DateTime::parse_from_rfc3339("2024-06-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Local);

        let bookmarks = vec![
            make_bookmark("2024-06-02T10:00:00Z"),
            make_bookmark("2024-07-01T00:00:00Z"),
        ];

        let result = filter_by_cutoff(&bookmarks, &cutoff);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_filter_excludes_items_before_cutoff() {
        let cutoff = DateTime::parse_from_rfc3339("2024-06-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Local);

        let bookmarks = vec![
            make_bookmark("2024-05-31T23:59:59Z"),
            make_bookmark("2024-01-01T00:00:00Z"),
        ];

        let result = filter_by_cutoff(&bookmarks, &cutoff);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_filter_includes_item_at_exact_cutoff() {
        let cutoff = DateTime::parse_from_rfc3339("2024-06-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Local);

        let bookmarks = vec![make_bookmark("2024-06-01T00:00:00Z")];
        let result = filter_by_cutoff(&bookmarks, &cutoff);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_filter_excludes_items_with_empty_timestamp() {
        let cutoff = DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Local);

        let bookmarks = vec![make_bookmark("")];
        let result = filter_by_cutoff(&bookmarks, &cutoff);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_filter_mixed_items() {
        let cutoff = DateTime::parse_from_rfc3339("2024-06-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Local);

        let bookmarks = vec![
            make_bookmark("2024-05-15T10:00:00Z"), // before
            make_bookmark("2024-06-15T10:00:00Z"), // after
            make_bookmark("2024-07-01T00:00:00Z"), // after
            make_bookmark("2024-01-01T00:00:00Z"), // before
        ];

        let result = filter_by_cutoff(&bookmarks, &cutoff);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].last_update, "2024-06-15T10:00:00Z");
        assert_eq!(result[1].last_update, "2024-07-01T00:00:00Z");
    }
}
