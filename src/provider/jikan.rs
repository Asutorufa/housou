use crate::model::{
    Item, ItemType, Language, MetadataSource, Site, Studio, TitleTranslate, UnifiedMetadata,
    UniversalCoverImage, UniversalTitle,
};
use crate::provider::MetadataProvider;
use crate::utils;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use worker::*;

#[derive(Debug, Deserialize)]
struct JikanResponse<T> {
    data: T,
}

#[derive(Debug, Deserialize)]
struct JikanAnime {
    mal_id: i32,
    url: String,
    images: HashMap<String, JikanImage>,
    title: String,
    title_english: Option<String>,
    title_japanese: Option<String>,
    #[serde(rename = "type")]
    type_field: Option<String>,
    episodes: Option<i32>,
    status: Option<String>,
    aired: JikanAired,
    score: Option<f64>,
    synopsis: Option<String>,
    #[allow(dead_code)]
    background: Option<String>,
    #[allow(dead_code)]
    season: Option<String>,
    #[allow(dead_code)]
    year: Option<i32>,
    broadcast: Option<JikanBroadcast>,
    studios: Vec<JikanEntity>,
    genres: Vec<JikanEntity>,
}

#[derive(Debug, Deserialize)]
struct JikanImage {
    image_url: Option<String>,
    large_image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct JikanAired {
    from: Option<String>,
    to: Option<String>,
}

#[derive(Debug, Deserialize)]
struct JikanBroadcast {
    string: Option<String>,
}

#[derive(Debug, Deserialize)]
struct JikanEntity {
    name: String,
}

pub struct JikanProvider;

impl MetadataProvider for JikanProvider {
    async fn fetch(&self, query: super::LookupQuery<'_>) -> Result<UnifiedMetadata> {
        let mal_id = match query {
            super::LookupQuery::ById(id) => id,
            super::LookupQuery::ByTitle { .. } => {
                return Err(Error::RustError("Jikan requires a MAL ID".into()));
            }
        };
        let url = format!("https://api.jikan.moe/v4/anime/{mal_id}/full");

        let response: JikanResponse<JikanAnime> = utils::fetch_json(&url)
            .await?
            .ok_or_else(|| Error::RustError("Jikan data not found".into()))?;

        let anime = response.data;
        Ok(convert_to_metadata(anime))
    }
}

pub async fn fetch_season(year: i32, season: &str) -> Result<Vec<Item>> {
    let url = format!("https://api.jikan.moe/v4/seasons/{year}/{season}");
    let response: Option<JikanResponse<Vec<JikanAnime>>> = utils::fetch_json(&url).await?;

    let items = response
        .map(|r| {
            let mut seen = HashSet::new();
            r.data
                .into_iter()
                .filter(|anime| seen.insert(anime.mal_id))
                .map(convert_to_item)
                .collect()
        })
        .unwrap_or_default();

    Ok(items)
}

fn convert_to_item(anime: JikanAnime) -> Item {
    let type_field = match anime.type_field.as_deref() {
        Some("TV") => ItemType::Tv,
        Some("Movie") => ItemType::Movie,
        Some("OVA") => ItemType::Ova,
        Some("ONA") => ItemType::Web,
        _ => ItemType::Tv,
    };

    let sites = vec![Site {
        site: "mal".to_string(),
        id: Some(anime.mal_id.to_string()),
        url: Some(anime.url.clone()),
        ..Default::default()
    }];

    // Prefer Japanese title as main title
    let (title, title_translate) = if let Some(ja_title) = &anime.title_japanese {
        if !ja_title.is_empty() {
            // Main title is Japanese
            let mut en = anime
                .title_english
                .clone()
                .map(|t| vec![t])
                .unwrap_or_default();
            if &anime.title != ja_title {
                en.push(anime.title.clone());
            }
            let mut tt = TitleTranslate::new();
            if !en.is_empty() {
                tt.insert("US".into(), en);
            }
            tt.insert("JP".into(), vec![ja_title.clone()]);
            (ja_title.clone(), tt)
        } else {
            let mut tt = TitleTranslate::new();
            if let Some(en) = anime.title_english.clone() {
                tt.insert("US".into(), vec![en]);
            }
            (anime.title.clone(), tt)
        }
    } else {
        let mut tt = TitleTranslate::new();
        if let Some(en) = anime.title_english.clone() {
            tt.insert("US".into(), vec![en]);
        }
        (anime.title.clone(), tt)
    };

    Item {
        title,
        title_translate,
        type_field,
        lang: Language::Ja,
        official_site: anime.url,
        begin: anime.aired.from,
        end: anime.aired.to,
        comment: anime.synopsis,
        sites,
        broadcast: anime.broadcast.and_then(|b| b.string),
    }
}

fn convert_to_metadata(anime: JikanAnime) -> UnifiedMetadata {
    let image = anime.images.get("jpg").or_else(|| anime.images.get("webp"));

    UnifiedMetadata {
        source: MetadataSource::Mal(anime.mal_id.to_string()),
        title: UniversalTitle {
            romaji: Some(anime.title),
            english: anime.title_english.clone(),
            native: anime.title_japanese.clone(),
        },
        title_translate: {
            let mut tt = TitleTranslate::new();
            if let Some(en) = anime.title_english {
                tt.insert("US".into(), vec![en]);
            }
            if let Some(ja) = anime.title_japanese {
                tt.insert("JP".into(), vec![ja]);
            }
            if tt.is_empty() { None } else { Some(tt) }
        },
        cover_image: UniversalCoverImage {
            large: image.and_then(|i| i.large_image_url.clone()),
            extra_large: image.and_then(|i| i.image_url.clone()), // Fallback
        },
        average_score: anime.score.map(|s| (s * 10.0) as i32),
        episodes: anime.episodes,
        genres: anime.genres.into_iter().map(|g| g.name).collect(),
        description: anime.synopsis,
        studios: anime
            .studios
            .into_iter()
            .map(|s| Studio {
                name: s.name,
                logo_url: None,
            })
            .collect(),
        characters: vec![],
        staff: vec![],
        episodes_list: vec![],
        is_finished: anime.status.as_deref() == Some("Finished Airing"),
        total_seasons: None,
        current_season: None,
        runtime: None,
        content_rating: None,
        videos: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::from_str;

    // Helper to create a minimal JikanAnime struct from JSON
    fn create_anime(json: &str) -> JikanAnime {
        from_str(json).expect("Failed to parse JSON")
    }

    #[test]
    fn test_convert_to_item_basic() {
        let json = r#"{
            "mal_id": 1,
            "url": "https://myanimelist.net/anime/1/Cowboy_Bebop",
            "images": { "jpg": { "image_url": "url", "large_image_url": "large_url" } },
            "title": "Cowboy Bebop",
            "title_english": "Cowboy Bebop",
            "title_japanese": "カウボーイビバップ",
            "type": "TV",
            "episodes": 26,
            "status": "Finished Airing",
            "aired": { "from": "1998-04-03T00:00:00+00:00", "to": "1999-04-24T00:00:00+00:00" },
            "score": 8.75,
            "synopsis": "In the year 2071...",
            "studios": [{ "name": "Sunrise" }],
            "genres": [{ "name": "Action" }]
        }"#;

        let anime = create_anime(json);
        let item = convert_to_item(anime);

        assert_eq!(item.title, "カウボーイビバップ");
        assert_eq!(item.type_field, ItemType::Tv);
        assert_eq!(
            item.official_site,
            "https://myanimelist.net/anime/1/Cowboy_Bebop"
        );
        assert_eq!(item.begin.as_deref(), Some("1998-04-03T00:00:00+00:00"));
        assert_eq!(item.end.as_deref(), Some("1999-04-24T00:00:00+00:00"));

        let translate = item.title_translate;
        assert_eq!(
            translate.get("JP"),
            Some(&vec!["カウボーイビバップ".to_string()])
        );
        // Check for exact equality to ensure no duplicates or unexpected entries
        assert_eq!(
            translate.get("US"),
            Some(&vec![
                "Cowboy Bebop".to_string(),
                "Cowboy Bebop".to_string()
            ])
        );

        assert_eq!(item.sites.len(), 1);
        assert_eq!(item.sites[0].site, "mal");
        assert_eq!(item.sites[0].id, Some("1".to_string()));
    }

    #[test]
    fn test_title_fallback_with_null_japanese_title() {
        let json = r#"{
            "mal_id": 2,
            "url": "https://example.com",
            "images": {},
            "title": "Main Title",
            "title_english": "English Title",
            "title_japanese": null,
            "type": "TV",
            "aired": {},
            "studios": [],
            "genres": []
        }"#;
        let item = convert_to_item(create_anime(json));
        assert_eq!(item.title, "Main Title");
    }

