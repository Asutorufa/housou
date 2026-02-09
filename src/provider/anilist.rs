use super::MetadataProvider;
use crate::model;
use serde::Deserialize;
use worker::*;

pub struct AnilistProvider;

impl MetadataProvider for AnilistProvider {
    async fn fetch(
        &self,
        _id: Option<&str>,
        title: Option<&str>,
        _year: Option<i32>,
    ) -> Result<model::UnifiedMetadata> {
        let title = title.ok_or_else(|| Error::RustError("Title required for AniList".into()))?;

        let gql_query = serde_json::json!({
            "query": r#"
                query ($search: String) {
                    Media(search: $search, type: ANIME) {
                        id
                        title { romaji english native }
                        coverImage { large extraLarge }
                        averageScore
                        episodes
                        status
                        genres
                        description(asHtml: false)
                        studios(isMain: true) { nodes { name } }
                        characters(sort: ROLE, perPage: 6) {
                            edges {
                                role
                                node { name { full } }
                                voiceActors(language: JAPANESE) { name { full } }
                            }
                        }
                    }
                }
            "#,
            "variables": { "search": title }
        });

        let mut init = RequestInit::new();
        init.with_method(Method::Post);
        init.with_body(Some(gql_query.to_string().into()));

        let mut cf = CfProperties::new();
        let mut ttl_by_status = std::collections::HashMap::new();
        ttl_by_status.insert("200".to_string(), crate::config::CACHE_TTL_ONGOING);
        cf.cache_ttl_by_status = Some(ttl_by_status);
        init.with_cf_properties(cf);

        let headers = Headers::new();
        headers.set("Content-Type", "application/json")?;
        init.with_headers(headers);

        let request = Request::new_with_init("https://graphql.anilist.co", &init)?;
        let mut anilist_response = Fetch::Request(request).send().await?;

        if anilist_response.status_code() != 200 {
            return Err(Error::RustError(format!(
                "AniList API error: {}",
                anilist_response.status_code()
            )));
        }

        let body: serde_json::Value = anilist_response.json().await?;
        let media_json = &body["data"]["Media"];
        if media_json.is_null() {
            return Err(Error::RustError("AniList: Not Found".into()));
        }

        let media: AniListMedia = serde_json::from_value(media_json.clone())
            .map_err(|e| Error::RustError(format!("AniList deserialization error: {}", e)))?;

        Ok(anilist_to_unified(media))
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct AniListMedia {
    pub id: i64,
    pub title: Option<AniListTitle>,
    #[serde(rename = "coverImage")]
    pub cover_image: Option<AniListCoverImage>,
    #[serde(rename = "averageScore")]
    pub average_score: Option<i32>,
    pub episodes: Option<i32>,
    pub status: Option<String>,
    pub genres: Option<Vec<serde_json::Value>>,
    pub description: Option<String>,
    pub studios: Option<AniListStudios>,
    pub characters: Option<AniListCharacters>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AniListTitle {
    pub romaji: Option<String>,
    pub english: Option<String>,
    pub native: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AniListCoverImage {
    pub large: Option<String>,
    #[serde(rename = "extraLarge")]
    pub extra_large: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AniListStudios {
    pub nodes: Option<Vec<AniListStudioNode>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AniListStudioNode {
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AniListCharacters {
    pub edges: Option<Vec<AniListCharacterEdge>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AniListCharacterEdge {
    pub role: Option<String>,
    pub node: Option<AniListCharacterNode>,
    #[serde(rename = "voiceActors")]
    pub voice_actors: Option<Vec<AniListVoiceActor>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AniListCharacterNode {
    pub name: Option<AniListName>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AniListName {
    pub full: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AniListVoiceActor {
    pub name: Option<AniListName>,
}

pub fn anilist_to_unified(media: AniListMedia) -> model::UnifiedMetadata {
    use model::*;

    let title = match media.title {
        Some(t) => UniversalTitle {
            romaji: t.romaji,
            english: t.english,
            native: t.native,
        },
        None => UniversalTitle::default(),
    };

    let cover_image = match media.cover_image {
        Some(c) => UniversalCoverImage {
            large: c.large,
            extra_large: c.extra_large,
        },
        None => UniversalCoverImage::default(),
    };

    let mut genres = Vec::new();
    if let Some(arr) = media.genres {
        for g in arr {
            if let Some(s) = g.as_str() {
                genres.push(s.to_string());
            }
        }
    }

    let mut studios = Vec::new();
    if let Some(nodes) = media.studios.and_then(|s| s.nodes) {
        for s in nodes {
            if let Some(name) = s.name {
                studios.push(name);
            }
        }
    }

    let mut characters = Vec::new();
    if let Some(edges) = media.characters.and_then(|c| c.edges) {
        for edge in edges {
            characters.push(UniversalCharacter {
                name: edge
                    .node
                    .as_ref()
                    .and_then(|n| n.name.as_ref())
                    .and_then(|n| n.full.clone())
                    .unwrap_or_default(),
                voice_actor: edge
                    .voice_actors
                    .as_ref()
                    .and_then(|v| v.first())
                    .and_then(|v| v.name.as_ref())
                    .and_then(|n| n.full.clone()),
                role: edge.role,
            });
        }
    }

    UnifiedMetadata {
        id: media.id.to_string(),
        title,
        cover_image,
        average_score: media.average_score,
        episodes: media.episodes,
        genres,
        description: media.description,
        studios,
        characters,
        staff: vec![],
        episodes_list: vec![],
        is_finished: media
            .status
            .map(|s| s == "FINISHED" || s == "CANCELLED")
            .unwrap_or(false),
        total_seasons: None, // AniList doesn't have seasons concept
        current_season: None,
        runtime: None,
        content_rating: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_anilist_to_unified_full() {
        let media_json = json!({
            "id": 12345,
            "title": {
                "romaji": "Test Anime",
                "english": "Test Anime English",
                "native": "テストアニメ"
            },
            "coverImage": {
                "large": "https://example.com/large.jpg",
                "extraLarge": "https://example.com/xlarge.jpg"
            },
            "averageScore": 85,
            "episodes": 12,
            "status": "FINISHED",
            "genres": ["Action", "Adventure"],
            "description": "This is a test description.",
            "studios": {
                "nodes": [
                    { "name": "Test Studio" }
                ]
            },
            "characters": {
                "edges": [
                    {
                        "role": "MAIN",
                        "node": {
                            "name": { "full": "Test Character" }
                        },
                        "voiceActors": [
                            { "name": { "full": "Test Voice Actor" } }
                        ]
                    }
                ]
            }
        });

        let media: AniListMedia = serde_json::from_value(media_json).unwrap();
        let unified = anilist_to_unified(media);

        assert_eq!(unified.id, "12345");
        assert_eq!(unified.title.romaji, Some("Test Anime".to_string()));
        assert_eq!(unified.title.english, Some("Test Anime English".to_string()));
        assert_eq!(unified.title.native, Some("テストアニメ".to_string()));
        assert_eq!(unified.cover_image.large, Some("https://example.com/large.jpg".to_string()));
        assert_eq!(unified.cover_image.extra_large, Some("https://example.com/xlarge.jpg".to_string()));
        assert_eq!(unified.average_score, Some(85));
        assert_eq!(unified.episodes, Some(12));
        assert!(unified.is_finished);
        assert_eq!(unified.genres, vec!["Action".to_string(), "Adventure".to_string()]);
        assert_eq!(unified.description, Some("This is a test description.".to_string()));
        assert_eq!(unified.studios, vec!["Test Studio".to_string()]);
        assert_eq!(unified.characters.len(), 1);
        assert_eq!(unified.characters[0].name, "Test Character");
        assert_eq!(unified.characters[0].role, Some("MAIN".to_string()));
        assert_eq!(unified.characters[0].voice_actor, Some("Test Voice Actor".to_string()));

        // Check fields that are always empty/None for AniList
        assert!(unified.staff.is_empty());
        assert!(unified.episodes_list.is_empty());
        assert_eq!(unified.total_seasons, None);
        assert_eq!(unified.current_season, None);
        assert_eq!(unified.runtime, None);
        assert_eq!(unified.content_rating, None);
    }

    #[test]
    fn test_anilist_to_unified_minimal() {
        let media_json = json!({
            "id": 67890,
            "title": {
                "romaji": "Minimal Anime"
            },
            "coverImage": {},
            "status": "RELEASING"
            // other fields missing
        });

        let media: AniListMedia = serde_json::from_value(media_json).unwrap();
        let unified = anilist_to_unified(media);

        assert_eq!(unified.id, "67890");
        assert_eq!(unified.title.romaji, Some("Minimal Anime".to_string()));
        assert_eq!(unified.title.english, None);
        assert_eq!(unified.title.native, None);
        assert_eq!(unified.cover_image.large, None);
        assert_eq!(unified.average_score, None);
        assert_eq!(unified.episodes, None);
        assert!(unified.genres.is_empty());
        assert_eq!(unified.description, None);
        assert!(unified.studios.is_empty());
        assert!(unified.characters.is_empty());
        assert!(!unified.is_finished); // RELEASING -> false
    }

    #[test]
    fn test_anilist_to_unified_arrays() {
        let media_json = json!({
            "id": 111,
            "title": { "romaji": "Array Test" },
            "genres": ["Action", null, 123, "Comedy"], // mixed types
            "studios": {
                "nodes": [
                    { "name": "Studio A" },
                    { "name": null }, // Should be skipped
                    { "missing_name": "Studio B" } // Should be skipped
                ]
            },
            "characters": {
                "edges": [
                    {
                        "role": "MAIN",
                        "node": { "name": { "full": "Char 1" } },
                        "voiceActors": [] // Empty voice actors
                    },
                    {
                        "role": "SUPPORTING",
                        "node": { "name": { "full": "Char 2" } },
                        "voiceActors": null // Null voice actors
                    },
                    {
                        "role": null,
                        "node": null, // malformed node
                        "voiceActors": [ { "name": { "full": "Actor 3" } } ]
                    }
                ]
            }
        });

        let media: AniListMedia = serde_json::from_value(media_json).unwrap();
        let unified = anilist_to_unified(media);

        // Check genres
        assert_eq!(unified.genres, vec!["Action".to_string(), "Comedy".to_string()]);

        // Check studios
        assert_eq!(unified.studios, vec!["Studio A".to_string()]);

        // Check characters
        assert_eq!(unified.characters.len(), 3);

        // Char 1: empty voice actor
        assert_eq!(unified.characters[0].name, "Char 1");
        assert_eq!(unified.characters[0].voice_actor, None);

        // Char 2: null voice actor
        assert_eq!(unified.characters[1].name, "Char 2");
        assert_eq!(unified.characters[1].voice_actor, None);

        // Char 3: malformed node (null) -> name becomes empty string
        assert_eq!(unified.characters[2].name, "");
        assert_eq!(unified.characters[2].role, None);
        assert_eq!(unified.characters[2].voice_actor, Some("Actor 3".to_string()));
    }

    #[test]
    fn test_anilist_deserialization_error() {
        // Case 1: ID is missing completely (field required)
        let media_no_id = json!({
            "title": { "romaji": "No ID Anime" },
            "status": "RELEASING"
        });
        let res: Result<AniListMedia, _> = serde_json::from_value(media_no_id);
        assert!(res.is_err());

        // Case 2: ID is not a number
        let media_bad_id = json!({
            "id": "not-a-number",
            "title": { "romaji": "Bad ID Anime" },
            "status": "RELEASING"
        });
        let res: Result<AniListMedia, _> = serde_json::from_value(media_bad_id);
        assert!(res.is_err());
    }
}
