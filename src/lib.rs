use serde_derive::Serialize;
use worker::*;

mod config;
mod model;
mod provider;
use model::{Item, SiteMeta, SiteMetadata, SiteType};

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
            resp.headers_mut().set(
                "Cache-Control",
                &format!("public, max-age={}", config::CACHE_TTL_API),
            )?;
        }

        // We need to clone the response to put it in cache because put() consumes it
        // Or rather, we return the response and put a clone.
        // Cache API in worker-rs: put(key, resp)
        let _ = cache.put(url.as_str(), resp.cloned()?).await;
    }

    Ok(resp)
}

async fn fetch_json<T: for<'de> serde::Deserialize<'de>>(url: &str) -> Result<T> {
    let mut init = RequestInit::new();
    init.with_method(Method::Get);

    let mut cf = CfProperties::new();
    let mut ttl_by_status = std::collections::HashMap::new();
    ttl_by_status.insert("200".to_string(), config::CACHE_TTL_SECONDS);
    ttl_by_status.insert("404".to_string(), config::CACHE_TTL_404);
    cf.cache_ttl_by_status = Some(ttl_by_status);
    init.with_cf_properties(cf);

    let request = Request::new_with_init(url, &init)?;
    let mut response = Fetch::Request(request).send().await?;

    if response.status_code() != 200 {
        return Err(Error::RustError(format!(
            "Failed to fetch {}: status {}",
            url,
            response.status_code()
        )));
    }

    response.json().await
}

async fn fetch_site_meta() -> Result<SiteMeta> {
    let mut sites: SiteMeta = std::collections::HashMap::new();
    let types = [
        ("info", SiteType::Info),
        ("onair", SiteType::Onair),
        ("resource", SiteType::Resource),
    ];

    for (name, stype) in types {
        let url = format!("{}sites/{}.json", config::BASE_DATA_URL, name);
        let mut data: std::collections::HashMap<String, SiteMetadata> = fetch_json(&url).await?;
        for meta in data.values_mut() {
            meta.type_field = Some(stype.clone());
        }
        sites.extend(data);
    }
    Ok(sites)
}

async fn fetch_items_for_season(year: i32, season: Option<&str>) -> Result<Vec<Item>> {
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
            match fetch_json::<Vec<Item>>(&url).await {
                Ok(items) => Ok(items),
                Err(e) => {
                    let msg = e.to_string();
                    if msg.contains("status 404") {
                        console_log!("Month data not found (404), skipping: {}", url);
                        Ok(Vec::new())
                    } else {
                        Err(e)
                    }
                }
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
            let years: Vec<i32> = (config::START_YEAR..=current_year).rev().collect();

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

            let mut response = Response::from_json(&config_resp)?;
            response
                .headers_mut()
                .set("Access-Control-Allow-Origin", "*")?;
            response.headers_mut().set(
                "Cache-Control",
                &format!("public, max-age={}", config::CACHE_TTL_CONFIG),
            )?;
            Ok(response)
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
