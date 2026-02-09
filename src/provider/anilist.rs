use super::MetadataProvider;
use crate::model;
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
        ttl_by_status.insert("200".to_string(), 604800); // 1 week default edge cache
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
        let media = &body["data"]["Media"];
        if media.is_null() {
            return Err(Error::RustError("AniList: Not Found".into()));
        }

        Ok(anilist_to_unified(media.clone()))
    }
}

pub fn anilist_to_unified(media: serde_json::Value) -> model::UnifiedMetadata {
    use model::*;

    let title = UniversalTitle {
        romaji: media["title"]["romaji"].as_str().map(|s| s.to_string()),
        english: media["title"]["english"].as_str().map(|s| s.to_string()),
        native: media["title"]["native"].as_str().map(|s| s.to_string()),
    };

    let cover_image = UniversalCoverImage {
        large: media["coverImage"]["large"].as_str().map(|s| s.to_string()),
        extra_large: media["coverImage"]["extraLarge"]
            .as_str()
            .map(|s| s.to_string()),
    };

    let mut genres = Vec::new();
    if let Some(arr) = media["genres"].as_array() {
        for g in arr {
            if let Some(s) = g.as_str() {
                genres.push(s.to_string());
            }
        }
    }

    let mut studios = Vec::new();
    if let Some(arr) = media["studios"]["nodes"].as_array() {
        for s in arr {
            if let Some(name) = s["name"].as_str() {
                studios.push(name.to_string());
            }
        }
    }

    let mut characters = Vec::new();
    if let Some(arr) = media["characters"]["edges"].as_array() {
        for edge in arr {
            characters.push(UniversalCharacter {
                name: edge["node"]["name"]["full"]
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
                voice_actor: edge["voiceActors"][0]["name"]["full"]
                    .as_str()
                    .map(|s| s.to_string()),
                role: edge["role"].as_str().map(|s| s.to_string()),
            });
        }
    }

    UnifiedMetadata {
        id: media["id"].as_i64().unwrap_or(0).to_string(),
        title,
        cover_image,
        average_score: media["averageScore"].as_i64().map(|v| v as i32),
        episodes: media["episodes"].as_i64().map(|v| v as i32),
        genres,
        description: media["description"].as_str().map(|s| s.to_string()),
        studios,
        characters,
        staff: vec![],
        episodes_list: vec![],
        is_finished: media["status"]
            .as_str()
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
        let media = json!({
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
        let media = json!({
            "id": 67890,
            "title": {
                "romaji": "Minimal Anime"
            },
            "coverImage": {},
            "status": "RELEASING"
            // other fields missing
        });

        let unified = anilist_to_unified(media);

        assert_eq!(unified.id, "67890");
        assert_eq!(unified.title.romaji, Some("Minimal Anime".to_string()));
        assert_eq!(unified.title.english, None);
        assert_eq!(unified.title.native, None);
        assert_eq!(unified.cover_image.large, None);
        assert_eq!(unified.average_score, None);
        assert_eq!(unified.episodes, None);
        assert_eq!(unified.genres.len(), 0);
        assert_eq!(unified.description, None);
        assert_eq!(unified.studios.len(), 0);
        assert_eq!(unified.characters.len(), 0);
        assert!(!unified.is_finished); // RELEASING -> false
    }

    #[test]
    fn test_anilist_to_unified_arrays() {
        let media = json!({
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
    fn test_anilist_to_unified_invalid_id() {
        // Case 1: ID is missing completely
        let media_no_id = json!({
            "title": { "romaji": "No ID Anime" },
            "status": "RELEASING"
        });
        assert_eq!(anilist_to_unified(media_no_id).id, "0");

        // Case 2: ID is not a number (e.g. string)
        // Note: The implementation uses `as_i64()`, which returns None if the value is not a number.
        let media_bad_id = json!({
            "id": "not-a-number",
            "title": { "romaji": "Bad ID Anime" },
            "status": "RELEASING"
        });
        assert_eq!(anilist_to_unified(media_bad_id).id, "0");
    }
}
