use super::MetadataProvider;
use crate::model;
use serde::Deserialize;
use worker::*;

#[derive(Deserialize, Debug)]
struct TmdbSearchResponse {
    results: Vec<TmdbSearchResult>,
}

#[derive(Deserialize, Debug)]
struct TmdbSearchResult {
    id: i64,
    media_type: String,
    release_date: Option<String>,   // Movies
    first_air_date: Option<String>, // TV
}

#[derive(Deserialize, Debug)]
pub struct TmdbMedia {
    pub name: Option<String>,           // TV
    pub title: Option<String>,          // Movie
    pub original_name: Option<String>,  // TV
    pub original_title: Option<String>, // Movie
    pub poster_path: Option<String>,
    pub vote_average: Option<f64>,
    pub number_of_episodes: Option<i32>,
    pub overview: Option<String>,
    #[serde(default)]
    pub genres: Vec<TmdbGenre>,
    #[serde(default)]
    pub production_companies: Vec<TmdbCompany>,
    pub credits: Option<TmdbCredits>,
    pub episodes: Option<Vec<TmdbBriefEpisode>>, // New field for seasons
    pub status: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct TmdbBriefEpisode {
    pub episode_number: i32,
    pub name: Option<String>,
    pub air_date: Option<String>,
    pub overview: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct TmdbGenre {
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct TmdbCompany {
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct TmdbCredits {
    #[serde(default)]
    pub cast: Vec<TmdbCastMember>,
    #[serde(default)]
    pub crew: Vec<TmdbCrewMember>,
}

#[derive(Deserialize, Debug)]
pub struct TmdbCastMember {
    pub character: String,
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct TmdbCrewMember {
    pub name: String,
    pub job: String,
    pub department: Option<String>,
}

pub struct TmdbClient {
    api_token: String,
    is_v3: bool,
}

impl TmdbClient {
    pub fn new(env: &Env) -> Result<Self> {
        let api_token = env
            .secret("TMDB_TOKEN")
            .map(|s| s.to_string())
            .or_else(|_| env.var("TMDB_TOKEN").map(|s| s.to_string()))
            .map_err(|_| Error::RustError("TMDB_TOKEN not set".into()))?;

        let is_v3 = api_token.len() == 32;
        Ok(Self { api_token, is_v3 })
    }

    fn build_request(&self, path: &str, query: &str) -> Result<Request> {
        let mut url = format!("https://api.themoviedb.org/3/{}?{}", path, query);
        if self.is_v3 {
            url.push_str(&format!("&api_key={}", self.api_token));
        }

        let mut init = RequestInit::new();
        init.with_method(Method::Get);

        if !self.is_v3 {
            let headers = Headers::new();
            headers.set("Authorization", &format!("Bearer {}", self.api_token))?;
            init.with_headers(headers);
        }

        let mut cf = CfProperties::new();
        let mut ttl_by_status = std::collections::HashMap::new();
        ttl_by_status.insert("200".to_string(), 604800); // 1 week default edge cache
        cf.cache_ttl_by_status = Some(ttl_by_status);
        init.with_cf_properties(cf);

        Request::new_with_init(&url, &init)
    }

    pub async fn get_media(&self, id: &str) -> Result<TmdbMedia> {
        let trimmed_id = id.trim_start_matches('/');
        // If it's a season ID like "tv/123/season/1" or "tv/123-slug/season/1", merge with main show
        if trimmed_id.contains("/season/") {
            let parts: Vec<&str> = trimmed_id.split('/').collect();
            if parts.len() >= 2 && parts[0] == "tv" {
                // TMDb accepts full slugs like "82046-double-decker"
                let main_id = format!("tv/{}", parts[1]);

                // Fetch both parent and season sequentially to avoid dependency on futures crate
                let main_media = self.get_media_raw(&main_id).await?;
                let mut season_media = self.get_media_raw(trimmed_id).await?;

                // Merge critical metadata from parent - season endpoints lack most info
                if season_media.genres.is_empty() {
                    season_media.genres = main_media.genres;
                }
                if season_media.production_companies.is_empty() {
                    season_media.production_companies = main_media.production_companies;
                }
                // Season endpoints often return 0.0 for vote_average instead of null
                if season_media.vote_average.is_none() || season_media.vote_average == Some(0.0) {
                    season_media.vote_average = main_media.vote_average;
                }
                // Overview/description from parent if season has none
                if season_media.overview.is_none()
                    || season_media
                        .overview
                        .as_ref()
                        .map(|s| s.is_empty())
                        .unwrap_or(false)
                {
                    season_media.overview = main_media.overview;
                }
                // Credits from parent - season credits are usually empty
                if season_media.credits.is_none() {
                    season_media.credits = main_media.credits;
                }
                // Status from parent
                if season_media.status.is_none() {
                    season_media.status = main_media.status;
                }

                // Ensure name consists of both if useful
                if let (Some(m_name), Some(s_name)) = (&main_media.name, &season_media.name) {
                    if m_name != s_name {
                        season_media.name = Some(format!("{}: {}", m_name, s_name));
                    }
                }

                return Ok(season_media);
            }
        }

        self.get_media_raw(id).await
    }

    async fn get_media_raw(&self, id: &str) -> Result<TmdbMedia> {
        let request = self.build_request(id, "append_to_response=credits&language=ja-JP")?;
        let mut response = Fetch::Request(request).send().await?;
        if response.status_code() != 200 {
            return Err(Error::RustError(format!(
                "TMDb API status {} for {}",
                response.status_code(),
                id
            )));
        }
        response.json().await
    }

    pub async fn search(&self, title: &str, year: Option<i32>) -> Result<String> {
        let query = format!(
            "query={}&language=ja-JP&page=1&include_adult=false",
            encode_uri(title)
        );
        let request = self.build_request("search/multi", &query)?;
        let mut response = Fetch::Request(request).send().await?;
        if response.status_code() != 200 {
            return Err(Error::RustError("TMDb search failed".into()));
        }
        let body: TmdbSearchResponse = response.json().await?;

        // 1. Try to find an exact year match if year is provided
        if let Some(y) = year {
            for res in &body.results {
                if res.media_type != "tv" && res.media_type != "movie" {
                    continue;
                }
                let date = res.first_air_date.as_ref().or(res.release_date.as_ref());
                if let Some(d) = date {
                    if d.starts_with(&y.to_string()) {
                        return Ok(format!("{}/{}", res.media_type, res.id));
                    }
                }
            }
        }

        // 2. Fallback to first suitable match
        for res in body.results {
            if res.media_type == "tv" || res.media_type == "movie" {
                return Ok(format!("{}/{}", res.media_type, res.id));
            }
        }
        Err(Error::RustError("No suitable match".into()))
    }
}

pub struct TmdbProvider<'a> {
    env: &'a Env,
}

impl<'a> TmdbProvider<'a> {
    pub fn new(env: &'a Env) -> Self {
        Self { env }
    }
}

impl<'a> MetadataProvider for TmdbProvider<'a> {
    async fn fetch(
        &self,
        id: Option<&str>,
        title: Option<&str>,
        year: Option<i32>,
    ) -> Result<model::UnifiedMetadata> {
        let client = TmdbClient::new(self.env)?;
        let media_id = if let Some(id) = id {
            id.to_string()
        } else if let Some(search_title) = title {
            client.search(search_title, year).await?
        } else {
            return Err(Error::RustError("ID or Title required".into()));
        };

        let media = client.get_media(&media_id).await?;
        Ok(tmdb_to_unified(media, &media_id))
    }
}

fn encode_uri(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{:02X}", b),
        })
        .collect()
}

pub fn tmdb_to_unified(media: TmdbMedia, id: &str) -> model::UnifiedMetadata {
    use model::*;

    let title = UniversalTitle {
        romaji: None,
        english: media.name.clone().or(media.title.clone()),
        native: media.original_name.or(media.original_title),
    };

    let poster_path = media.poster_path;
    let cover_image = UniversalCoverImage {
        large: poster_path
            .as_ref()
            .map(|p| format!("https://image.tmdb.org/t/p/w500{}", p)),
        extra_large: poster_path
            .as_ref()
            .map(|p| format!("https://image.tmdb.org/t/p/original{}", p)),
    };

    let genres = media.genres.into_iter().map(|g| g.name).collect();
    let studios = media
        .production_companies
        .into_iter()
        .map(|s| s.name)
        .collect();

    let mut characters = Vec::new();
    let mut staff = Vec::new();
    if let Some(credits) = media.credits {
        for member in credits.cast.into_iter().take(6) {
            characters.push(UniversalCharacter {
                name: member.character,
                voice_actor: Some(member.name),
                role: Some("Cast".to_string()),
            });
        }
        for member in credits.crew.into_iter().take(10) {
            staff.push(UniversalStaff {
                name: member.name,
                role: member.job,
                department: member.department,
            });
        }
    }

    let episodes_list = media
        .episodes
        .map(|list| {
            list.into_iter()
                .map(|e| UniversalEpisode {
                    number: e.episode_number,
                    title: e.name,
                    air_date: e.air_date,
                    overview: e.overview,
                })
                .collect()
        })
        .unwrap_or_default();

    UnifiedMetadata {
        id: id.to_string(),
        title,
        cover_image,
        average_score: media.vote_average.map(|v| (v * 10.0) as i32),
        episodes: media.number_of_episodes,
        genres,
        description: media.overview,
        studios,
        characters,
        staff,
        episodes_list,
        is_finished: media
            .status
            .map(|s| s == "Ended" || s == "Canceled" || s == "Released")
            .unwrap_or(false),
    }
}
