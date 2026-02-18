use crate::config;
use crate::model::Item;
use crate::provider;
use crate::utils;
use worker::*;

pub async fn fetch_items(year: i32, season: Option<&str>) -> Result<Vec<Item>> {
    let current_year = js_sys::Date::new_0().get_full_year() as i32;
    let current_season_str = get_current_season();

    // Determine if we should use Jikan (Future) or Bangumi (Past/Present)
    let is_future = is_future_season(year, season, current_year, current_season_str);

    if is_future {
        if let Some(s) = season {
            let jikan_season = match s {
                "Autumn" => "fall",
                _ => s,
            };
            return provider::jikan::fetch_season(year, &jikan_season.to_lowercase()).await;
        } else {
            // Fetch all 4 seasons from Jikan
            let seasons = ["winter", "spring", "summer", "fall"]; // Jikan uses "fall" instead of "autumn"
            let tasks = seasons
                .iter()
                .map(|s| provider::jikan::fetch_season(year, s));

            let results: Result<Vec<Vec<Item>>, _> =
                futures::future::join_all(tasks).await.into_iter().collect();
            return Ok(results?.into_iter().flatten().collect());
        }
    }

    // Bangumi Data logic
    let months = match season {
        Some("Winter") => vec![1, 2, 3],
        Some("Spring") => vec![4, 5, 6],
        Some("Summer") => vec![7, 8, 9],
        Some("Autumn") => vec![10, 11, 12],
        _ => (1..=12).collect(),
    };

    let mut futures = Vec::new();

    for &month in &months {
        let url = format!("{}items/{}/{:02}.json", config::BASE_DATA_URL, year, month);
        futures.push(async move {
            match utils::fetch_json::<Vec<Item>>(&url).await {
                Ok(Some(items)) => Ok(items),
                Ok(None) => {
                    console_log!("Month data not found (404), skipping: {}", url);
                    Ok(Vec::new())
                }
                Err(e) => Err(e),
            }
        });
    }

    let results: Result<Vec<Vec<Item>>, _> = futures::future::join_all(futures)
        .await
        .into_iter()
        .collect();
    Ok(results?.into_iter().flatten().collect())
}

fn get_season_from_month(month: u32) -> &'static str {
    match month {
        1..=3 => "Winter",
        4..=6 => "Spring",
        7..=9 => "Summer",
        10..=12 => "Autumn",
        _ => unreachable!("Month should be between 1 and 12"),
    }
}

fn get_current_season() -> &'static str {
    let month = js_sys::Date::new_0().get_month() + 1;
    get_season_from_month(month)
}

fn season_to_num(season: &str) -> i32 {
    match season {
        "Winter" => 1,
        "Spring" => 2,
        "Summer" => 3,
        "Autumn" => 4,
        _ => 0,
    }
}

fn is_future_season(
    year: i32,
    season: Option<&str>,
    current_year: i32,
    current_season: &str,
) -> bool {
    year > current_year
        || (year == current_year
            && season.is_some_and(|s| season_to_num(s) > season_to_num(current_season)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_season_to_num() {
        assert_eq!(season_to_num("Winter"), 1);
        assert_eq!(season_to_num("Spring"), 2);
        assert_eq!(season_to_num("Summer"), 3);
        assert_eq!(season_to_num("Autumn"), 4);
        assert_eq!(season_to_num("Invalid"), 0);
    }

    #[test]
    fn test_is_future_season() {
        // Future year
        assert!(is_future_season(2025, Some("Winter"), 2024, "Winter"));
        assert!(is_future_season(2025, None, 2024, "Winter"));

        // Past year
        assert!(!is_future_season(2023, Some("Winter"), 2024, "Winter"));
        assert!(!is_future_season(2023, None, 2024, "Winter"));

        // Current year, future season
        assert!(is_future_season(2024, Some("Spring"), 2024, "Winter"));
        assert!(is_future_season(2024, Some("Summer"), 2024, "Winter"));
        assert!(is_future_season(2024, Some("Autumn"), 2024, "Winter"));

        // Current year, current season
        assert!(!is_future_season(2024, Some("Winter"), 2024, "Winter"));

        // Current year, past season
        assert!(!is_future_season(2024, Some("Winter"), 2024, "Spring"));

        // Current year, all seasons (None) -> defaults to false (Bangumi data)
        assert!(!is_future_season(2024, None, 2024, "Winter"));
    }

    #[test]
    fn test_get_season_from_month() {
        assert_eq!(get_season_from_month(1), "Winter");
        assert_eq!(get_season_from_month(3), "Winter");
        assert_eq!(get_season_from_month(4), "Spring");
        assert_eq!(get_season_from_month(6), "Spring");
        assert_eq!(get_season_from_month(7), "Summer");
        assert_eq!(get_season_from_month(9), "Summer");
        assert_eq!(get_season_from_month(10), "Autumn");
        assert_eq!(get_season_from_month(12), "Autumn");
    }

    #[test]
    #[should_panic(expected = "Month should be between 1 and 12")]
    fn test_get_season_from_month_panic() {
        get_season_from_month(13);
    }
}