    #[test]
    fn test_title_fallback_with_empty_japanese_title() {
        let json = r#"{
            "mal_id": 3,
            "url": "https://example.com",
            "images": {},
            "title": "Main Title",
            "title_english": null,
            "title_japanese": "",
            "type": "TV",
            "aired": {},
            "studios": [],
            "genres": []
        }"#;
        let item = convert_to_item(create_anime(json));
        assert_eq!(item.title, "Main Title");
    }

    #[test]
    fn test_convert_to_item_types() {
        let types = vec![
            ("TV", ItemType::Tv),
            ("Movie", ItemType::Movie),
            ("OVA", ItemType::Ova),
            ("ONA", ItemType::Web),
            ("Special", ItemType::Tv), // Default fallback
        ];

        for (jikan_type, item_type) in types {
            let json = format!(
                r#"{{
                "mal_id": 1,
                "url": "url",
                "images": {{}},
                "title": "Title",
                "type": "{}",
                "aired": {{}},
                "studios": [],
                "genres": []
            }}"#,
                jikan_type
            );

            let item = convert_to_item(create_anime(&json));
            assert_eq!(
                item.type_field, item_type,
                "Failed for type: {}",
                jikan_type
            );
        }
    }

    #[test]
    fn test_convert_to_item_minimal() {
        let json = r#"{
            "mal_id": 1,
            "url": "url",
            "images": {},
            "title": "Title",
            "aired": {},
            "studios": [],
            "genres": []
        }"#;
        // Should not panic even with many missing optional fields
        let item = convert_to_item(create_anime(json));
        assert_eq!(item.title, "Title");
        assert_eq!(item.type_field, ItemType::Tv); // Default
    }

    #[test]
    fn test_convert_to_metadata_full() {
        let json = r#"{
            "mal_id": 1,
            "url": "https://myanimelist.net/anime/1/Cowboy_Bebop",
            "images": { "jpg": { "image_url": "jpg_url", "large_image_url": "large_jpg_url" } },
            "title": "Cowboy Bebop",
            "title_english": "Cowboy Bebop",
            "title_japanese": "カウボーイビバップ",
            "type": "TV",
            "episodes": 26,
            "status": "Finished Airing",
            "aired": { "from": "1998-04-03T00:00:00+00:00", "to": "1999-04-24T00:00:00+00:00" },
            "score": 8.75,
            "synopsis": "Space western...",
            "studios": [{ "name": "Sunrise" }],
            "genres": [{ "name": "Action" }, { "name": "Sci-Fi" }]
        }"#;

        let anime = create_anime(json);
        let metadata = convert_to_metadata(anime);

        assert_eq!(metadata.source, MetadataSource::Mal("1".to_string()));
        assert_eq!(metadata.title.romaji, Some("Cowboy Bebop".to_string()));
        assert_eq!(metadata.title.english, Some("Cowboy Bebop".to_string()));
        assert_eq!(
            metadata.title.native,
            Some("カウボーイビバップ".to_string())
        );

        let translate = metadata.title_translate.unwrap();
        assert_eq!(translate.get("US"), Some(&vec!["Cowboy Bebop".to_string()]));
        assert_eq!(
            translate.get("JP"),
            Some(&vec!["カウボーイビバップ".to_string()])
        );

        assert_eq!(
            metadata.cover_image.large,
            Some("large_jpg_url".to_string())
        );
        assert_eq!(
            metadata.cover_image.extra_large,
            Some("jpg_url".to_string())
        );
        assert_eq!(metadata.average_score, Some(87)); // 8.75 * 10 = 87.5, truncated to 87
        assert_eq!(metadata.episodes, Some(26));
        assert_eq!(metadata.genres, vec!["Action", "Sci-Fi"]);
        assert_eq!(metadata.description, Some("Space western...".to_string()));
        assert_eq!(metadata.studios.len(), 1);
        assert_eq!(metadata.studios[0].name, "Sunrise");
        assert!(metadata.is_finished);
    }

    #[test]
    fn test_convert_to_metadata_minimal() {
        let json = r#"{
            "mal_id": 999,
            "url": "url",
            "images": {},
            "title": "Minimal Anime",
            "aired": {},
            "studios": [],
            "genres": []
        }"#;

        let anime = create_anime(json);
        let metadata = convert_to_metadata(anime);

        assert_eq!(metadata.source, MetadataSource::Mal("999".to_string()));
        assert_eq!(metadata.title.romaji, Some("Minimal Anime".to_string()));
        assert_eq!(metadata.title.english, None);
        assert_eq!(metadata.title.native, None);
        assert!(metadata.title_translate.is_none());
        assert_eq!(metadata.cover_image.large, None);
        assert_eq!(metadata.cover_image.extra_large, None);
        assert_eq!(metadata.average_score, None);
        assert_eq!(metadata.episodes, None);
        assert!(metadata.genres.is_empty());
        assert_eq!(metadata.description, None);
        assert!(metadata.studios.is_empty());
        assert!(!metadata.is_finished); // Not "Finished Airing"
    }

    #[test]
    fn test_convert_to_metadata_images() {
        // Case 1: Prefer JPG
        let json_jpg = r#"{
            "mal_id": 1, "url": "u", "title": "t", "aired": {}, "studios": [], "genres": [],
            "images": {
                "jpg": { "image_url": "jpg_s", "large_image_url": "jpg_l" },
                "webp": { "image_url": "webp_s", "large_image_url": "webp_l" }
            }
        }"#;
        let m1 = convert_to_metadata(create_anime(json_jpg));
        assert_eq!(m1.cover_image.large, Some("jpg_l".to_string()));
        assert_eq!(m1.cover_image.extra_large, Some("jpg_s".to_string()));

        // Case 2: Fallback to WebP
        let json_webp = r#"{
            "mal_id": 1, "url": "u", "title": "t", "aired": {}, "studios": [], "genres": [],
            "images": {
                "webp": { "image_url": "webp_s", "large_image_url": "webp_l" }
            }
        }"#;
        let m2 = convert_to_metadata(create_anime(json_webp));
        assert_eq!(m2.cover_image.large, Some("webp_l".to_string()));
        assert_eq!(m2.cover_image.extra_large, Some("webp_s".to_string()));

        // Case 3: No images
        let json_none = r#"{
            "mal_id": 1, "url": "u", "title": "t", "aired": {}, "studios": [], "genres": [],
            "images": {}
        }"#;
        let m3 = convert_to_metadata(create_anime(json_none));
        assert_eq!(m3.cover_image.large, None);
        assert_eq!(m3.cover_image.extra_large, None);
    }
}
