use super::match_score::{self, MediaKind};
use super::{LookupQuery, MetadataProvider};
use crate::model::{
    MetadataSource, Studio, TitleTranslate, UnifiedMetadata, UniversalCoverImage, UniversalTitle,
};
use serde::Deserialize;
use worker::wasm_bindgen::JsValue;
use worker::*;

const BANGUMI_API: &str = "https://api.bgm.tv";
const USER_AGENT: &str = "housou/0.1.0 (https://github.com/Asutorufa/housou)";
const SEARCH_LIMIT: usize = 10;
const SEARCH_TITLE_LIMIT: usize = 4;

pub struct BangumiProvider;

#[derive(Debug, Clone, Deserialize)]
struct BangumiSubject {
    id: i64,
    #[serde(default)]
    name: String,
    #[serde(default)]
    name_cn: String,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    date: Option<String>,
    #[serde(default)]
    platform: String,
    #[serde(default)]
    images: Option<BangumiImages>,
    #[serde(default)]
    infobox: Vec<WikiItem>,
    #[serde(default)]
    eps: i32,
    #[serde(default)]
    total_episodes: i32,
    #[serde(default)]
    rating: Option<BangumiRating>,
    #[serde(default)]
    meta_tags: Vec<String>,
    #[serde(default)]
    tags: Vec<BangumiTag>,
    #[serde(default)]
    nsfw: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct BangumiImages {
    #[serde(default)]
    large: String,
    #[serde(default)]
    common: String,
}

#[derive(Debug, Clone, Deserialize)]
struct BangumiRating {
    #[serde(default)]
    score: f64,
}

#[derive(Debug, Clone, Deserialize)]
struct BangumiTag {
    name: String,
}

#[derive(Debug, Clone, Deserialize)]
struct WikiItem {
    key: String,
    value: WikiValue,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum WikiValue {
    Text(String),
    Items(Vec<WikiValueItem>),
    Other(serde_json::Value),
}

#[derive(Debug, Clone, Deserialize)]
struct WikiValueItem {
    #[serde(default)]
    k: Option<String>,
    v: String,
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    #[serde(default)]
    data: Vec<BangumiSubject>,
}

impl MetadataProvider for BangumiProvider {
    async fn fetch(&self, query: LookupQuery<'_>) -> Result<UnifiedMetadata> {
        let subject = match query {
            LookupQuery::ById(id) => fetch_subject(id).await?,
            LookupQuery::ByTitle {
                title,
                aliases,
                year,
                media_type,
            } => search_subject(title, aliases, year, media_type).await?,
        };

        Ok(subject_to_unified(subject))
    }
}

async fn fetch_subject(id: &str) -> Result<BangumiSubject> {
    let id = id
        .trim()
        .trim_start_matches("subject/")
        .trim_start_matches('/');
    if id.is_empty() || !id.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(Error::RustError("Invalid Bangumi subject ID".into()));
    }

