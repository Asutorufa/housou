use serde_derive::Serialize;
use worker::*;

use crate::db::Database;
use crate::model::{SiteMeta, SiteMetadata, SiteType};
use crate::{ResponseExt, auth, config, db, provider, utils};

#[derive(Serialize)]
pub struct ConfigResponse<'a> {
    pub site_meta: &'a SiteMeta,
    pub years: Vec<i32>,
    pub attribution: Attribution,
    pub auth_enabled: bool,
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

    Response::from_json(&config_resp)?.add_header(
        "Cache-Control",
        &format!("public, max-age={}", config::CACHE_TTL_CONFIG),
    )
}

pub async fn handle_items(req: Request, _env: Env) -> Result<Response> {
    let url = req.url()?;
    let mut year_param = None;
    let mut season_param = None;

    for (k, v) in url.query_pairs() {
        match k.as_ref() {
            "year" => year_param = Some(v),
            "season" => season_param = Some(v),
            _ => {}
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

    let items = provider::season::fetch_items(target_year, target_season).await?;
    Response::from_json(&items)
}

pub async fn handle_user_status(mut req: Request, env: Env) -> Result<Response> {
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
                    Response::from_json(&status_map)
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

pub async fn handle_metadata(req: Request, env: Env) -> Result<Response> {
    let url = req.url()?;
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
