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
    }
}
