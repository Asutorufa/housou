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
    let current_year = crate::utils::now_utc()?.year();
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
            let user_items = if let Some(year) = query.year {
                let (start_ts, end_ts) =
                    utils::season::get_season_timestamp_range(year, normalized_season)
                        .map_err(Error::RustError)?;

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

pub async fn handle_metadata(mut req: Request, ctx: RouteContext<Context>) -> Result<Response> {
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
            let ctx = &ctx;
            let cache_origin = cache_origin.clone();
            async move {
                let metadata = provider::fetch_metadata(&r, ctx, &cache_origin).await.ok();
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

    provider::get_metadata(args, &ctx, &cache_origin).await
}

#[derive(serde_derive::Deserialize)]
struct FaviconQuery {
    domain: String,
}

#[derive(serde_derive::Deserialize)]
struct ReviewListQuery {
    title: Option<String>,
    page: Option<i64>,
    page_size: Option<i64>,
}

#[derive(serde_derive::Deserialize)]
struct CreateReviewRequest {
    title: String,
    comment: String,
    score: Option<i32>,
}

#[derive(serde_derive::Deserialize)]
struct DeleteReviewRequest {
    title: String,
}

#[async_trait::async_trait(?Send)]
pub trait FaviconFetcher {
    async fn fetch(&self, url: &str) -> Result<Option<(Vec<u8>, String)>>;
}

struct WorkerFaviconFetcher;

#[async_trait::async_trait(?Send)]
impl FaviconFetcher for WorkerFaviconFetcher {
    async fn fetch(&self, url: &str) -> Result<Option<(Vec<u8>, String)>> {
        let mut init = RequestInit::new();
        init.with_method(Method::Get);

        let request = Request::new_with_init(url, &init)?;
        let response = Fetch::Request(request).send().await;

        match response {
            Ok(mut resp) if resp.status_code() == 200 => {
                let bytes = resp.bytes().await?;
                let content_type = resp
                    .headers()
                    .get("Content-Type")?
                    .unwrap_or_else(|| "image/x-icon".to_string());
                Ok(Some((bytes, content_type)))
            }
            _ => Ok(None),
        }
    }
}

async fn fetch_favicon_core(
    hostname: &str,
    fetcher: &impl FaviconFetcher,
) -> Result<Option<(Vec<u8>, String)>> {
    let providers = [
        format!("https://icons.duckduckgo.com/ip3/{}.ico", hostname),
        format!(
            "https://www.google.com/s2/favicons?domain={}&sz=32",
            hostname
        ),
        format!("https://{}/favicon.ico", hostname),
    ];

    let futures = providers.iter().map(|url| {
        Box::pin(async move {
            match fetcher.fetch(url).await {
                Ok(Some(res)) => Ok(res),
                Ok(None) => Err(Error::RustError("Not Found".to_string())),
                Err(e) => Err(e),
            }
        })
    });

    match futures::future::select_ok(futures).await {
        Ok((res, _)) => Ok(Some(res)),
        Err(_) => Ok(None),
    }
}

pub async fn handle_favicon(req: Request, _env: Env) -> Result<Response> {
    let url = req.url()?;
    let query_str = url.query().unwrap_or("");
    let query: FaviconQuery = serde_urlencoded::from_str(query_str)
        .map_err(|e| Error::RustError(format!("Invalid query: {}", e)))?;

    let hostname = &query.domain;

    // Use robust validation to prevent SSRF
    if !utils::validation::is_safe_hostname(hostname) {
        return Response::error("Bad Request: invalid domain", 400);
    }

    let fetcher = WorkerFaviconFetcher;
    if let Some((bytes, content_type)) = fetch_favicon_core(hostname, &fetcher).await? {
        let headers = Headers::new();
        headers.set("Content-Type", &content_type)?;
        headers.set(
            "Cache-Control",
            &format!("public, max-age={}", config::CACHE_TTL_FAVICON),
        )?;

        return Ok(Response::from_bytes(bytes)?.with_headers(headers));
    }

    // No favicon found from any provider
    Response::error("Not Found", 404)?.add_header(
        "Cache-Control",
        &format!("public, max-age={}", config::CACHE_TTL_FAVICON_404),
    )
}

pub async fn handle_list_reviews(req: Request, env: Env) -> Result<Response> {
    let current_user_id = auth::get_auth(&req, &env).await?.map(|(u, _)| u.id);
    let url = req.url()?;
    let query_str = url.query().unwrap_or("");
    let query: ReviewListQuery = match serde_urlencoded::from_str(query_str) {
        Ok(v) => v,
        Err(_) => return Response::error("Bad Request: invalid query", 400),
    };
    let title = match query.title {
        Some(t) if !t.is_empty() => t,
        _ => return Response::error("Bad Request: 'title' parameter is required", 400),
    };
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 50);
    let offset = (page - 1) * page_size;

    let db = auth::get_db(&env)?;
    let reviews = db
        .get_item_reviews_by_title(&title, current_user_id, page_size, offset)
        .await?;
    Response::from_json(&reviews)
}

pub async fn handle_create_review(mut req: Request, env: Env) -> Result<Response> {
    let (user, _) = match auth::get_auth(&req, &env).await? {
        Some(u) => u,
        None => return Response::error("Unauthorized", 401),
    };

    let body: CreateReviewRequest = req.json().await?;
    if body.title.trim().is_empty() || body.comment.trim().is_empty() {
        return Response::error("Bad Request: title/comment cannot be empty", 400);
    }

    let db = auth::get_db(&env)?;
    db.create_item_review(&body.title, user.id, body.score, &body.comment)
        .await?;
    Response::ok("Created")
}

pub async fn handle_delete_review(mut req: Request, env: Env) -> Result<Response> {
    let (user, _) = match auth::get_auth(&req, &env).await? {
        Some(u) => u,
        None => return Response::error("Unauthorized", 401),
    };

    let body: DeleteReviewRequest = req.json().await?;
    if body.title.trim().is_empty() {
        return Response::error("Bad Request: title cannot be empty", 400);
    }

    let db = auth::get_db(&env)?;
    db.delete_item_review(user.id, &body.title).await?;
    Response::ok("Deleted")
}

#[cfg(test)]
mod tests {
    use super::{FaviconFetcher, fetch_favicon_core, normalize_season_query};
    use std::collections::HashMap;
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::{Arc, Mutex};
    use std::task::{Context, Poll};
    use worker::{Error, Result};

    #[test]
    fn test_normalize_season_query() {
        // Valid cases
        assert_eq!(normalize_season_query(None).unwrap(), None);
        assert_eq!(normalize_season_query(Some("all")).unwrap(), None);
        assert_eq!(normalize_season_query(Some("")).unwrap(), None);

        for season in ["Winter", "Spring", "Summer", "Autumn"] {
            assert_eq!(normalize_season_query(Some(season)).unwrap(), Some(season));
        }

        // Invalid cases
        let invalid_cases = [
            // Lowercase
            "winter",
            "spring",
            "summer",
            "autumn",
            // Uppercase
            "WINTER",
            "SPRING",
            "SUMMER",
            "AUTUMN",
            // Mixed case
            "WiNtEr",
            "sPrInG",
            // Whitespace
            " Winter",
            "Spring ",
            "\nSummer\t",
            // Other "all" variants
            "ALL",
            "All",
            // Miscellaneous
            "fall",
            "Invalid",
            "1",
            "2024",
        ];

        for case in invalid_cases {
            assert!(
                normalize_season_query(Some(case)).is_err(),
                "Expected error for input: {}",
                case
            );
        }
    }

    struct DelayedResult<T> {
        value: Option<T>,
        delay_polls: usize,
    }

    struct DelayedFuture<T> {
        state: Arc<Mutex<DelayedResult<T>>>,
    }

    impl<T: Unpin> Future for DelayedFuture<T> {
        type Output = T;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let mut state = self.state.lock().unwrap();
            if state.delay_polls > 0 {
                state.delay_polls -= 1;
                cx.waker().wake_by_ref();
                Poll::Pending
            } else {
                Poll::Ready(state.value.take().unwrap())
            }
        }
    }

    struct MockResponse {
        data: Option<(Vec<u8>, String)>,
        error: Option<String>,
        delay: usize,
    }

    struct MockFaviconFetcher {
        responses: HashMap<String, MockResponse>,
    }

    impl MockFaviconFetcher {
        fn new() -> Self {
            Self {
                responses: HashMap::new(),
            }
        }

        fn add_response(
            &mut self,
            url: &str,
            result: Result<Option<(Vec<u8>, String)>>,
            delay: usize,
        ) {
            let (data, error) = match result {
                Ok(d) => (d, None),
                Err(e) => (None, Some(e.to_string())),
            };
            self.responses
                .insert(url.to_string(), MockResponse { data, error, delay });
        }
    }

    #[async_trait::async_trait(?Send)]
    impl FaviconFetcher for MockFaviconFetcher {
        async fn fetch(&self, url: &str) -> Result<Option<(Vec<u8>, String)>> {
            if let Some(res) = self.responses.get(url) {
                let result = if let Some(e) = &res.error {
                    Err(Error::RustError(e.clone()))
                } else {
                    Ok(res.data.clone())
                };

                let state = Arc::new(Mutex::new(DelayedResult {
                    value: Some(result),
                    delay_polls: res.delay,
                }));

                DelayedFuture { state }.await
            } else {
                Ok(None)
            }
        }
    }

    #[test]
    fn test_favicon_race_condition() {
        let mut fetcher = MockFaviconFetcher::new();
        let hostname = "example.com";
        // Order in fetch_favicon_core: duckduckgo, google, direct
        let url_slow = format!("https://icons.duckduckgo.com/ip3/{}.ico", hostname);
        let url_fast = format!(
            "https://www.google.com/s2/favicons?domain={}&sz=32",
            hostname
        );

        // SLOW returns "A" after 10 polls
        fetcher.add_response(
            &url_slow,
            Ok(Some((b"A".to_vec(), "image/x-icon".into()))),
            10,
        );
        // FAST returns "B" after 1 poll
        fetcher.add_response(&url_fast, Ok(Some((b"B".to_vec(), "image/png".into()))), 1);

        let result = futures::executor::block_on(fetch_favicon_core(hostname, &fetcher));

        // CONCURRENT IMPLEMENTATION (Optimization):
        // duckduckgo is first but SLOW (10 polls).
        // Google is second but FAST (1 poll).
        // We expect "B" because it finishes first.

        let (bytes, _) = result.unwrap().unwrap();
        assert_eq!(
            bytes, b"B",
            "Concurrent implementation should return the faster provider's result"
        );
    }

    #[test]
    fn test_favicon_all_fail() {
        let fetcher = MockFaviconFetcher::new();
        let hostname = "example.com";
        // All fail

        // No responses added = all return Ok(None) or Err.
        // MockFaviconFetcher returns Ok(None) if not found.
        // fetch_favicon_core maps Ok(None) to Err.
        // select_ok should fail if all fail.
        // fetch_favicon_core catches Err from select_ok and returns Ok(None).

        let result = futures::executor::block_on(fetch_favicon_core(hostname, &fetcher));
        assert!(result.unwrap().is_none());
    }
}
