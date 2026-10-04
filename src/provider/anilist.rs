use super::MetadataProvider;
use super::match_score::{self, MediaKind};
use crate::model;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::sync::OnceLock;
use worker::wasm_bindgen::JsValue;
use worker::*;

pub struct AnilistProvider;

static ANILIST_CLIENT: OnceLock<rust_anilist::Client> = OnceLock::new();

const ANILIST_API: &str = "https://graphql.anilist.co/";
const SEARCH_TITLE_LIMIT: usize = 4;
const SEARCH_RESULT_LIMIT: i32 = 10;

const SEARCH_QUERY: &str = r#"
query ($search: String!, $page: Int!, $perPage: Int!) {
  Page(page: $page, perPage: $perPage) {
    media(search: $search, type: ANIME, sort: SEARCH_MATCH) {
      id
      idMal
      title { romaji english native }
      format
      seasonYear
      startDate { year }
    }
  }
}
"#;

const MAL_ID_QUERY: &str = r#"
query ($idMal: Int!) {
  Media(idMal: $idMal, type: ANIME) {
    id
  }
}
"#;

#[derive(Debug, Deserialize)]
struct GraphQlResponse<T> {
    data: Option<T>,
    #[serde(default)]
    errors: Vec<GraphQlError>,
}

#[derive(Debug, Deserialize)]
struct GraphQlError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct SearchData {
    #[serde(rename = "Page")]
    page: SearchPage,
}

