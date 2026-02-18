pub mod anilist;
pub mod jikan;
pub mod season;
pub mod tmdb;

use crate::{ResponseExt, model};
use serde_derive::{Deserialize, Serialize};
use worker::*;

#[derive(Debug, Default)]
pub struct MetadataArgs<'a> {
    pub tmdb_id: Option<&'a str>,
    pub mal_id: Option<&'a str>,
    pub anilist_id: Option<&'a str>,
    pub title: Option<&'a str>,
    pub year: Option<i32>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MetadataRequest {
    pub request_id: Option<String>,
    pub tmdb_id: Option<String>,
    pub mal_id: Option<String>,
    pub anilist_id: Option<String>,
    pub title: Option<String>,
    pub year: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetadataResponse {
    pub request_id: Option<String>,
    pub metadata: Option<model::UnifiedMetadata>,
}

pub trait MetadataProvider {
    async fn fetch(
        &self,
        id: Option<&str>,
        title: Option<&str>,
        year: Option<i32>,
    ) -> Result<model::UnifiedMetadata>;
}

/// Helper to generate a cache key URL for a request
fn get_cache_key(req: &MetadataRequest, host: &str) -> String {
    let mut params_map = std::collections::BTreeMap::new();

    if let Some(ref id) = req.anilist_id {
        params_map.insert("anilist_id", id.as_str());
    }
    if let Some(ref id) = req.mal_id {
        params_map.insert("mal_id", id.as_str());
    }
    if let Some(ref t) = req.title {
        params_map.insert("title", t.as_str());
    }
    if let Some(ref id) = req.tmdb_id {
        params_map.insert("tmdb_id", id.as_str());
    }
    let year_str; // Define lifetime outside
    if let Some(y) = req.year {
        year_str = y.to_string();
        params_map.insert("year", year_str.as_str());
    }

    let mut serializer = url::form_urlencoded::Serializer::new(String::new());
    for (k, v) in params_map {
        serializer.append_pair(k, v);
    }

    // "begin" was also used in handlers.rs logic to derive year, but here we just use year.
    // We should ensure the key matches what GET /api/metadata generates if we want shared cache.
    // But since we are creating a new internal cache path for consistency, let's stick to this.
    // Ideally we match `handlers::handle_metadata` URL construction.
    // But `handlers::handle_metadata` uses incoming request URL which might have extra params or different order.
    // For now, consistent internal key is sufficient.
    format!("https://{}/api/metadata?{}", host, serializer.finish())
}

pub async fn fetch_metadata(
    req: &MetadataRequest,
    env: &Env,
    host: &str,
) -> Result<model::UnifiedMetadata> {
    let cache = Cache::open(format!("housou-cache-{}", crate::config::CACHE_VERSION)).await;
    let cache_key = get_cache_key(req, host);

    // 1. Check Cache
    if let Ok(Some(mut resp)) = cache.get(&cache_key, true).await
        && let Ok(unified) = resp.json::<model::UnifiedMetadata>().await
    {
        return Ok(unified);
    }

    // 2. Fetch from Providers
    let args = MetadataArgs {
        tmdb_id: req.tmdb_id.as_deref(),
        mal_id: req.mal_id.as_deref(),
        anilist_id: req.anilist_id.as_deref(),
        title: req.title.as_deref(),
        year: req.year,
    };

    let (unified, ttl) = fetch_metadata_from_providers(args, env).await?;

    // 3. Cache Result
    let mut resp = Response::from_json(&unified)?;
    resp.headers_mut().set(
        "Cache-Control",
        &format!(
            "public, max-age={}",
            ttl.unwrap_or(crate::config::CACHE_TTL_ONGOING)
        ),
    )?;
    // We must ignore the promise here to not block, but we can await it if needed.
    // worker::Cache::put returns a Future.
    if let Err(e) = cache.put(&cache_key, resp).await {
        console_warn!("Failed to cache metadata: {:?}", e);
    }

    Ok(unified)
}

async fn fetch_metadata_from_providers(
    args: MetadataArgs<'_>,
    env: &Env,
) -> Result<(model::UnifiedMetadata, Option<i32>)> {
    // 1. Try TMDb
    if args.tmdb_id.is_some() {
        let tmdb = tmdb::TmdbProvider::new(env);
        match tmdb.fetch(args.tmdb_id, args.title, args.year).await {
            Ok(unified) => return Ok((unified, None)),
            Err(e) => console_log!("TMDb fetch failed {:?}", e),
        }
    }

    // 2. Try Jikan
    if args.mal_id.is_some() {
        let jikan = jikan::JikanProvider;
        match jikan.fetch(args.mal_id, args.title, args.year).await {
            Ok(unified) => {
                return Ok((unified, Some(crate::config::CACHE_TTL_JIKAN)));
            }
            Err(e) => console_log!("Jikan fetch failed {:?}", e),
        }
    }

    // 3. Fallback to AniList
    let anilist = anilist::AnilistProvider;
    let fallback_title = args
        .title
        .ok_or_else(|| Error::RustError("Title required for metadata lookup".into()))?;

    match anilist
        .fetch(args.anilist_id, Some(fallback_title), args.year)
        .await
    {
        Ok(unified) => Ok((unified, None)),
        Err(e) => Err(e),
    }
}

pub async fn get_metadata(args: MetadataArgs<'_>, env: &Env) -> Result<Response> {
    // Convert args to owned Request for internal function
    // For legacy/single requests, we use a dummy host or extract from env if possible.
    // But this function signature doesn't provide URL/Host.
    // We can use "api.housou.local" as internal host key.
    let req = MetadataRequest {
        request_id: None,
        tmdb_id: args.tmdb_id.map(|s| s.to_string()),
        mal_id: args.mal_id.map(|s| s.to_string()),
        anilist_id: args.anilist_id.map(|s| s.to_string()),
        title: args.title.map(|s| s.to_string()),
        year: args.year,
    };

    let unified = fetch_metadata(&req, env, "api.housou.local").await?;
    create_response(&unified, env, None)
}

fn create_response(
    unified: &model::UnifiedMetadata,
    _env: &Env,
    ttl_override: Option<i32>,
) -> Result<Response> {
    let ttl = if let Some(t) = ttl_override {
        t
    } else if unified.is_finished {
        crate::config::CACHE_TTL_FINISHED
    } else {
        crate::config::CACHE_TTL_ONGOING
    };

    Response::from_json(unified)?.add_header("Cache-Control", &format!("public, max-age={ttl}"))
}
