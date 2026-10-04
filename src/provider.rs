pub mod anilist;
mod cache;
pub mod jikan;
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

#[derive(Debug, Clone)]
pub(super) struct ProviderFetch {
    pub metadata: model::UnifiedMetadata,
    pub tmdb_failed: bool,
}

pub async fn fetch_metadata(
    req: &MetadataRequest,
    ctx: &RouteContext<Context>,
    cache_origin: &str,
) -> Result<model::UnifiedMetadata> {
    cache::fetch_metadata(req, ctx, cache_origin).await
}

pub(super) async fn fetch_tmdb_metadata(
    args: MetadataArgs<'_>,
    env: &Env,
) -> Option<Result<model::UnifiedMetadata>> {
    let title_query = args.title.map(|title| LookupQuery::ByTitle {
        title,
        year: args.year,
    });
    let query = args.tmdb_id.map(LookupQuery::ById).or(title_query)?;
    let tmdb = tmdb::TmdbProvider::new(env);
    Some(tmdb.fetch(query).await)
}

pub(super) async fn fetch_metadata_from_providers(
    args: MetadataArgs<'_>,
    env: &Env,
) -> Result<ProviderFetch> {
    let mut tmdb_failed = false;

    // TMDb is the preferred source because it provides the richest metadata.
    if let Some(result) = fetch_tmdb_metadata(args, env).await {
        match result {
            Ok(metadata) => {
                return Ok(ProviderFetch {
                    metadata,
                    tmdb_failed: false,
                });
            }
            Err(error) => {
                tmdb_failed = true;
                console_log!("TMDb fetch failed {:?}", error);
            }
        }
    }

    // Jikan is ID-only and is the first fallback when MAL data is available.
    if let Some(id) = args.mal_id {
        let jikan = jikan::JikanProvider;
        match jikan.fetch(LookupQuery::ById(id)).await {
            Ok(metadata) => {
                return Ok(ProviderFetch {
                    metadata,
                    tmdb_failed,
                });
            }
            Err(error) => console_log!("Jikan fetch failed {:?}", error),
        }
    }

    // AniList is the final fallback and can search by title when no ID exists.
    let title_query = args.title.map(|title| LookupQuery::ByTitle {
        title,
        year: args.year,
    });
    let anilist_query = args.anilist_id.map(LookupQuery::ById).or(title_query);
    if let Some(query) = anilist_query {
        let anilist = anilist::AnilistProvider;
        match anilist.fetch(query).await {
            Ok(metadata) => {
                return Ok(ProviderFetch {
                    metadata,
                    tmdb_failed,
                });
            }
            Err(error) => console_log!("AniList fetch failed {:?}", error),
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
    let req = MetadataRequest {
        request_id: None,
        tmdb_id: args.tmdb_id.map(str::to_string),
        mal_id: args.mal_id.map(str::to_string),
        anilist_id: args.anilist_id.map(str::to_string),
        title: args.title.map(str::to_string),
        year: args.year,
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
