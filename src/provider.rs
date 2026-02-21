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

#[derive(Debug, Clone, Copy)]
pub enum LookupQuery<'a> {
    ById(&'a str),
    ByTitle { title: &'a str, year: Option<i32> },
}

pub trait MetadataProvider {
    async fn fetch(&self, query: LookupQuery<'_>) -> Result<model::UnifiedMetadata>;
}

#[derive(Serialize)]
struct CacheKeyParams<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    anilist_id: Option<&'a String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mal_id: Option<&'a String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<&'a String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tmdb_id: Option<&'a String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    year: Option<i32>,
}

/// Helper to generate a cache key URL for a request
fn get_cache_key(req: &MetadataRequest, cache_origin: &str) -> String {
    let params = CacheKeyParams {
        anilist_id: req.anilist_id.as_ref(),
        mal_id: req.mal_id.as_ref(),
        title: req.title.as_ref(),
        tmdb_id: req.tmdb_id.as_ref(),
        year: req.year,
    };
    let qs = serde_urlencoded::to_string(&params).unwrap_or_default();
    format!("{}/api/metadata?{}", cache_origin, qs)
}

pub async fn fetch_metadata(
    req: &MetadataRequest,
    ctx: &RouteContext<Context>,
    cache_origin: &str,
) -> Result<model::UnifiedMetadata> {
    let cache = Cache::open(format!("housou-cache-{}", crate::config::CACHE_VERSION)).await;
    let cache_key = get_cache_key(req, cache_origin);

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

    let (unified, ttl) = fetch_metadata_from_providers(args, &ctx.env).await?;

    // 3. Cache Result
    let mut resp = Response::from_json(&unified)?;
    resp.headers_mut().set(
        "Cache-Control",
        &format!(
            "public, max-age={}",
            ttl.unwrap_or(crate::config::CACHE_TTL_ONGOING)
        ),
    )?;

    // Don't block the response for cache write. Use waitUntil.
    ctx.data.wait_until(async move {
        if let Err(e) = cache.put(&cache_key, resp).await {
            console_warn!("Failed to cache metadata: {:?}", e);
        }
    });

    Ok(unified)
}

async fn fetch_metadata_from_providers(
    args: MetadataArgs<'_>,
    env: &Env,
) -> Result<(model::UnifiedMetadata, Option<i32>)> {
    // Build the title-based query once (reused across providers)
    let title_query = args.title.map(|t| LookupQuery::ByTitle {
        title: t,
        year: args.year,
    });

    // 1. Try TMDb first (always — by ID if available, otherwise by title search)
    let tmdb_query = args.tmdb_id.map(LookupQuery::ById).or(title_query);
    if let Some(query) = tmdb_query {
        let tmdb = tmdb::TmdbProvider::new(env);
        match tmdb.fetch(query).await {
            Ok(unified) => return Ok((unified, None)),
            Err(e) => console_log!("TMDb fetch failed {:?}", e),
        }
    }

    // 2. Try Jikan (ID-only provider)
    if let Some(id) = args.mal_id {
        let jikan = jikan::JikanProvider;
        match jikan.fetch(LookupQuery::ById(id)).await {
            Ok(unified) => {
                return Ok((unified, Some(crate::config::CACHE_TTL_JIKAN)));
            }
            Err(e) => console_log!("Jikan fetch failed {:?}", e),
        }
    }

    // 3. Fallback to AniList
    let anilist_query = args.anilist_id.map(LookupQuery::ById).or(title_query);
    if let Some(query) = anilist_query {
        let anilist = anilist::AnilistProvider;
        match anilist.fetch(query).await {
            Ok(unified) => return Ok((unified, None)),
            Err(e) => console_log!("AniList fetch failed {:?}", e),
        }
    }

    Err(Error::RustError(
        "No metadata provider could fulfill the request".into(),
    ))
}

pub async fn get_metadata(
    args: MetadataArgs<'_>,
    ctx: &RouteContext<Context>,
    cache_origin: &str,
) -> Result<Response> {
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

    let unified = fetch_metadata(&req, ctx, cache_origin).await?;
    create_response(&unified, &ctx.env, None)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_is_stable() {
        let req = MetadataRequest {
            request_id: Some("abc".into()),
            tmdb_id: Some("tv/1".into()),
            mal_id: Some("2".into()),
            anilist_id: Some("3".into()),
            title: Some("Test".into()),
            year: Some(2026),
        };

        let key1 = get_cache_key(&req, "https://example.com");
        let key2 = get_cache_key(&req, "https://example.com");
        assert_eq!(key1, key2);
        assert!(key1.starts_with("https://example.com/api/metadata?"));
        assert!(!key1.contains("request_id"));
    }
}
