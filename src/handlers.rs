use regex::Regex;
use serde_derive::Serialize;
use std::sync::OnceLock;
use time::{Date as TimeDate, Month};
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

fn normalize_season_query(season: Option<&str>) -> std::result::Result<Option<&str>, &'static str> {
    match season {
        Some("all") | None | Some("") => Ok(None),
        Some("Winter") | Some("Spring") | Some("Summer") | Some("Autumn") => Ok(season),
        Some(_) => Err("Bad Request: invalid 'season' parameter"),
    }
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
    let current_year = crate::utils::now_utc().year();
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

    let target_season = match normalize_season_query(query.season.as_deref()) {
        Ok(s) => s,
        Err(msg) => return Response::error(msg, 400),
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
    let query: ItemsQuery = match serde_urlencoded::from_str(query_str) {
        Ok(q) => q,
        Err(_) => return Response::error("Bad Request: invalid query", 400),
    };

    let normalized_season = match normalize_season_query(query.season.as_deref()) {
        Ok(s) => s,
        Err(msg) => return Response::error(msg, 400),
    };

    match auth::get_db(&env) {
        Ok(db) => {
            let user_items = if let Some(year) = query.year
                && let Some(season) = normalized_season
            {
                // Calculate timestamp range for the season
                let (start_month, end_month) = match season {
                    "Winter" => (1, 3),
                    "Spring" => (4, 6),
                    "Summer" => (7, 9),
                    "Autumn" => (10, 12),
                    _ => unreachable!("season is validated"),
                };

                // Approximate timestamps (milliseconds)
                let start_month = match Month::try_from(start_month as u8) {
                    Ok(m) => m,
                    Err(_) => Month::January,
                };
                let start_date =
                    TimeDate::from_calendar_date(year, start_month, 1).unwrap_or_else(|_| {
                        TimeDate::from_calendar_date(year, Month::January, 1).unwrap()
                    });
                let start_ts = start_date.midnight().assume_utc().unix_timestamp() * 1000;

                let next_month_year = if end_month == 12 { year + 1 } else { year };
                let next_month = if end_month == 12 { 1 } else { end_month + 1 };
                let next_month_m = match Month::try_from(next_month as u8) {
                    Ok(m) => m,
                    Err(_) => Month::January,
                };
                let end_date = TimeDate::from_calendar_date(next_month_year, next_month_m, 1)
                    .unwrap_or_else(|_| {
                        TimeDate::from_calendar_date(next_month_year, Month::January, 1).unwrap()
                    });
                let end_ts = end_date.midnight().assume_utc().unix_timestamp() * 1000;

                db.get_user_items_by_range(user.id, start_ts, end_ts).await
            } else if let Some(year) = query.year {
                // If only year is provided, fetch for the whole year
                let start_date = TimeDate::from_calendar_date(year, Month::January, 1)
                    .unwrap_or_else(|_| {
                        TimeDate::from_calendar_date(year, Month::January, 1).unwrap()
                    });
                let start_ts = start_date.midnight().assume_utc().unix_timestamp() * 1000;

                let next_year = year + 1;
                let end_date = TimeDate::from_calendar_date(next_year, Month::January, 1)
                    .unwrap_or_else(|_| {
                        TimeDate::from_calendar_date(next_year, Month::January, 1).unwrap()
                    });
                let end_ts = end_date.midnight().assume_utc().unix_timestamp() * 1000;

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
    let req_url = req.url()?;
    let cache_origin = req_url.origin().ascii_serialization();

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

        let futures = requests.into_iter().map(|r| {
            let env = &env;
            let cache_origin = cache_origin.clone();
            async move {
                let metadata = provider::fetch_metadata(&r, env, &cache_origin).await.ok();
                provider::MetadataResponse {
                    request_id: r.request_id,
                    metadata,
                }
            }
        });

        let results: Vec<provider::MetadataResponse> = futures::future::join_all(futures).await;
        return Response::from_json(&results);
    }

    let query_str = req_url.query().unwrap_or("");
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

    provider::get_metadata(args, &env, &cache_origin).await
}

#[derive(serde_derive::Deserialize)]
struct FaviconQuery {
    domain: String,
}

static DOMAIN_REGEX: OnceLock<Regex> = OnceLock::new();

fn is_safe_hostname(hostname: &str) -> bool {
    // 1. Basic length validation
    if hostname.is_empty() || hostname.len() > 255 {
        return false;
    }

    // 2. Reject characters dangerous for URLs to prevent userinfo injection, port specification, and path/query manipulation
    if hostname
        .chars()
        .any(|c| matches!(c, '@' | ':' | '/' | '\\' | '?' | '#' | ' ' | '\r' | '\n' | '\t'))
    {
        return false;
    }

    // 3. Reject valid IP addresses (IPv4/IPv6)
    if hostname.parse::<std::net::IpAddr>().is_ok() {
        return false;
    }

    // 4. Reject purely numeric hostnames (decimal representation of IPv4)
    if hostname.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    // 5. Require at least one dot and ensure labels are not empty
    let parts: Vec<&str> = hostname.split('.').collect();
    if parts.len() < 2 || parts.iter().any(|s| s.is_empty()) {
        return false;
    }

    // 6. Reject known non-public/reserved TLDs
    if let Some(tld) = parts.last() {
        let tld_lower = tld.to_lowercase();
        let reserved_tlds = [
            "local",
            "internal",
            "lan",
            "home",
            "host",
            "corp",
            "test",
            "invalid",
            "localhost",
            "onion",
            "example",
            "arpa",
        ];
        if reserved_tlds.contains(&tld_lower.as_str()) {
            return false;
        }
    }

    // 7. Use Regex to enforce standard domain name format
    let re = DOMAIN_REGEX.get_or_init(|| {
        Regex::new(r"^(?i)[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?(\.[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?)+$")
            .expect("Invalid domain regex")
    });

    re.is_match(hostname)
}

pub async fn handle_favicon(req: Request, _env: Env) -> Result<Response> {
    let url = req.url()?;
    let query_str = url.query().unwrap_or("");
    let query: FaviconQuery = serde_urlencoded::from_str(query_str)
        .map_err(|e| Error::RustError(format!("Invalid query: {}", e)))?;

    let hostname = &query.domain;

    // Use robust validation to prevent SSRF
    if !is_safe_hostname(hostname) {
        return Response::error("Bad Request: invalid domain", 400);
    }

    let providers = [
        format!("https://icons.duckduckgo.com/ip3/{}.ico", hostname),
        format!(
            "https://www.google.com/s2/favicons?domain={}&sz=32",
            hostname
        ),
        format!("https://{}/favicon.ico", hostname),
    ];

    for provider_url in &providers {
        let mut init = RequestInit::new();
        init.with_method(Method::Get);

        let request = Request::new_with_init(provider_url, &init)?;
        let response = Fetch::Request(request).send().await;

        if let Ok(mut resp) = response
            && resp.status_code() == 200
        {
            // Read the favicon bytes from upstream
            let bytes = resp.bytes().await?;

            let headers = Headers::new();
            if let Some(ct) = resp.headers().get("Content-Type")? {
                headers.set("Content-Type", &ct)?;
            } else {
                headers.set("Content-Type", "image/x-icon")?;
            }
            headers.set(
                "Cache-Control",
                &format!("public, max-age={}", config::CACHE_TTL_FAVICON),
            )?;

            return Ok(Response::from_bytes(bytes)?.with_headers(headers));
        }
    }

    // No favicon found from any provider
    Response::error("Not Found", 404)?.add_header(
        "Cache-Control",
        &format!("public, max-age={}", config::CACHE_TTL_FAVICON_404),
    )
}

#[cfg(test)]
mod tests {
    use super::normalize_season_query;

    #[test]
    fn test_is_safe_hostname() {
        use super::is_safe_hostname;

        // Valid domains
        assert!(is_safe_hostname("google.com"));
        assert!(is_safe_hostname("www.google.com"));
        assert!(is_safe_hostname("my-domain.co.uk"));
        assert!(is_safe_hostname("a.b.c.d.e.f.g.com"));

        // Invalid: Empty or too long
        assert!(!is_safe_hostname(""));
        assert!(!is_safe_hostname(&"a".repeat(256)));

        // Invalid: Suspicious characters
        assert!(!is_safe_hostname("user@google.com"));
        assert!(!is_safe_hostname("google.com:8080"));
        assert!(!is_safe_hostname("google.com/path"));
        assert!(!is_safe_hostname("google.com?query"));
        assert!(!is_safe_hostname("google.com#fragment"));
        assert!(!is_safe_hostname("google.com "));
        assert!(!is_safe_hostname("google\n.com"));

        // Invalid: IP addresses
        assert!(!is_safe_hostname("127.0.0.1"));
        assert!(!is_safe_hostname("8.8.8.8"));
        assert!(!is_safe_hostname("::1"));
        assert!(!is_safe_hostname("[::1]"));

        // Invalid: Numeric hostnames (decimal/hex representations of IPs)
        assert!(!is_safe_hostname("2130706433")); // 127.0.0.1 in decimal

        // Invalid: Reserved/Internal TLDs
        assert!(!is_safe_hostname("localhost"));
        assert!(!is_safe_hostname("something.local"));
        assert!(!is_safe_hostname("my.internal"));
        assert!(!is_safe_hostname("test.test"));
        assert!(!is_safe_hostname("example.onion"));

        // Invalid: Formatting issues
        assert!(!is_safe_hostname(".google.com"));
        assert!(!is_safe_hostname("google.com."));
        assert!(!is_safe_hostname("google..com"));
    }

    #[test]
    fn test_normalize_season_query() {
        assert_eq!(normalize_season_query(None).unwrap(), None);
        assert_eq!(normalize_season_query(Some("all")).unwrap(), None);
        assert_eq!(normalize_season_query(Some("")).unwrap(), None);
        assert_eq!(
            normalize_season_query(Some("Winter")).unwrap(),
            Some("Winter")
        );
        assert_eq!(
            normalize_season_query(Some("Spring")).unwrap(),
            Some("Spring")
        );
        assert!(normalize_season_query(Some("fall")).is_err());
        assert!(normalize_season_query(Some("Invalid")).is_err());
    }
}
