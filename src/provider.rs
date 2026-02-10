pub mod anilist;
pub mod jikan;
pub mod tmdb;

use crate::{model, ResponseExt};
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
            Ok(unified) => return create_response(&unified, env, None),
            Err(e) => console_log!("TMDb fetch failed {:?}", e),
        }
    }

    // 2. Try Jikan if MAL ID is present (or no TMDB ID was found)
    if args.mal_id.is_some() {
        let jikan = jikan::JikanProvider;
        match jikan.fetch(args.mal_id, args.title, args.year).await {
            Ok(unified) => {
                return create_response(&unified, env, Some(crate::config::CACHE_TTL_JIKAN));
            }
            Err(e) => console_log!("Jikan fetch failed {:?}", e),
        }
    }

    // 3. Fallback to AniList
    let anilist = anilist::AnilistProvider;
    let fallback_title = args
        .title
        .ok_or_else(|| Error::RustError("Title required for metadata lookup".into()))?;

    match anilist.fetch(args.anilist_id, Some(fallback_title), args.year).await {
        Ok(unified) => create_response(&unified, env, None),
        Err(e) => Err(e),
    }
}

fn create_response(
    unified: &model::UnifiedMetadata,
    env: &Env,
    ttl_override: Option<i32>,
) -> Result<Response> {
    let ttl = if let Some(t) = ttl_override {
        t
    } else if unified.is_finished {
        crate::config::CACHE_TTL_FINISHED
    } else {
        crate::config::CACHE_TTL_ONGOING
    };

    Response::from_json(unified)?
        .add_cors(env)?
        .add_header("Cache-Control", &format!("public, max-age={}", ttl))
}
