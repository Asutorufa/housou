pub mod anilist;
pub mod bangumi;
mod cache;
pub mod jikan;
mod match_score;
pub mod season;
pub mod tmdb;

use crate::{ResponseExt, model};
use serde_derive::{Deserialize, Serialize};
use worker::*;

#[derive(Debug, Default, Clone, Copy)]
pub struct MetadataArgs<'a> {
    pub tmdb_id: Option<&'a str>,
    pub mal_id: Option<&'a str>,
    pub anilist_id: Option<&'a str>,
    pub bangumi_id: Option<&'a str>,
    pub title: Option<&'a str>,
    pub aliases: &'a [String],
    pub year: Option<i32>,
    pub media_type: Option<&'a str>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MetadataRequest {
    pub request_id: Option<String>,
    pub tmdb_id: Option<String>,
    pub mal_id: Option<String>,
    pub anilist_id: Option<String>,
    pub bangumi_id: Option<String>,
    pub title: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
    pub year: Option<i32>,
    pub media_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetadataResponse {
    pub request_id: Option<String>,
    pub metadata: Option<model::UnifiedMetadata>,
}

#[derive(Debug, Clone, Copy)]
pub enum LookupQuery<'a> {
    ById(&'a str),
    ByTitle {
        title: &'a str,
        aliases: &'a [String],
        year: Option<i32>,
        media_type: Option<&'a str>,
    },
}

