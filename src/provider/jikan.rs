use crate::model::{
    Item, ItemType, Language, Site, TitleTranslate, UnifiedMetadata, UniversalCoverImage,
    UniversalTitle,
};
use crate::provider::MetadataProvider;
use crate::utils;
use regex::Regex;
use serde_derive::Deserialize;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;
use worker::*;

static HTML_TAG_REGEX: OnceLock<Regex> = OnceLock::new();

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
    status: String,
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
    async fn fetch(
        &self,
        id: Option<&str>,
        _title: Option<&str>,
        _year: Option<i32>,
    ) -> Result<UnifiedMetadata> {
        let mal_id = id.ok_or_else(|| Error::RustError("MAL ID required".into()))?;
        let url = format!("https://api.jikan.moe/v4/anime/{}/full", mal_id);

        let response: JikanResponse<JikanAnime> = utils::fetch_json(&url)
            .await?
            .ok_or_else(|| Error::RustError("Jikan data not found".into()))?;

        let anime = response.data;
        Ok(convert_to_metadata(anime))
    }
}

pub async fn fetch_season(year: i32, season: &str) -> Result<Vec<Item>> {
    let url = format!("https://api.jikan.moe/v4/seasons/{}/{}", year, season);
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
            let mut en = anime.title_english.map(|t| vec![t]).unwrap_or_default();
            // If the default 'title' is different from Japanese title, add it to English/Romaji fallback
            // Jikan's 'title' is usually Romaji.
            if &anime.title != ja_title {
                en.push(anime.title);
            }
            (
                ja_title.clone(),
                TitleTranslate {
                    en: if en.is_empty() { None } else { Some(en) },
                    ja: Some(vec![ja_title.clone()]),
                    ..Default::default()
                },
            )
        } else {
            // Fallback to default title
            (
                anime.title.clone(),
                TitleTranslate {
                    en: anime.title_english.map(|t| vec![t]),
                    ..Default::default()
                },
            )
        }
    } else {
        (
            anime.title.clone(),
            TitleTranslate {
                en: anime.title_english.map(|t| vec![t]),
                ..Default::default()
            },
        )
    };

    // Strip HTML tags from synopsis
    let comment = anime.synopsis.map(|s| {
        let regex = HTML_TAG_REGEX.get_or_init(|| Regex::new(r"<[^>]*>").unwrap());
        regex.replace_all(&s, "").to_string()
    });

    Item {
        title,
        title_translate,
        type_field,
        lang: Language::Ja,
        official_site: anime.url,
        begin: anime.aired.from,
        end: anime.aired.to,
        comment,
        sites,
        broadcast: anime.broadcast.and_then(|b| b.string),
    }
}

fn convert_to_metadata(anime: JikanAnime) -> UnifiedMetadata {
    let image = anime.images.get("jpg").or_else(|| anime.images.get("webp"));

    // Strip HTML from description
    let description = anime.synopsis.map(|s| {
        let regex = HTML_TAG_REGEX.get_or_init(|| Regex::new(r"<[^>]*>").unwrap());
        regex.replace_all(&s, "").to_string()
    });

    UnifiedMetadata {
        id: anime.mal_id.to_string(),
        title: UniversalTitle {
            romaji: Some(anime.title),
            english: anime.title_english,
            native: anime.title_japanese,
        },
        cover_image: UniversalCoverImage {
            large: image.and_then(|i| i.large_image_url.clone()),
            extra_large: image.and_then(|i| i.image_url.clone()), // Fallback
        },
        average_score: anime.score.map(|s| (s * 10.0) as i32),
        episodes: anime.episodes,
        genres: anime.genres.into_iter().map(|g| g.name).collect(),
        description,
        studios: anime.studios.into_iter().map(|s| s.name).collect(),
        characters: vec![],
        staff: vec![],
        episodes_list: vec![],
        is_finished: anime.status == "Finished Airing",
        total_seasons: None,
        current_season: None,
        runtime: None,
        content_rating: None,
    }
}
