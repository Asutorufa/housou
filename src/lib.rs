use serde_derive::Serialize;
use worker::*;

mod model;
mod provider;
use model::{Root, SiteMeta};

#[cfg(not(feature = "dev"))]
const DATA_URL: &str =
    "https://raw.githubusercontent.com/bangumi-data/bangumi-data/refs/heads/master/dist/data.json";
#[cfg(not(feature = "dev"))]
const CACHE_TTL_SECONDS: i32 = 86400; // 1 day cache

// Embed local data for dev mode
#[cfg(feature = "dev")]
const LOCAL_DATA: &str = include_str!("../data.json");

#[derive(Serialize)]
struct ConfigResponse<'a> {
    site_meta: &'a SiteMeta,
    years: Vec<i32>,
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let cache = Cache::default();
    let url = req.url()?;

    // 1. Only cache GET requests
    if req.method() == Method::Get {
        if let Ok(Some(resp)) = cache.get(url.as_str(), true).await {
            return Ok(resp);
        }
    }

    let mut resp = router(req, env).await?;

    // 2. Cache successful GET responses
    if url.path().get(0..4).unwrap_or("") == "/api" && resp.status_code() == 200 {
        // Ensure Cache-Control is set (provider already sets it, but good to ensure)
        if !resp.headers().has("Cache-Control")? {
            resp.headers_mut()
                .set("Cache-Control", "public, max-age=86400")?;
        }

        // We need to clone the response to put it in cache because put() consumes it
        // Or rather, we return the response and put a clone.
        // Cache API in worker-rs: put(key, resp)
        let _ = cache.put(url.as_str(), resp.cloned()?).await;
    }

    Ok(resp)
}

async fn fetch_data_cached() -> Result<Root> {
    // In dev mode, use embedded local data
    #[cfg(feature = "dev")]
    {
        let data: Root = serde_json::from_str(LOCAL_DATA)
            .map_err(|e| Error::RustError(format!("JSON parse error: {}", e)))?;
        return Ok(data);
    }

    // In production mode, fetch from GitHub with caching
    #[cfg(not(feature = "dev"))]
    {
        let mut init = RequestInit::new();
        init.with_method(Method::Get);

        // Set cf cache properties - only cache 200 responses
        let mut cf = CfProperties::new();
        let mut ttl_by_status = std::collections::HashMap::new();
        ttl_by_status.insert("200".to_string(), CACHE_TTL_SECONDS);
        cf.cache_ttl_by_status = Some(ttl_by_status);
        init.with_cf_properties(cf);

        let request = Request::new_with_init(DATA_URL, &init)?;
        let mut response = Fetch::Request(request).send().await?;

        if response.status_code() != 200 {
            return Err(Error::RustError(format!(
                "Failed to fetch data: status {}",
                response.status_code()
            )));
        }

        let text = response.text().await?;
        let data: Root = serde_json::from_str(&text)
            .map_err(|e| Error::RustError(format!("JSON parse error: {}", e)))?;

        Ok(data)
    }
}

async fn router(req: Request, env: Env) -> Result<Response> {
    let method = req.method();
    let path = req.path();
    let url = req.url()?;
    let query: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();

    match (method, path.as_str()) {
        (Method::Get, "/api/config") => {
            let data = fetch_data_cached().await?;
            let mut years: Vec<i32> = data
                .items
                .iter()
                .filter_map(|i| i.begin.get(0..4).and_then(|y| y.parse().ok()))
                .collect();
            years.sort_unstable();
            years.dedup();
            years.reverse(); // Newest first

            let config = ConfigResponse {
                site_meta: &data.site_meta,
                years,
            };

            let mut response = Response::from_json(&config)?;
            response
                .headers_mut()
                .set("Access-Control-Allow-Origin", "*")?;
            Ok(response)
        }
        (Method::Get, "/api/items") => {
            let data = fetch_data_cached().await?;
            let year_param = query.get("year").and_then(|y| y.parse::<i32>().ok());
            let season_param = query.get("season").map(|s| s.as_str());

            if year_param.is_none() {
                return Response::error("Bad Request: 'year' parameter is required", 400);
            }
            let target_year = year_param.unwrap();

            let items: Vec<_> = data
                .items
                .iter()
                .filter(|i| {
                    // Filter Year
                    if let Ok(y) = i.begin.get(0..4).unwrap_or("").parse::<i32>() {
                        if y != target_year {
                            return false;
                        }
                    } else {
                        return false;
                    }

                    // Filter Season
                    if let Some(target_season) = season_param {
                        if target_season.is_empty() {
                            return true;
                        } // "All Seasons"
                        if let Ok(month) = i.begin.get(5..7).unwrap_or("").parse::<u32>() {
                            let season = match month {
                                4..=6 => "Spring",
                                7..=9 => "Summer",
                                10..=12 => "Autumn",
                                1..=3 => "Winter",
                                _ => "Unknown",
                            };
                            if season != target_season {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }

                    true
                })
                .collect();

            let mut response = Response::from_json(&items)?;
            response
                .headers_mut()
                .set("Access-Control-Allow-Origin", "*")?;
            Ok(response)
        }
        (Method::Get, "/api/metadata") => {
            let tmdb_id = query.get("tmdb_id").map(|s| s.as_str());
            let title = query.get("title").map(|s| s.as_str());
            let year = query
                .get("begin")
                .and_then(|d| d.get(0..4))
                .and_then(|y| y.parse::<i32>().ok());

            provider::get_metadata(tmdb_id, title, year, &env).await
        }
        _ => Response::error("Not Found", 404),
    }
}
