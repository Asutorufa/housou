use serde_derive::Serialize;
use worker::*;

use crate::db::Database;
use crate::model::{SiteMeta, SiteMetadata, SiteType};
use crate::{ResponseExt, auth, config, db, provider, utils};

#[derive(Serialize)]
pub struct ConfigResponse {
    pub site_meta: SiteMeta,
    pub years: Vec<i32>,
    pub attribution: Attribution,
    pub auth_enabled: bool,
    pub github_enabled: bool,
    pub telegram_bot_name: Option<String>,
}

#[derive(serde_derive::Deserialize)]
struct ItemsQuery {
    year: Option<i32>,
    season: Option<String>,
}

#[derive(serde_derive::Deserialize)]
struct MetadataQuery {
    tmdb_id: Option<String>,
    mal_id: Option<String>,
    anilist_id: Option<String>,
    title: Option<String>,
    begin: Option<String>,
}

#[derive(Serialize)]
pub struct Attribution {
    pub tmdb: TmdbAttribution,
}

#[derive(Serialize)]
pub struct TmdbAttribution {
    pub logo_square: String,
    pub logo_long: String,
    pub logo_alt_long: String,
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
                .ok_or_else(|| Error::RustError(format!("Failed to fetch site meta: {url}")))?;

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

pub async fn handle_config(_req: Request, env: Env) -> Result<Response> {
    let auth_enabled = env.d1("DB").is_ok();
    let github_enabled = env.var("GITHUB_CLIENT_ID").is_ok();
    let telegram_bot_name = env.var("TELEGRAM_BOT_NAME").ok().map(|v| v.to_string());
    let site_meta = fetch_site_meta().await?;

    // Fixed range of years to avoid fetching all month files just to get the list
    let current_year = js_sys::Date::new_0().get_full_year() as i32;
    // Add +1 year for future schedule
    let years: Vec<i32> = (config::START_YEAR..=current_year + 1).rev().collect();

    let config_resp = ConfigResponse {
        site_meta,
        years,
        attribution: Attribution {
            tmdb: TmdbAttribution {
                logo_square: config::TMDB_LOGO_SQUARE.to_string(),
                logo_long: config::TMDB_LOGO_LONG.to_string(),
                logo_alt_long: config::TMDB_LOGO_ALT_LONG.to_string(),
            },
        },
        auth_enabled,
        github_enabled,
        telegram_bot_name,
    };

    Response::from_json(&config_resp)?.add_header(
        "Cache-Control",
        &format!("public, max-age={}", config::CACHE_TTL_CONFIG),
    )
}

pub async fn handle_items(req: Request, _env: Env) -> Result<Response> {
    let url = req.url()?;
    let query_str = url.query().unwrap_or("");
    let query: ItemsQuery = serde_urlencoded::from_str(query_str)
        .map_err(|e| Error::RustError(format!("Invalid query: {}", e)))?;

    let target_year = match query.year {
        Some(y) => y,
        None => return Response::error("Bad Request: 'year' parameter is required", 400),
    };

    let target_season = match query.season.as_deref() {
        Some("all") | None | Some("") => None,
        Some(s) => Some(s),
    };

    let items = provider::season::fetch_items(target_year, target_season).await?;
    Response::from_json(&items)
}

pub async fn handle_user_status(req: Request, env: Env) -> Result<Response> {
    let (user, _) = match auth::get_auth(&req, &env).await? {
        Some(u) => u,
        None => return Response::error("Unauthorized", 401),
    };

    let url = req.url()?;
    let query_str = url.query().unwrap_or("");
    let query: ItemsQuery = serde_urlencoded::from_str(query_str).unwrap_or(ItemsQuery {
        year: None,
        season: None,
    });

    match auth::get_db(&env) {
        Ok(db) => {
            let user_items = if let Some(year) = query.year
                && let Some(season) = query.season.as_deref()
                && season != "all"
            {
                // Calculate timestamp range for the season
                let (start_month, end_month) = match season {
                    "Winter" => (1, 3),
                    "Spring" => (4, 6),
                    "Summer" => (7, 9),
                    "Autumn" => (10, 12),
                    _ => (1, 12),
                };

                // Approximate timestamps (seconds)
                // We use 00:00:00 of the 1st day of start_month
                // to 23:59:59 of the last day of end_month.
                // For simplicity, we can just use the start of the next month as end bound.
                let next_month_year = if end_month == 12 { year + 1 } else { year };
                let next_month = if end_month == 12 { 1 } else { end_month + 1 };

                let start_ts =
                    js_sys::Date::parse(&format!("{year:04}-{start_month:02}-01T00:00:00Z")) as i64;
                let end_ts = js_sys::Date::parse(&format!(
                    "{next_month_year:04}-{next_month:02}-01T00:00:00Z"
                )) as i64;

                db.get_user_items_by_range(user.id, start_ts, end_ts).await
            } else if let Some(year) = query.year {
                // If only year is provided, fetch for the whole year
                let start_ts = js_sys::Date::parse(&format!("{year:04}-01-01T00:00:00Z")) as i64;
                let end_ts = js_sys::Date::parse(&format!("{}:01-01T00:00:00Z", year + 1)) as i64;

                db.get_user_items_by_range(user.id, start_ts, end_ts).await
            } else {
                db.get_user_items_all(user.id).await
            };

            match user_items {
                Ok(user_items) => {
                    let status_map: std::collections::HashMap<String, db::UserItemSummary> =
                        user_items
                            .into_iter()
                            .map(|item| {
                                (
                                    item.title,
                                    db::UserItemSummary {
                                        status: item.status,
                                        score: item.score,
                                    },
                                )
                            })
                            .collect();
                    Response::from_json(&status_map)
                }
                Err(e) => {
                    console_error!("Failed to fetch user items: {}", e);
                    Response::error("Internal Server Error", 500)
                }
            }
        }
        Err(e) => {
            console_error!("Failed to get DB connection: {}", e);
            Response::error("Internal Server Error", 500)
        }
    }
}

pub async fn handle_metadata(mut req: Request, env: Env) -> Result<Response> {
    if req.method() == Method::Post {
        let requests: Vec<provider::MetadataRequest> = match req.json().await {
            Ok(r) => r,
            Err(_) => {
                return Response::error(
                    "Bad Request: Body must be a list of metadata requests",
                    400,
                );
            }
        };

        if requests.len() > 10 {
            return Response::error("Bad Request: Batch size exceeds limit of 10", 400);
        }

        let host = req
            .url()?
            .host_str()
            .unwrap_or("api.housou.local")
            .to_string();

        let futures = requests.into_iter().map(|r| {
            let env = &env;
            let host = &host;
            async move {
                let metadata = provider::fetch_metadata(&r, env, host).await.ok();
                provider::MetadataResponse {
                    request_id: r.request_id,
                    metadata,
                }
            }
        });

        let results: Vec<provider::MetadataResponse> = futures::future::join_all(futures).await;
        return Response::from_json(&results);
    }

    let url = req.url()?;
    let query_str = url.query().unwrap_or("");
    let query: MetadataQuery = serde_urlencoded::from_str(query_str).unwrap_or(MetadataQuery {
        tmdb_id: None,
        mal_id: None,
        anilist_id: None,
        title: None,
        begin: None,
    });

    let year = query
        .begin
        .as_deref()
        .and_then(|d| d.get(0..4))
        .and_then(|y| y.parse::<i32>().ok());

    let args = provider::MetadataArgs {
        tmdb_id: query.tmdb_id.as_deref(),
        mal_id: query.mal_id.as_deref(),
        anilist_id: query.anilist_id.as_deref(),
        title: query.title.as_deref(),
        year,
    };

    provider::get_metadata(args, &env).await
}
