pub mod anilist;
pub mod jikan;
pub mod tmdb;

use crate::{ResponseExt, model};
use worker::*;

#[derive(Debug, Default)]
pub struct MetadataArgs<'a> {
    pub tmdb_id: Option<&'a str>,
    pub mal_id: Option<&'a str>,
    pub anilist_id: Option<&'a str>,
    pub title: Option<&'a str>,
    pub year: Option<i32>,
}

pub trait MetadataProvider {
    async fn fetch(
        &self,
        id: Option<&str>,
        title: Option<&str>,
        year: Option<i32>,
    ) -> Result<model::UnifiedMetadata>;
}

pub async fn get_metadata(args: MetadataArgs<'_>, env: &Env) -> Result<Response> {
    // 1. Try TMDb first if TMDB ID is present or configured
    if args.tmdb_id.is_some() {
        let tmdb = tmdb::TmdbProvider::new(env);
        match tmdb.fetch(args.tmdb_id, args.title, args.year).await {
            Ok(unified) => return create_response(&unified, env),
            Err(e) => console_log!("TMDb fetch failed {:?}", e),
        }
    }

    // 2. Try Jikan if MAL ID is present (or no TMDB ID was found)
    if args.mal_id.is_some() {
        let jikan = jikan::JikanProvider;
        match jikan.fetch(args.mal_id, args.title, args.year).await {
            Ok(unified) => return create_response(&unified, env),
            Err(e) => console_log!("Jikan fetch failed {:?}", e),
        }
    }

    // 3. Fallback to AniList (try ID first, then title)
    let anilist = anilist::AnilistProvider;
    // We pass ID if present, otherwise title fallback logic is handled inside or we error here if both missing?
    // provider::anilist::fetch logic will be updated to handle ID.
    // If ID is missing, title is required.

    if args.anilist_id.is_some() || args.title.is_some() {
        match anilist.fetch(args.anilist_id, args.title, args.year).await {
            Ok(unified) => return create_response(&unified, env),
            Err(e) => console_log!("AniList fetch failed {:?}", e),
        }
    }

    // If all failed or no inputs
    Err(Error::RustError(
        "No suitable metadata provider found or all failed".into(),
    ))
}

fn create_response(unified: &model::UnifiedMetadata, env: &Env) -> Result<Response> {
    let ttl = if unified.is_finished {
        crate::config::CACHE_TTL_FINISHED
    } else {
        crate::config::CACHE_TTL_ONGOING
    };

    Response::from_json(unified)?
        .add_cors(env)?
        .add_header("Cache-Control", &format!("public, max-age={}", ttl))
}
