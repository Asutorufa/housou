use time::{Date, Month};

/// Calculates the start and end timestamps (in milliseconds) for a given year and optional season.
///
/// If `season` is provided, the range covers that season.
/// If `season` is `None`, the range covers the entire year.
///
/// Seasons map to:
/// - Winter: Jan - Mar
/// - Spring: Apr - Jun
/// - Summer: Jul - Sep
/// - Autumn: Oct - Dec
pub fn get_season_timestamp_range(year: i32, season: Option<&str>) -> Result<(i64, i64), String> {
    let (start_month, end_month) = match season {
        Some("Winter") => (1, 3),
        Some("Spring") => (4, 6),
        Some("Summer") => (7, 9),
        Some("Autumn") => (10, 12),
        None => (1, 12),
        Some(other) => return Err(format!("Invalid season: {}", other)),
    };

    let start_month_enum = Month::try_from(start_month as u8)
        .map_err(|_| format!("Invalid start month: {}", start_month))?;

    let start_date = Date::from_calendar_date(year, start_month_enum, 1)
        .map_err(|e| format!("Invalid start date: {}", e))?;

    let start_ts = start_date.midnight().assume_utc().unix_timestamp() * 1000;

    // Calculate end date (start of the next month after end_month)
    let (next_year, next_month) = if end_month == 12 {
        (year + 1, 1)
    } else {
        (year, end_month + 1)
    };

    let next_month_enum = Month::try_from(next_month as u8)
        .map_err(|_| format!("Invalid next month: {}", next_month))?;

    let end_date = Date::from_calendar_date(next_year, next_month_enum, 1)
        .map_err(|e| format!("Invalid end date: {}", e))?;

    let end_ts = end_date.midnight().assume_utc().unix_timestamp() * 1000;

    Ok((start_ts, end_ts))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_season_ranges() {
        // 2024
        // Winter: Jan 1 2024 - Apr 1 2024
        let (start, end) = get_season_timestamp_range(2024, Some("Winter")).unwrap();
        assert_eq!(start, Date::from_calendar_date(2024, Month::January, 1).unwrap().midnight().assume_utc().unix_timestamp() * 1000);
        assert_eq!(end, Date::from_calendar_date(2024, Month::April, 1).unwrap().midnight().assume_utc().unix_timestamp() * 1000);

        // Spring: Apr 1 2024 - Jul 1 2024
        let (start, end) = get_season_timestamp_range(2024, Some("Spring")).unwrap();
        assert_eq!(start, Date::from_calendar_date(2024, Month::April, 1).unwrap().midnight().assume_utc().unix_timestamp() * 1000);
        assert_eq!(end, Date::from_calendar_date(2024, Month::July, 1).unwrap().midnight().assume_utc().unix_timestamp() * 1000);

        // Summer: Jul 1 2024 - Oct 1 2024
        let (start, end) = get_season_timestamp_range(2024, Some("Summer")).unwrap();
        assert_eq!(start, Date::from_calendar_date(2024, Month::July, 1).unwrap().midnight().assume_utc().unix_timestamp() * 1000);
        assert_eq!(end, Date::from_calendar_date(2024, Month::October, 1).unwrap().midnight().assume_utc().unix_timestamp() * 1000);

        // Autumn: Oct 1 2024 - Jan 1 2025
        let (start, end) = get_season_timestamp_range(2024, Some("Autumn")).unwrap();
        assert_eq!(start, Date::from_calendar_date(2024, Month::October, 1).unwrap().midnight().assume_utc().unix_timestamp() * 1000);
        assert_eq!(end, Date::from_calendar_date(2025, Month::January, 1).unwrap().midnight().assume_utc().unix_timestamp() * 1000);
    }

    #[test]
    fn test_full_year_range() {
        // 2024: Jan 1 2024 - Jan 1 2025
        let (start, end) = get_season_timestamp_range(2024, None).unwrap();
        assert_eq!(start, Date::from_calendar_date(2024, Month::January, 1).unwrap().midnight().assume_utc().unix_timestamp() * 1000);
        assert_eq!(end, Date::from_calendar_date(2025, Month::January, 1).unwrap().midnight().assume_utc().unix_timestamp() * 1000);
    }

    #[test]
    fn test_invalid_season() {
        let res = get_season_timestamp_range(2024, Some("Invalid"));
        assert!(res.is_err());
        assert_eq!(res.err(), Some("Invalid season: Invalid".to_string()));
    }
}