#[derive(Debug, Deserialize)]
struct SearchPage {
    #[serde(default)]
    media: Vec<SearchMedia>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchMedia {
    id: i64,
    #[allow(dead_code)]
    id_mal: Option<i64>,
    title: SearchTitle,
    format: Option<String>,
    season_year: Option<i32>,
    start_date: Option<SearchDate>,
}

#[derive(Debug, Deserialize)]
struct SearchTitle {
    romaji: Option<String>,
    english: Option<String>,
    native: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SearchDate {
    year: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct MalLookupData {
    #[serde(rename = "Media")]
    media: Option<IdOnly>,
}

#[derive(Debug, Deserialize)]
struct IdOnly {
    id: i64,
}

impl AnilistProvider {
    fn client(&self) -> &'static rust_anilist::Client {
        ANILIST_CLIENT.get_or_init(rust_anilist::Client::default)
    }

    pub async fn fetch_by_mal_id(&self, mal_id: &str) -> Result<model::UnifiedMetadata> {
        let mal_id = mal_id
            .parse::<i64>()
            .map_err(|error| Error::RustError(format!("Invalid MAL ID: {error}")))?;

        let data: MalLookupData =
            graphql(MAL_ID_QUERY, serde_json::json!({ "idMal": mal_id })).await?;
        let id = data
            .media
            .ok_or_else(|| Error::RustError("AniList: Not Found".into()))?
            .id;

        let anime =
            self.client().get_anime(id).await.map_err(|error| {
                Error::RustError(format!("AniList API error (get_anime): {error}"))
            })?;
        Ok(anilist_to_unified(anime))
    }

    async fn search(
        &self,
        title: &str,
        aliases: &[String],
        year: Option<i32>,
        media_type: Option<&str>,
    ) -> Result<model::UnifiedMetadata> {
        let expected_titles =
            match_score::title_candidates(Some(title), aliases, SEARCH_TITLE_LIMIT);
        if expected_titles.is_empty() {
            return Err(Error::RustError("AniList: empty search title".into()));
        }

        let expected_kind = MediaKind::from_request(media_type);
        let min_score = if year.is_none() && expected_kind.is_none() {
            90
        } else {
            105
        };
        let mut best: Option<(i32, i64)> = None;

        for search_title in &expected_titles {
            let data: SearchData = graphql(
                SEARCH_QUERY,
                serde_json::json!({
                    "search": search_title,
                    "page": 1,
                    "perPage": SEARCH_RESULT_LIMIT
                }),
            )
            .await?;

            for candidate in data.page.media {
                let score = score_candidate(&expected_titles, year, expected_kind, &candidate);
                if best
                    .as_ref()
                    .is_none_or(|(best_score, _)| score > *best_score)
                {
                    best = Some((score, candidate.id));
                }
            }

            if best.as_ref().is_some_and(|(score, _)| *score >= 150) {
                break;
            }
        }

        let id = match best {
            Some((score, id)) if score >= min_score => id,
            _ => return Err(Error::RustError("AniList: Not Found".into())),
        };

        let anime =
            self.client().get_anime(id).await.map_err(|error| {
                Error::RustError(format!("AniList API error (get_anime): {error}"))
            })?;
        Ok(anilist_to_unified(anime))
    }
}

impl MetadataProvider for AnilistProvider {
    async fn fetch(&self, query: super::LookupQuery<'_>) -> Result<model::UnifiedMetadata> {
        let anime = match query {
            super::LookupQuery::ById(id) => {
                let anime_id = id
                    .parse::<i64>()
                    .map_err(|error| Error::RustError(format!("Invalid AniList ID: {error}")))?;
                self.client().get_anime(anime_id).await.map_err(|error| {
                    Error::RustError(format!("AniList API error (get_anime): {error}"))
                })?
            }
            super::LookupQuery::ByTitle {
                title,
                aliases,
                year,
                media_type,
            } => {
                return self.search(title, aliases, year, media_type).await;
            }
        };

        Ok(anilist_to_unified(anime))
    }
}

async fn graphql<T: DeserializeOwned>(query: &str, variables: serde_json::Value) -> Result<T> {
    let body = serde_json::json!({
        "query": query,
        "variables": variables,
    });

    let headers = Headers::new();
    headers.set("Accept", "application/json")?;
    headers.set("Content-Type", "application/json")?;
    headers.set(
        "User-Agent",
        "housou/0.1.0 (https://github.com/Asutorufa/housou)",
    )?;

    let mut init = RequestInit::new();
    init.with_method(Method::Post);
    init.with_headers(headers);
    init.with_body(Some(JsValue::from_str(&body.to_string())));

    let request = Request::new_with_init(ANILIST_API, &init)?;
    let mut response = Fetch::Request(request).send().await?;
    if response.status_code() != 200 {
        return Err(Error::RustError(format!(
            "AniList GraphQL request failed with HTTP {}",
            response.status_code()
        )));
    }

    let response: GraphQlResponse<T> = response.json().await?;
    if let Some(data) = response.data {
        return Ok(data);
    }

    let message = response
        .errors
        .into_iter()
        .map(|error| error.message)
        .collect::<Vec<_>>()
        .join("; ");
    Err(Error::RustError(if message.is_empty() {
        "AniList: empty GraphQL response".into()
    } else {
        format!("AniList API error: {message}")
    }))
}

fn score_candidate(
    expected_titles: &[String],
    year: Option<i32>,
    expected_kind: Option<MediaKind>,
    candidate: &SearchMedia,
) -> i32 {
    let titles = [
        candidate.title.romaji.as_deref(),
        candidate.title.english.as_deref(),
        candidate.title.native.as_deref(),
    ];
    let actual_year = candidate
        .season_year
        .or_else(|| candidate.start_date.as_ref().and_then(|date| date.year));
    let actual_kind = anilist_media_kind(candidate.format.as_deref());

    match_score::title_score(expected_titles, titles.into_iter().flatten())
        + match_score::year_score(year, actual_year)
        + match_score::media_kind_score(expected_kind, actual_kind)
}

fn anilist_media_kind(format: Option<&str>) -> MediaKind {
    match format.unwrap_or("").to_ascii_uppercase().as_str() {
        "MOVIE" => MediaKind::Movie,
        "OVA" => MediaKind::Ova,
        "ONA" => MediaKind::Ona,
        "SPECIAL" => MediaKind::Special,
        "TV" | "TV_SHORT" => MediaKind::Tv,
        _ => MediaKind::Other,
    }
}

pub fn anilist_to_unified(media: rust_anilist::models::Anime) -> model::UnifiedMetadata {
    use model::*;

    let title = UniversalTitle {
        romaji: (!media.title.romaji().is_empty()).then(|| media.title.romaji().to_string()),
        english: (!media.title.english().is_empty()).then(|| media.title.english().to_string()),
        native: (!media.title.native().is_empty()).then(|| media.title.native().to_string()),
    };

    let cover_image = UniversalCoverImage {
        large: media.cover.large,
        extra_large: media.cover.extra_large,
    };

    let genres = media.genres.unwrap_or_default();

    let studios = media
        .studios
        .unwrap_or_default()
        .into_iter()
        .map(|s| model::Studio {
            name: s.name,
            logo_url: None,
        })
        .collect();

    let characters = media
        .characters
        .unwrap_or_default()
        .into_iter()
        .map(|c| {
            let voice_actor = c
                .voice_actors
                .as_ref()
                .and_then(|v| v.first())
                .map(|va| va.name.full.clone());

            UniversalCharacter {
                name: c.name.full.unwrap_or_default(),
                voice_actor: voice_actor.flatten(),
                role: c.role.map(|r| r.to_string()),
            }
        })
        .collect();

    let staff = media
        .staff
        .unwrap_or_default()
        .into_iter()
        .map(|s| model::UniversalStaff {
            name: s.name.full.unwrap_or_default(),
            role: "".to_string(),
            department: None,
        })
        .collect();

    let description = if media.description.is_empty() {
        None
    } else {
        Some(media.description)
    };

    UnifiedMetadata {
        source: MetadataSource::Anilist(media.id.to_string()),
        title,
        title_translate: None,
        cover_image,
        average_score: media.average_score.map(|s| s as i32),
        episodes: media.episodes.map(|e| e as i32),
        genres,
        description,
        studios,
        characters,
        staff,
        episodes_list: vec![],
        is_finished: matches!(
            media.status,
            rust_anilist::models::Status::Finished | rust_anilist::models::Status::Cancelled
        ),
        total_seasons: None,
        current_season: None,
        runtime: media.duration.map(|d| d as i32),
        content_rating: None,
        videos: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::MetadataSource;
    use rust_anilist::models::Anime;
    use serde_json::json;

    #[test]
    fn candidate_score_uses_alias_year_and_format() {
        let candidate = SearchMedia {
            id: 1,
            id_mal: Some(2),
            title: SearchTitle {
                romaji: Some("Test Anime".into()),
                english: None,
                native: Some("テストアニメ".into()),
            },
            format: Some("TV".into()),
            season_year: Some(2026),
            start_date: None,
        };
        let expected = vec!["テストアニメ".to_string()];
        assert!(score_candidate(&expected, Some(2026), Some(MediaKind::Tv), &candidate) >= 150);
    }

    // Helper to create an Anime struct via serde since fields are private or hard to construct
    fn create_anime_from_json(json: serde_json::Value) -> Anime {
        match serde_json::from_value(json.clone()) {
            Ok(anime) => anime,
            Err(e) => panic!("Failed to deserialize Anime: {}\nJSON: {}", e, json),
        }
    }

    #[test]
    fn test_anilist_to_unified_full() {
        let anime_json = json!({
            "id": 12345,
            "title": {
                "romaji": "Test Anime",
                "english": "Test Anime English",
                "native": "テストアニメ",
                "userPreferred": "Test Anime"
            },
            "format": "TV",
            "status": "FINISHED",
            "description": "This is a test description.",
            "coverImage": {
                "large": "https://example.com/large.jpg",
                "extraLarge": "https://example.com/xlarge.jpg"
            },
            "bannerImage": null,
            "averageScore": 85,
            "meanScore": 80,
            "episodes": 12,
            "duration": 24,
            "genres": ["Action", "Adventure"],
            "studios": {
                "nodes": [
                    { "name": "Test Studio", "id": 1, "isAnimationStudio": true, "url": "", "favourites": 0 }
                ]
            },
            "characters": {
                "edges": [
                    {
                        "node": {
                            "id": 1,
                            "name": { "full": "Test Character", "alternative": [] },
                            "image": { "large": "", "medium": "" },
                            "description": "",
                            "siteUrl": ""
                        },
                        "role": "MAIN",
                        "voiceActors": [
                            {
                                "id": 1,
                                "name": { "full": "Test Voice Actor", "alternative": [] },
                                "image": { "large": "", "medium": "" },
                                "description": "",
                                "siteUrl": "",
                                "languageV2": "Japanese",
                                "gender": "Male",
                                "favourites": 0
                            }
                        ]
                    }
                ]
            },
            "siteUrl": "https://anilist.co/anime/12345",
            "isAdult": false,
            "relations": { "edges": [] }, // Required field
            "staff": { "nodes": [] }, // Required field (key must be present, but value can be null)
            "isFavourite": false,
            "isFavouriteBlocked": false
        });

        let anime = create_anime_from_json(anime_json);
        let unified = anilist_to_unified(anime);

        assert_eq!(unified.source, MetadataSource::Anilist("12345".to_string()));
        assert_eq!(unified.title.romaji, Some("Test Anime".to_string()));
        assert_eq!(
            unified.title.english,
            Some("Test Anime English".to_string())
        );
        assert_eq!(unified.title.native, Some("テストアニメ".to_string()));
        assert_eq!(
            unified.cover_image.large,
            Some("https://example.com/large.jpg".to_string())
        );
        assert_eq!(
            unified.cover_image.extra_large,
            Some("https://example.com/xlarge.jpg".to_string())
        );
        assert_eq!(unified.average_score, Some(85));
        assert_eq!(unified.episodes, Some(12));
        assert!(unified.is_finished);
        assert_eq!(
            unified.genres,
            vec!["Action".to_string(), "Adventure".to_string()]
        );
        assert_eq!(
            unified.description,
            Some("This is a test description.".to_string())
        );
        assert_eq!(
            unified.studios,
            vec![model::Studio {
                name: "Test Studio".to_string(),
                logo_url: None
            }]
        );
        assert_eq!(unified.characters.len(), 1);
        assert_eq!(unified.characters[0].name, "Test Character");
        assert_eq!(unified.characters[0].role, Some("Main".to_string()));
        assert_eq!(
            unified.characters[0].voice_actor,
            Some("Test Voice Actor".to_string())
        );
        assert_eq!(unified.runtime, Some(24));
    }

    #[test]
    fn test_anilist_to_unified_minimal() {
        let anime_json = json!({
            "id": 67890,
            "title": {
                "romaji": "Minimal Anime",
                "native": "Minimal Anime"
            },
            "format": "TV",
            "status": "RELEASING",
            "description": "",
            "coverImage": {},
            "siteUrl": "https://anilist.co/anime/67890",
            "isAdult": false,
            "relations": { "edges": [] },
            "characters": { "edges": [] },
            "staff": { "nodes": [] },
            "studios": { "nodes": [] },
            "externalLinks": [],
            "streamingEpisodes": []
        });

        let anime = create_anime_from_json(anime_json);
        let unified = anilist_to_unified(anime);

        assert_eq!(unified.source, MetadataSource::Anilist("67890".to_string()));
        assert_eq!(unified.title.romaji, Some("Minimal Anime".to_string()));
        assert_eq!(unified.title.english, Some("Minimal Anime".to_string()));
        assert_eq!(unified.title.native, Some("Minimal Anime".to_string()));

        assert_eq!(unified.cover_image.large, None);
        assert_eq!(unified.average_score, None);
        assert_eq!(unified.episodes, None);
        assert!(unified.genres.is_empty());
        assert_eq!(unified.description, None); // Should be None
        assert!(unified.studios.is_empty());
        assert!(unified.characters.is_empty());
        assert!(!unified.is_finished); // RELEASING -> false
        assert_eq!(unified.runtime, None);
    }

    #[test]
    fn test_anilist_to_unified_with_null_staff() {
        let anime_json = json!({
            "id": 11111,
            "title": {
                "romaji": "Null Staff Anime",
                "native": "Null Staff Anime"
            },
            "format": "TV",
            "status": "FINISHED",
            "description": "",
            "coverImage": {},
            "siteUrl": "https://anilist.co/anime/11111",
            "isAdult": false,
            "relations": { "edges": [] },
            "characters": { "edges": [] },
            "staff": null,
            "studios": { "nodes": [] },
            "externalLinks": [],
            "streamingEpisodes": []
        });

        // This should not panic if staff is optional
        let anime = create_anime_from_json(anime_json);
        let unified = anilist_to_unified(anime);

        assert!(unified.staff.is_empty());
    }
}
