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
            let results = futures::future::join_all(tasks).await;
            let mut all_items = Vec::new();
            for items in results.into_iter().flatten() {
                all_items.extend(items);
            }
            return Ok(all_items);
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

    let mut all_items = Vec::new();
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

    let results = futures::future::join_all(futures).await;
    for result in results {
        all_items.extend(result?);
    }
    Ok(all_items)
}

fn get_current_season() -> &'static str {
    let month = js_sys::Date::new_0().get_month() + 1;
    match month {
        1..=3 => "Winter",
        4..=6 => "Spring",
        7..=9 => "Summer",
        10..=12 => "Autumn",
        _ => "Winter",
    }
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
    if year > current_year {
        true
    } else if year == current_year {
        if let Some(s) = season {
            season_to_num(s) > season_to_num(current_season)
        } else {
            false
        }
    } else {
        false
    }
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
}
