pub mod anilist;
pub mod tmdb;

use crate::{ResponseExt, model};
use worker::*;

pub trait MetadataProvider {
    async fn fetch(
        &self,
        id: Option<&str>,
        title: Option<&str>,
        year: Option<i32>,
    ) -> Result<model::UnifiedMetadata>;
}

pub async fn get_metadata(
    tmdb_id: Option<&str>,
    title: Option<&str>,
    year: Option<i32>,
    env: &Env,
) -> Result<Response> {
    // 1. Try TMDb first if configured
    let tmdb = tmdb::TmdbProvider::new(env);

    match tmdb.fetch(tmdb_id, title, year).await {
        Ok(unified) => return create_response(&unified, env),
        Err(e) => console_log!("TMDb fetch failed {:?}", e),
    }

    // 2. Fallback to AniList
    let anilist = anilist::AnilistProvider;
    let fallback_title =
        title.ok_or_else(|| Error::RustError("Title required for metadata lookup".into()))?;

    match anilist.fetch(None, Some(fallback_title), year).await {
        Ok(unified) => create_response(&unified, env),
        Err(e) => Err(e),
    }
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
