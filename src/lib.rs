use serde_derive::Serialize;
use std::sync::OnceLock;
use worker::*;

mod config;
mod model;
mod provider;
mod utils;
use model::{Item, SiteMeta, SiteMetadata, SiteType};

pub trait ResponseExt {
    fn add_cors(self, env: &Env) -> Result<Response>;
    fn add_header(self, key: &str, value: &str) -> Result<Response>;
    fn add_security_headers(self) -> Result<Response>;
}

static CORS_ALLOWED_ORIGIN: OnceLock<String> = OnceLock::new();

impl ResponseExt for Response {
    fn add_cors(mut self, env: &Env) -> Result<Response> {
        let allowed_origin = CORS_ALLOWED_ORIGIN.get_or_init(|| {
            env.var("CORS_ALLOWED_ORIGIN")
                .map(|s| s.to_string())
                .unwrap_or_else(|_| "*".to_string())
        });
        self.headers_mut()
            .set("Access-Control-Allow-Origin", allowed_origin)?;
        Ok(self)
    }

    fn add_header(mut self, key: &str, value: &str) -> Result<Response> {
        self.headers_mut().set(key, value)?;
        Ok(self)
    }

    fn add_security_headers(self) -> Result<Response> {
        self.add_header(
            "Content-Security-Policy",
            "default-src 'none'; frame-ancestors 'none';",
        )?
        .add_header("X-Content-Type-Options", "nosniff")?
        .add_header("X-Frame-Options", "DENY")
    }
}

#[derive(Serialize)]
struct ConfigResponse<'a> {
    site_meta: &'a SiteMeta,
    years: Vec<i32>,
    attribution: Attribution,
}

#[derive(Serialize)]
struct Attribution {
    tmdb: TmdbAttribution,
}

#[derive(Serialize)]
struct TmdbAttribution {
    logo_square: String,
    logo_long: String,
    logo_alt_long: String,
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let cache = Cache::open(format!("housou-cache-{}", config::CACHE_VERSION)).await;
    let url = req.url()?;

    // 1. Handle caching and routing
    let resp = if req.method() == Method::Get {
        if let Ok(Some(mut cached_resp)) = cache.get(url.as_str(), true).await {
            // Use cached response, clone to make it mutable for adding security headers
            cached_resp.cloned()?
        } else {
            // Generate new response
            let mut fresh_resp = router(req, env).await?;

            // Cache successful GET responses
            if url.path().get(0..4).unwrap_or("") == "/api" && fresh_resp.status_code() == 200 {
                if !fresh_resp.headers().has("Cache-Control")? {
                    fresh_resp = fresh_resp.add_header(
                        "Cache-Control",
                        &format!("public, max-age={}", config::CACHE_TTL_API),
                    )?;
                }
                let _ = cache.put(url.as_str(), fresh_resp.cloned()?).await;
            }
            fresh_resp
        }
    } else {
        router(req, env).await?
    };

    // Add security headers to ALL responses
    resp.add_security_headers()
}