    let url = format!("{BANGUMI_API}/v0/subjects/{id}");
    let mut response = send(Method::Get, &url, None).await?;
    match response.status_code() {
        200 => response.json().await,
        404 => Err(Error::RustError("Bangumi: Not Found".into())),
        status => Err(Error::RustError(format!(
            "Bangumi subject request failed with HTTP {status}"
        ))),
    }
}

async fn search_subject(
    title: &str,
    aliases: &[String],
    year: Option<i32>,
    media_type: Option<&str>,
) -> Result<BangumiSubject> {
    let expected_titles = match_score::title_candidates(Some(title), aliases, SEARCH_TITLE_LIMIT);
    if expected_titles.is_empty() {
        return Err(Error::RustError("Bangumi: empty search title".into()));
    }

    let expected_kind = MediaKind::from_request(media_type);
    let min_score = if year.is_none() && expected_kind.is_none() {
        90
    } else {
        105
    };
    let mut best: Option<(i32, BangumiSubject)> = None;

    for query in &expected_titles {
        let body = serde_json::json!({
            "keyword": query,
            "sort": "match",
            "filter": {
                "type": [2]
            }
        });
        let url = format!("{BANGUMI_API}/v0/search/subjects?limit={SEARCH_LIMIT}&offset=0");
        let mut response = send(Method::Post, &url, Some(&body)).await?;
        if response.status_code() != 200 {
            return Err(Error::RustError(format!(
                "Bangumi search failed with HTTP {}",
                response.status_code()
            )));
        }

        let results: SearchResponse = response.json().await?;
        for subject in results.data {
            let score = score_subject(&expected_titles, year, expected_kind, &subject);
            if best
                .as_ref()
                .is_none_or(|(best_score, _)| score > *best_score)
            {
                best = Some((score, subject));
            }
        }

        if best.as_ref().is_some_and(|(score, _)| *score >= 150) {
            break;
        }
    }

    match best {
        Some((score, subject)) if score >= min_score => Ok(subject),
        _ => Err(Error::RustError("Bangumi: Not Found".into())),
    }
}

async fn send(method: Method, url: &str, body: Option<&serde_json::Value>) -> Result<Response> {
    let headers = Headers::new();
    headers.set("Accept", "application/json")?;
    headers.set("User-Agent", USER_AGENT)?;
    if body.is_some() {
        headers.set("Content-Type", "application/json")?;
    }

    let mut init = RequestInit::new();
    init.with_method(method);
    init.with_headers(headers);

    if let Some(body) = body {
        init.with_body(Some(JsValue::from_str(&body.to_string())));
    }

    let request = Request::new_with_init(url, &init)?;
    Fetch::Request(request).send().await
}

fn score_subject(
    expected_titles: &[String],
    year: Option<i32>,
    expected_kind: Option<MediaKind>,
    subject: &BangumiSubject,
) -> i32 {
    let mut titles = vec![subject.name.as_str(), subject.name_cn.as_str()];
    let infobox_aliases = subject_aliases(subject);
    titles.extend(infobox_aliases.iter().map(String::as_str));

    match_score::title_score(expected_titles, titles)
        + match_score::year_score(year, subject_year(subject))
        + match_score::media_kind_score(expected_kind, subject_media_kind(subject))
}

fn subject_year(subject: &BangumiSubject) -> Option<i32> {
    subject
        .date
        .as_deref()
        .and_then(|date| date.get(..4))
        .and_then(|year| year.parse().ok())
}

fn subject_media_kind(subject: &BangumiSubject) -> MediaKind {
    match subject.platform.trim().to_ascii_lowercase().as_str() {
        "movie" | "剧场版" | "劇場版" => MediaKind::Movie,
        "ova" => MediaKind::Ova,
        "web" | "ona" => MediaKind::Ona,
        "special" => MediaKind::Special,
        "tv" => MediaKind::Tv,
        _ => MediaKind::Other,
    }
}

fn wiki_values(item: &WikiItem) -> Vec<(Option<&str>, &str)> {
    match &item.value {
        WikiValue::Text(value) => vec![(None, value.as_str())],
        WikiValue::Items(values) => values
            .iter()
            .map(|value| (value.k.as_deref(), value.v.as_str()))
            .collect(),
        WikiValue::Other(value) => {
            let _ = value;
            Vec::new()
        }
    }
}

fn subject_aliases(subject: &BangumiSubject) -> Vec<String> {
    subject
        .infobox
        .iter()
        .filter(|item| {
            let key = item.key.to_ascii_lowercase();
            key.contains("别名") || key.contains("別名") || key.contains("alias")
        })
        .flat_map(wiki_values)
        .map(|(_, value)| value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

fn subject_to_unified(subject: BangumiSubject) -> UnifiedMetadata {
    let mut title_translate = TitleTranslate::new();
    if !subject.name.is_empty() {
        title_translate.insert("JP".into(), vec![subject.name.clone()]);
    }
    if !subject.name_cn.is_empty() {
        title_translate.insert("CN".into(), vec![subject.name_cn.clone()]);
    }

    let mut english = None;
    let mut romaji = None;
    let mut studios = Vec::new();

    for item in &subject.infobox {
        let key = item.key.to_ascii_lowercase();
        let values = wiki_values(item);

        if key.contains("别名") || key.contains("別名") || key.contains("alias") {
            for &(label, value) in &values {
                let label = label.unwrap_or("").to_ascii_lowercase();
                if label.contains("英文") || label.contains("english") {
                    english.get_or_insert_with(|| value.to_string());
                    title_translate
                        .entry("US".into())
                        .or_default()
                        .push(value.to_string());
                } else if label.contains("罗马")
                    || label.contains("羅馬")
                    || label.contains("romaji")
                {
                    romaji.get_or_insert_with(|| value.to_string());
                }
            }
        }

        if matches!(
            item.key.as_str(),
            "动画制作" | "動畫製作" | "アニメーション制作" | "制作"
        ) {
            studios.extend(values.into_iter().filter_map(|(_, value)| {
                let name = value.trim();
                (!name.is_empty()).then(|| Studio {
                    name: name.to_string(),
                    logo_url: None,
                })
            }));
        }
    }

    let mut genres = subject.meta_tags.clone();
    if genres.is_empty() {
        genres = subject
            .tags
            .iter()
            .take(10)
            .map(|tag| tag.name.clone())
            .collect();
    }

    let episodes = if subject.eps > 0 {
        Some(subject.eps)
    } else if subject.total_episodes > 0 {
        Some(subject.total_episodes)
    } else {
        None
    };

    let cover_image = UniversalCoverImage {
        large: subject
            .images
            .as_ref()
            .map(|images| images.common.clone())
            .filter(|url| !url.is_empty()),
        extra_large: subject
            .images
            .as_ref()
            .map(|images| images.large.clone())
            .filter(|url| !url.is_empty()),
    };

    UnifiedMetadata {
        source: MetadataSource::Bangumi(subject.id.to_string()),
        title: UniversalTitle {
            romaji,
            english,
            native: (!subject.name.is_empty()).then_some(subject.name),
        },
        title_translate: (!title_translate.is_empty()).then_some(title_translate),
        cover_image,
        average_score: subject
            .rating
            .and_then(|rating| (rating.score > 0.0).then(|| (rating.score * 10.0).round() as i32)),
        episodes,
        genres,
        description: (!subject.summary.is_empty()).then_some(subject.summary),
        studios,
        characters: vec![],
        staff: vec![],
        episodes_list: vec![],
        is_finished: false,
        total_seasons: None,
        current_season: None,
        runtime: None,
        content_rating: subject.nsfw.then_some("R18".to_string()),
        videos: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn subject(name: &str, name_cn: &str, date: &str, platform: &str) -> BangumiSubject {
        BangumiSubject {
            id: 616808,
            name: name.into(),
            name_cn: name_cn.into(),
            summary: "summary".into(),
            date: Some(date.into()),
            platform: platform.into(),
            images: None,
            infobox: vec![],
            eps: 12,
            total_episodes: 12,
            rating: Some(BangumiRating { score: 8.2 }),
            meta_tags: vec!["动画".into()],
            tags: vec![],
            nsfw: false,
        }
    }

    #[test]
    fn matching_uses_chinese_alias_and_year() {
        let subject = subject("テストアニメ", "测试动画", "2026-04-01", "TV");
        let expected = vec!["测试动画".to_string()];
        assert!(score_subject(&expected, Some(2026), Some(MediaKind::Tv), &subject) >= 150);
    }

    #[test]
    fn conversion_keeps_bangumi_source_and_urls() {
        let mut subject = subject("テストアニメ", "测试动画", "2026-04-01", "TV");
        subject.images = Some(BangumiImages {
            large: "https://lain.bgm.tv/large.jpg".into(),
            common: "https://lain.bgm.tv/common.jpg".into(),
        });
        let metadata = subject_to_unified(subject);
        assert_eq!(metadata.source, MetadataSource::Bangumi("616808".into()));
        assert_eq!(metadata.average_score, Some(82));
        assert_eq!(metadata.episodes, Some(12));
        assert_eq!(
            metadata.cover_image.large.as_deref(),
            Some("https://lain.bgm.tv/common.jpg")
        );
        assert_eq!(
            metadata.cover_image.extra_large.as_deref(),
            Some("https://lain.bgm.tv/large.jpg")
        );
    }
}
