use super::MetadataProvider;
use crate::model;
use regex::Regex;
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
    pub number_of_seasons: Option<i32>, // Total seasons (from main show)
    pub season_number: Option<i32>,     // Current season number (from season endpoint)
    pub overview: Option<String>,
    pub runtime: Option<i32>,
    #[serde(default)]
    pub genres: Vec<TmdbGenre>,
    #[serde(default)]
    pub production_companies: Vec<TmdbCompany>,
    pub credits: Option<TmdbCredits>,
    pub episodes: Option<Vec<TmdbBriefEpisode>>,
    pub status: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TmdbBriefEpisode {
    pub episode_number: i32,
    pub name: Option<String>,
    pub air_date: Option<String>,
    pub overview: Option<String>,
    pub runtime: Option<i32>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TmdbGenre {
    pub name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TmdbCompany {
    pub name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TmdbCredits {
    #[serde(default)]
    pub cast: Vec<TmdbCastMember>,
    #[serde(default)]
    pub crew: Vec<TmdbCrewMember>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TmdbCastMember {
    pub character: String,
    pub name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TmdbCrewMember {
    pub name: String,
    pub job: String,
    pub department: Option<String>,
}

/// Parsed TMDb ID components
enum ParsedId {
    /// TV show with show_id and season number
    Tv { show_id: String, season: i32 },
    /// Movie or other media type
    Other(String),
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

        // Parse ID into components: (media_type, show_id, season_num)
        let parsed = Self::parse_tmdb_id(trimmed_id);

        match parsed {
            // Movie or non-TV: fetch directly
            ParsedId::Other(id) => self.get_media_raw(&id).await,

            // TV show: fetch season data and merge with parent
            ParsedId::Tv { show_id, season } => {
                let main_id = format!("tv/{}", show_id);
                let season_id = format!("tv/{}/season/{}", show_id, season);

                let main_media = self.get_media_raw(&main_id).await?;
                let mut season_media = self.get_media_raw(&season_id).await?;

                // Merge missing fields from parent
                Self::merge_from_parent(&mut season_media, &main_media);

                Ok(season_media)
            }
        }
    }

    /// Parse TMDb ID into structured components
    fn parse_tmdb_id(id: &str) -> ParsedId {
        // Strip episode part if present (e.g., "tv/123/season/1/episode/5" -> "tv/123/season/1")
        let id = id.split("/episode/").next().unwrap_or(id);

        let parts: Vec<&str> = id.split('/').collect();

        // Check if it's a TV show
        if parts.first() == Some(&"tv") && parts.len() >= 2 {
            let show_id = parts[1].to_string();

            // Extract season number, default to 1 if not specified
            let season = if parts.len() >= 4 && parts[2] == "season" {
                parts[3].parse().unwrap_or(1)
            } else {
                1 // Default to season 1 for plain TV IDs
            };

            ParsedId::Tv { show_id, season }
        } else {
            ParsedId::Other(id.to_string())
        }
    }

    /// Merge missing metadata from parent show into season media
    fn merge_from_parent(season: &mut TmdbMedia, parent: &TmdbMedia) {
        if season.genres.is_empty() {
            season.genres = parent.genres.clone();
        }
        if season.production_companies.is_empty() {
            season.production_companies = parent.production_companies.clone();
        }
        if season.vote_average.is_none() || season.vote_average == Some(0.0) {
            season.vote_average = parent.vote_average;
        }
        if season
            .overview
            .as_ref()
            .map(|s| s.is_empty())
            .unwrap_or(true)
        {
            season.overview = parent.overview.clone();
        }
        if season.credits.is_none() {
            season.credits = parent.credits.clone();
        }
        if season.status.is_none() {
            season.status = parent.status.clone();
        }
        if season.poster_path.is_none() {
            season.poster_path = parent.poster_path.clone();
        }
        // Always inherit total seasons count from parent
        season.number_of_seasons = parent.number_of_seasons;

        // Combine names if different
        if let (Some(p_name), Some(s_name)) = (&parent.name, &season.name) {
            if p_name != s_name {
                season.name = Some(format!("{}: {}", p_name, s_name));
            }
        }
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
        // Try original title first
        if let Ok(id) = self.search_single(title, year).await {
            return Ok(id);
        }

        // Try normalized title (e.g. adding spaces around dashes)
        let normalized = Self::normalize_title(title);
        if normalized != title {
            if let Ok(id) = self.search_single(&normalized, year).await {
                return Ok(id);
            }
        }

        Err(Error::RustError("No suitable match".into()))
    }

    async fn search_single(&self, title: &str, year: Option<i32>) -> Result<String> {
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
            let mut close_match = None;

            for res in &body.results {
                if res.media_type != "tv" && res.media_type != "movie" {
                    continue;
                }
                let date = res.first_air_date.as_ref().or(res.release_date.as_ref());

                if let Some(d) = date {
                    // Try to parse year from YYYY-MM-DD
                    if let Ok(res_year) = d.split('-').next().unwrap_or("").parse::<i32>() {
                        // Exact match
                        if res_year == y {
                            return Ok(format!("{}/{}", res.media_type, res.id));
                        }

                        // Close match (+/- 1 year), save the first one found as fallback
                        if (res_year - y).abs() <= 1 && close_match.is_none() {
                            close_match = Some(format!("{}/{}", res.media_type, res.id));
                        }
                    }
                }
            }

            // If no exact match, return close match if found
            if let Some(m) = close_match {
                return Ok(m);
            }

            return Err(Error::RustError(format!(
                "No match found for {} ({})",
                title, y
            )));
        }

        // 2. Fallback to first suitable match (only if year is None)
        for res in body.results {
            if res.media_type == "tv" || res.media_type == "movie" {
                return Ok(format!("{}/{}", res.media_type, res.id));
            }
        }
        Err(Error::RustError("No suitable match".into()))
    }

    fn normalize_title(title: &str) -> String {
        // 1. Add spaces around dashes: "Text-Text" -> "Text - Text"
        let mut normalized = title.replace("-", " - ");

        // 2. Remove season suffixes (e.g., "第2期", "Season 2", "2nd Season")
        // Use a static regex to avoid recompiling on every call (though in worker environment it might recompile per request)
        // Patterns to match:
        // - 第\d+期 (e.g., 第2期)
        // - 第\d+クール (e.g., 第2クール)
        // - Season \d+ (e.g., Season 2)
        // - \d+(st|nd|rd|th) Season (e.g., 2nd Season)
        // - Ⅱ, Ⅲ, Ⅳ, Ⅴ, Ⅵ, Ⅶ, Ⅷ, Ⅸ, Ⅹ (Roman numerals commonly used for seasons)
        if let Ok(re) = Regex::new(
            r"(?i)(\s*第\d+期|\s*第\d+クール|\s*Season\s*\d+|\s*\d+(st|nd|rd|th)\s*Season|\s*[ⅡⅢⅣⅤⅥⅦⅧⅨⅩ]+\s*)$",
        ) {
            normalized = re.replace(&normalized, "").to_string();
        }

        // 3. Remove year in parentheses: (2025)
        if let Ok(re) = Regex::new(r"\s*\(\d{4}\)\s*$") {
            normalized = re.replace(&normalized, "").to_string();
        }

        // 4. Clean up extra spaces
        normalized.replace("  ", " ").trim().to_string()
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
        english: None,
        native: media
            .name
            .or(media.title)
            .or(media.original_name)
            .or(media.original_title),
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
                    runtime: e.runtime,
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
        total_seasons: media.number_of_seasons,
        current_season: media.season_number,
        runtime: media.runtime,
    }
}