async fn fetch_site_meta() -> Result<SiteMeta> {
    let mut sites: SiteMeta = std::collections::HashMap::new();
    let types = [
        ("info", SiteType::Info),
        ("onair", SiteType::Onair),
        ("resource", SiteType::Resource),
    ];

    let tasks = types.iter().map(|(name, stype)| {
        let url = format!("{}sites/{}.json", config::BASE_DATA_URL, name);
        let stype = stype.clone();
        async move {
            let mut data: std::collections::HashMap<String, SiteMetadata> = utils::fetch_json(&url)
                .await?
                .ok_or_else(|| Error::RustError(format!("Failed to fetch site meta: {}", url)))?;

            for meta in data.values_mut() {
                meta.type_field = Some(stype.clone());
            }
            Ok::<_, Error>(data)
        }
    });

    let results = futures::future::join_all(tasks).await;
    for result in results {
        sites.extend(result?);
    }
    Ok(sites)
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

async fn fetch_items_for_season(year: i32, season: Option<&str>) -> Result<Vec<Item>> {
    let current_year = js_sys::Date::new_0().get_full_year() as i32;
    let current_season_str = get_current_season();

    // Determine if we should use Jikan (Future) or Bangumi (Past/Present)
    // Future if:
    // 1. Year > Current Year
    // 2. Year == Current Year AND Season > Current Season
    // Note: If season is "all" (None), we treat it as future if year > current_year.
    // If year == current_year and season is "all", we technically have mixed data (past seasons + future seasons).
    // The current bangumi-data implementation for "all" fetches all months (1-12).
    // If we want to strictly follow the requirement "use Jikan for future", we might need to mix sources for the current year "all" request,
    // but the requirement says "to current time data from bangumi-data", "future from jikan".
    // A simple split is:
    // If year > current_year: Use Jikan (All seasons).
    // If year == current_year:
    //    If specific season requested:
    //       If season > current_season: Use Jikan.
    //       Else: Use Bangumi.
    //    If "all" requested:
    //       Use Bangumi (it covers 1-12, potentially empty for future months but usually pre-filled? No, bangumi-data updates periodically).
    //       Actually, bangumi-data might not have future data yet.
    //       Let's stick to the prompt: "Future upcoming data use Jikan API".
    //       Ideally for "all" in current year, we'd fetch past/current from Bangumi and future from Jikan, but that's complex to merge.
    //       Let's assume "all" for current year uses Bangumi (safe default).
    //       Only if user explicitly selects a future season or a future year we switch to Jikan.

    let is_future = if year > current_year {
        true
    } else if year == current_year {
        if let Some(s) = season {
            season_to_num(s) > season_to_num(current_season_str)
        } else {
            false
        }
    } else {
        false
    };

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

async fn router(req: Request, env: Env) -> Result<Response> {
    let method = req.method();
    let path = req.path();
    let url = req.url()?;
    let query: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();

    match (method, path.as_str()) {
        (Method::Get, "/api/config") => {
            let site_meta = fetch_site_meta().await?;

            // Fixed range of years to avoid fetching all month files just to get the list
            let current_year = js_sys::Date::new_0().get_full_year() as i32;
            // Add +1 year for future schedule
            let years: Vec<i32> = (config::START_YEAR..=current_year + 1).rev().collect();

            let config_resp = ConfigResponse {
                site_meta: &site_meta,
                years,
                attribution: Attribution {
                    tmdb: TmdbAttribution {
                        logo_square: config::TMDB_LOGO_SQUARE.to_string(),
                        logo_long: config::TMDB_LOGO_LONG.to_string(),
                        logo_alt_long: config::TMDB_LOGO_ALT_LONG.to_string(),
                    },
                },
            };

            Response::from_json(&config_resp)?
                .add_cors(&env)?
                .add_header(
                    "Cache-Control",
                    &format!("public, max-age={}", config::CACHE_TTL_CONFIG),
                )
        }
        (Method::Get, "/api/items") => {
            let year_param = query.get("year").and_then(|y| y.parse::<i32>().ok());
            let season_param = query.get("season").map(|s| s.as_str());

            if year_param.is_none() {
                return Response::error("Bad Request: 'year' parameter is required", 400);
            }
            let target_year = year_param.unwrap();

            let target_season = match season_param {
                Some("all") | None | Some("") => None,
                Some(s) => Some(s),
            };

            let items = fetch_items_for_season(target_year, target_season).await?;

            Response::from_json(&items)?.add_cors(&env)
        }
        (Method::Get, "/api/metadata") => {
            let tmdb_id = query.get("tmdb_id").map(|s| s.as_str());
            let mal_id = query.get("mal_id").map(|s| s.as_str());
            let anilist_id = query.get("anilist_id").map(|s| s.as_str());
            let title = query.get("title").map(|s| s.as_str());
            let year = query
                .get("begin")
                .and_then(|d| d.get(0..4))
                .and_then(|y| y.parse::<i32>().ok());

            let args = provider::MetadataArgs {
                tmdb_id,
                mal_id,
                anilist_id,
                title,
                year,
            };

            provider::get_metadata(args, &env).await
        }
        _ => Response::error("Not Found", 404),
    }
}
