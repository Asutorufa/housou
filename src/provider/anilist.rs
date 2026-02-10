use super::MetadataProvider;
use crate::model;
use std::sync::OnceLock;
use worker::*;

pub struct AnilistProvider;

static ANILIST_CLIENT: OnceLock<rust_anilist::Client> = OnceLock::new();

impl MetadataProvider for AnilistProvider {
    async fn fetch(
        &self,
        id: Option<&str>,
        title: Option<&str>,
        _year: Option<i32>,
    ) -> Result<model::UnifiedMetadata> {
        let client = ANILIST_CLIENT.get_or_init(rust_anilist::Client::default);

        let anime = if let Some(i) = id {
            // AniList ID must be an integer
            let anime_id = i
                .parse::<i64>()
                .map_err(|e| Error::RustError(format!("Invalid AniList ID: {}", e)))?;
            client
                .get_anime(anime_id)
                .await
                .map_err(|e| Error::RustError(format!("AniList API error (get_anime): {}", e)))?
        } else if let Some(t) = title {
            let results = client.search_anime(t, 1, 1).await;

            // rust-anilist search_anime returns Option<Vec<Anime>>
            // and it might return None or an empty vector.
            match results.and_then(|v| v.into_iter().next()) {
                Some(anime) => anime,
                None => return Err(Error::RustError("AniList: Not Found".into())),
            }
        } else {
            return Err(Error::RustError(
                "AniList provider requires ID or Title".into(),
            ));
        };

        Ok(anilist_to_unified(anime))
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
        .map(|s| s.name)
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
        id: media.id.to_string(),
        title,
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_anilist::models::Anime;
    use serde_json::json;

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
                            "name": { "full": "Test Character" },
                            "image": { "large": "", "medium": "" },
                            "description": "",
                            "siteUrl": ""
                        },
                        "role": "MAIN",
                        "voiceActors": [
                            {
                                "id": 1,
                                "name": { "full": "Test Voice Actor" },
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
            "staff": { "nodes": [] }, // Required field? (Wait, staff is Option<Vec<Person>> with custom deserializer)
            "isFavourite": false,
            "isFavouriteBlocked": false
        });

        let anime = create_anime_from_json(anime_json);
        let unified = anilist_to_unified(anime);

        assert_eq!(unified.id, "12345");
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
        assert_eq!(unified.studios, vec!["Test Studio".to_string()]);
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

        assert_eq!(unified.id, "67890");
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
}