pub trait MetadataProvider {
    async fn fetch(&self, query: LookupQuery<'_>) -> Result<model::UnifiedMetadata>;
}

#[derive(Debug, Clone)]
pub(super) struct ProviderFetch {
    pub metadata: model::UnifiedMetadata,
    pub tmdb_retry_after_seconds: Option<i32>,
}

#[derive(Debug)]
pub(super) struct ProviderChainError {
    pub error: Error,
    pub retry_after_seconds: i32,
}

pub async fn fetch_metadata(
    req: &MetadataRequest,
    ctx: &RouteContext<Context>,
    cache_origin: &str,
) -> Result<model::UnifiedMetadata> {
    cache::fetch_metadata(req, ctx, cache_origin).await
}

fn title_query(args: MetadataArgs<'_>) -> Option<LookupQuery<'_>> {
    args.title.map(|title| LookupQuery::ByTitle {
        title,
        aliases: args.aliases,
        year: args.year,
        media_type: args.media_type,
    })
}

pub(super) async fn fetch_tmdb_metadata(
    args: MetadataArgs<'_>,
    env: &Env,
) -> Option<Result<model::UnifiedMetadata>> {
    let query = args
        .tmdb_id
        .map(LookupQuery::ById)
        .or_else(|| title_query(args))?;
    let tmdb = tmdb::TmdbProvider::new(env);
    Some(tmdb.fetch(query).await)
}

fn provider_retry_after_seconds(error: &Error) -> i32 {
    let error = error.to_string().to_ascii_lowercase();
    if error.contains("no suitable match")
        || error.contains("not found")
        || error.contains("404")
        || error.contains("invalid movie id")
        || error.contains("invalid show id")
    {
        crate::config::CACHE_TTL_METADATA_MISS
    } else {
        crate::config::CACHE_TTL_METADATA_TRANSIENT
    }
}

pub(super) async fn fetch_metadata_from_providers(
    args: MetadataArgs<'_>,
    env: &Env,
) -> std::result::Result<ProviderFetch, ProviderChainError> {
    let mut tmdb_retry_after = None;
    let mut chain_retry_after = crate::config::CACHE_TTL_METADATA_MISS;

    // TMDb is the preferred source because it provides the richest metadata.
    if let Some(result) = fetch_tmdb_metadata(args, env).await {
        match result {
            Ok(metadata) => {
                return Ok(ProviderFetch {
                    metadata,
                    tmdb_retry_after_seconds: None,
                });
            }
            Err(error) => {
                let retry_after = provider_retry_after_seconds(&error);
                tmdb_retry_after = Some(retry_after);
                chain_retry_after = chain_retry_after.min(retry_after);
                console_log!("TMDb fetch failed {:?}", error);
            }
        }
    }

    // Jikan is deterministic when a MAL ID is available.
    if let Some(id) = args.mal_id {
        let jikan = jikan::JikanProvider;
        match jikan.fetch(LookupQuery::ById(id)).await {
            Ok(metadata) => {
                return Ok(ProviderFetch {
                    metadata,
                    tmdb_retry_after_seconds: tmdb_retry_after,
                });
            }
            Err(error) => {
                chain_retry_after = chain_retry_after.min(provider_retry_after_seconds(&error));
                console_log!("Jikan fetch failed {:?}", error);
            }
        }
    }

    // AniList can use its own ID, a MAL ID, or ranked title matching.
    let anilist = anilist::AnilistProvider;
    let anilist_result = if let Some(id) = args.anilist_id {
        Some(anilist.fetch(LookupQuery::ById(id)).await)
    } else if let Some(mal_id) = args.mal_id {
        match anilist.fetch_by_mal_id(mal_id).await {
            Ok(Some(metadata)) => Some(Ok(metadata)),
            Ok(None) => {
                // A missing MAL mapping must not remove the existing title fallback.
                if let Some(query) = title_query(args) {
                    Some(anilist.fetch(query).await)
                } else {
                    None
                }
            }
            Err(error) => Some(Err(error)),
        }
    } else if let Some(query) = title_query(args) {
        Some(anilist.fetch(query).await)
    } else {
        None
    };

    if let Some(result) = anilist_result {
        match result {
            Ok(metadata) => {
                return Ok(ProviderFetch {
                    metadata,
                    tmdb_retry_after_seconds: tmdb_retry_after,
                });
            }
            Err(error) => {
                chain_retry_after = chain_retry_after.min(provider_retry_after_seconds(&error));
                console_log!("AniList fetch failed {:?}", error);
            }
        }
    }

    // Bangumi is the final fallback. Prefer a known subject ID, otherwise use
    // ranked title matching against anime subjects only.
    let bangumi = bangumi::BangumiProvider;
    let bangumi_query = args
        .bangumi_id
        .map(LookupQuery::ById)
        .or_else(|| title_query(args));
    if let Some(query) = bangumi_query {
        match bangumi.fetch(query).await {
            Ok(metadata) => {
                return Ok(ProviderFetch {
                    metadata,
                    tmdb_retry_after_seconds: tmdb_retry_after,
                });
            }
            Err(error) => {
                chain_retry_after = chain_retry_after.min(provider_retry_after_seconds(&error));
                console_log!("Bangumi fetch failed {:?}", error);
            }
        }
    }

    Err(ProviderChainError {
        error: Error::RustError("No metadata provider could fulfill the request".into()),
        retry_after_seconds: chain_retry_after,
    })
}

pub async fn get_metadata(
    args: MetadataArgs<'_>,
    ctx: &RouteContext<Context>,
    cache_origin: &str,
) -> Result<Response> {
    let req = MetadataRequest {
        request_id: None,
        tmdb_id: args.tmdb_id.map(str::to_string),
        mal_id: args.mal_id.map(str::to_string),
        anilist_id: args.anilist_id.map(str::to_string),
        bangumi_id: args.bangumi_id.map(str::to_string),
        title: args.title.map(str::to_string),
        aliases: args.aliases.to_vec(),
        year: args.year,
        media_type: args.media_type.map(str::to_string),
    };

    let unified = fetch_metadata(&req, ctx, cache_origin).await?;
    create_response(&unified)
}

fn create_response(unified: &model::UnifiedMetadata) -> Result<Response> {
    Response::from_json(unified)?.add_header(
        "Cache-Control",
        &format!("public, max-age={}", crate::config::CACHE_TTL_METADATA_L1),
    )
}
