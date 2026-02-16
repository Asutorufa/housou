use serde_derive::Serialize;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use worker::*;

mod auth;
mod config;
mod db;
mod model;
mod provider;
mod utils;
use db::Database; // Import Database trait
use model::{Item, SiteMeta, SiteMetadata, SiteType};

pub trait ResponseExt {
    fn add_cors(self, env: &Env) -> Result<Response>;
    fn add_header(self, key: &str, value: &str) -> Result<Response>;
    fn add_security_headers(self) -> Result<Response>;
}

static CORS_ALLOWED_ORIGIN: OnceLock<String> = OnceLock::new();
static MIGRATION_DONE: AtomicBool = AtomicBool::new(false);

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
    auth_enabled: bool,
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
    // Migration Logic (Lazy)
    if let Ok(d1) = env.d1("DB")
        && MIGRATION_DONE
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    {
        let db = db::AppDatabase::new(d1);
        // We ignore migration errors here to not block the whole app,
        // but ideally we should log them.
        if let Err(e) = db.migrate().await {
            console_error!("Migration failed: {}", e);
            MIGRATION_DONE.store(false, Ordering::Relaxed);
        }
    }

    let cache = Cache::open(format!("housou-cache-{}", config::CACHE_VERSION)).await;
    let url = req.url()?;

    // 1. Handle caching and routing
    let resp = if req.method() == Method::Get {
        // Skip caching for auth routes and items when auth is enabled (user specific data)
        // Actually, we should check auth cookie presence before skipping cache, or make items endpoint handle caching carefully.
        // For simplicity: skip caching items endpoint if auth cookie is present or if we want to be safe.
        // But the router handles caching internally for items? No, router returns Response.
        // The main block handles caching.

        let is_auth_route =
            url.path().starts_with("/api/auth") || url.path().starts_with("/api/user");
        // We also want to skip caching /api/items if the user is authenticated, because the response is personalized.
        // Checking for cookie presence is a simple heuristic.
        let has_session_cookie = req
            .headers()
            .get("Cookie")?
            .unwrap_or_default()
            .contains("housou_session");

        if is_auth_route || (url.path() == "/api/items" && has_session_cookie) {
            router(req, env).await?
        } else if let Ok(Some(mut cached_resp)) = cache.get(url.as_str(), true).await {
            // Use cached response, clone to make it mutable for adding security headers
            cached_resp.cloned()?
        } else {
            // Generate new response
            let mut fresh_resp = router(req, env).await?;

            // Cache successful GET responses (except auth and personalized items)
            if url.path().starts_with("/api")
                && !is_auth_route
                && !(url.path() == "/api/items" && has_session_cookie)
                && fresh_resp.status_code() == 200
            {
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

async fn router(mut req: Request, env: Env) -> Result<Response> {
    let method = req.method();
    let path = req.path();
    let url = req.url()?;

    // Check if Auth is enabled (DB binding exists)
    let auth_enabled = env.d1("DB").is_ok();

    match (method.clone(), path.as_str()) {
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
                auth_enabled,
            };

            Response::from_json(&config_resp)?
                .add_cors(&env)?
                .add_header(
                    "Cache-Control",
                    &format!("public, max-age={}", config::CACHE_TTL_CONFIG),
                )
        }
        (Method::Get, "/api/items") => {
            let mut year_param = None;
            let mut season_param = None;

            for (k, v) in url.query_pairs() {
                if k == "year" {
                    year_param = Some(v);
                } else if k == "season" {
                    season_param = Some(v);
                }
            }

            let target_year = match year_param.as_deref().and_then(|y| y.parse::<i32>().ok()) {
                Some(y) => y,
                None => return Response::error("Bad Request: 'year' parameter is required", 400),
            };

            let target_season = match season_param.as_deref() {
                Some("all") | None | Some("") => None,
                Some(s) => Some(s),
            };

            let items = fetch_items_for_season(target_year, target_season).await?;
            Response::from_json(&items)?.add_cors(&env)
        }
        (Method::Post, "/api/user/status") if auth_enabled => {
            let titles: Vec<String> = match req.json().await {
                Ok(t) => t,
                Err(_) => {
                    return Response::error("Bad Request: Body must be a list of strings", 400);
                }
            };

            match auth::get_auth(&req, &env).await {
                Ok(Some((user, _))) => match auth::get_db(&env) {
                    Ok(db) => match db.get_user_items_by_titles(user.id, &titles).await {
                        Ok(user_items) => {
                            let status_map: std::collections::HashMap<_, _> = user_items
                                .into_iter()
                                .map(|ui| {
                                    (
                                        ui.title,
                                        db::UserItemSummary {
                                            status: ui.status,
                                            score: ui.score,
                                        },
                                    )
                                })
                                .collect();
                            Response::from_json(&status_map)?.add_cors(&env)
                        }
                        Err(e) => {
                            console_error!("Failed to fetch user items: {}", e);
                            Response::error("Internal Server Error", 500)
                        }
                    },
                    Err(e) => {
                        console_error!("Failed to get DB connection: {}", e);
                        Response::error("Internal Server Error", 500)
                    }
                },
                Ok(None) => Response::error("Unauthorized", 401),
                Err(e) => {
                    console_error!("Auth error: {}", e);
                    Response::error("Internal Server Error", 500)
                }
            }
        }

        (Method::Get, "/api/metadata") => {
            let mut tmdb_id = None;
            let mut mal_id = None;
            let mut anilist_id = None;
            let mut title = None;
            let mut begin_param = None;

            for (k, v) in url.query_pairs() {
                match k.as_ref() {
                    "tmdb_id" => tmdb_id = Some(v),
                    "mal_id" => mal_id = Some(v),
                    "anilist_id" => anilist_id = Some(v),
                    "title" => title = Some(v),
                    "begin" => begin_param = Some(v),
                    _ => {}
                }
            }

            let year = begin_param
                .as_deref()
                .and_then(|d| d.get(0..4))
                .and_then(|y| y.parse::<i32>().ok());

            let args = provider::MetadataArgs {
                tmdb_id: tmdb_id.as_deref(),
                mal_id: mal_id.as_deref(),
                anilist_id: anilist_id.as_deref(),
                title: title.as_deref(),
                year,
            };

            provider::get_metadata(args, &env).await
        }
        // Auth Routes (Only if enabled)
        (Method::Post, "/api/auth/register") if auth_enabled => {
            auth::handle_register(req, env.clone())
                .await?
                .add_cors(&env)
        }
        (Method::Post, "/api/auth/login") if auth_enabled => {
            auth::handle_login(req, env.clone()).await?.add_cors(&env)
        }
        (Method::Post, "/api/auth/logout") if auth_enabled => {
            auth::handle_logout(req, env.clone()).await?.add_cors(&env)
        }
        (Method::Get, "/api/auth/me") if auth_enabled => {
            auth::handle_me(req, env.clone()).await?.add_cors(&env)
        }
        (Method::Put, "/api/auth/profile") if auth_enabled => {
            auth::handle_update_profile(req, env.clone())
                .await?
                .add_cors(&env)
        }
        (Method::Put, "/api/auth/password") if auth_enabled => {
            auth::handle_change_password(req, env.clone())
                .await?
                .add_cors(&env)
        }
        (Method::Get, "/api/auth/github/authorize") if auth_enabled => {
            auth::handle_github_authorize(req, env.clone())
                .await?
                .add_cors(&env)
        }
        (Method::Get, "/api/auth/github/callback") if auth_enabled => {
            auth::handle_github_callback(req, env.clone())
                .await?
                .add_cors(&env)
        }
        (Method::Get, "/api/user/item") if auth_enabled => auth::handle_get_item(req, env.clone())
            .await?
            .add_cors(&env),
        (Method::Post, "/api/user/item") if auth_enabled => {
            auth::handle_update_item(req, env.clone())
                .await?
                .add_cors(&env)
        }

        // Handle Options for CORS on auth routes
        (Method::Options, path)
            if path.starts_with("/api/auth") || path.starts_with("/api/user") =>
        {
            Response::empty()?.add_cors(&env)
        }

        _ => Response::error("Not Found", 404),
    }
}
