use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum SiteType {
    #[default]
    Info,
    Onair,
    Resource,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum Language {
    #[serde(rename = "ja")]
    #[default]
    Ja,
    #[serde(rename = "en")]
    En,
    #[serde(rename = "zh-Hans")]
    ZhHans,
    #[serde(rename = "zh-Hant")]
    ZhHant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum UserStatus {
    #[default]
    Unregistered = 0,
    Watching = 1,
    Completed = 2,
    OnHold = 3,
    Dropped = 4,
    PlanToWatch = 5,
}

impl serde::Serialize for UserStatus {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i32(*self as i32)
    }
}

impl<'de> serde::Deserialize<'de> for UserStatus {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = i32::deserialize(deserializer)?;
        match v {
            0 => Ok(UserStatus::Unregistered),
            1 => Ok(UserStatus::Watching),
            2 => Ok(UserStatus::Completed),
            3 => Ok(UserStatus::OnHold),
            4 => Ok(UserStatus::Dropped),
            5 => Ok(UserStatus::PlanToWatch),
            _ => Ok(UserStatus::Unregistered),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum ItemType {
    #[default]
    Tv,
    Web,
    Movie,
    Ova,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteMetadata {
    pub title: String,
    pub url_template: String,
    pub regions: Option<Vec<String>>,
    #[serde(rename = "type")]
    pub type_field: Option<SiteType>,
}

pub type SiteMeta = std::collections::HashMap<String, SiteMetadata>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct Root {
    pub site_meta: SiteMeta,
    pub items: Vec<Item>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub title: String,
    pub title_translate: TitleTranslate,
    #[serde(rename = "type")]
    pub type_field: ItemType,
    pub lang: Language,
    pub official_site: String,
    pub begin: Option<String>,
    pub broadcast: Option<String>,
    pub end: Option<String>,
    pub comment: Option<String>,
    pub sites: Vec<Site>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TitleTranslate {
    #[serde(rename = "zh-Hans")]
    #[serde(default)]
    pub zh_hans: Option<Vec<String>>,
    #[serde(default)]
    pub en: Option<Vec<String>>,
    #[serde(rename = "zh-Hant")]
    #[serde(default)]
    pub zh_hant: Option<Vec<String>>,
    #[serde(default)]
    pub ja: Option<Vec<String>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Site {
    pub site: String,
    pub id: Option<String>,
    pub begin: Option<String>,
    pub broadcast: Option<String>,
    pub end: Option<String>,
    pub comment: Option<String>,
    pub url: Option<String>,
    pub regions: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "sourceSite", content = "id", rename_all = "lowercase")]
pub enum MetadataSource {
    Tmdb(String),
    Mal(String),
    #[serde(rename = "aniList")]
    Anilist(String),
    Bangumi(String),
    #[serde(rename = "anidb")]
    AniDB(String),
    #[serde(rename = "bilibili")]
    Bilibili(String),
    #[serde(rename = "acfun")]
    AcFun(String),
}

impl Default for MetadataSource {
    fn default() -> Self {
        Self::Tmdb("".to_string())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnifiedMetadata {
    #[serde(flatten)]
    pub source: MetadataSource,
    pub title: UniversalTitle,
    pub cover_image: UniversalCoverImage,
    pub average_score: Option<i32>,
    pub episodes: Option<i32>,
    pub genres: Vec<String>,
    pub description: Option<String>,
    pub studios: Vec<String>,
    pub characters: Vec<UniversalCharacter>,
    pub staff: Vec<UniversalStaff>,
    pub episodes_list: Vec<UniversalEpisode>,
    pub is_finished: bool,
    pub total_seasons: Option<i32>,
    pub current_season: Option<i32>,
    pub runtime: Option<i32>,
    pub content_rating: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniversalTitle {
    pub romaji: Option<String>,
    pub english: Option<String>,
    pub native: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniversalCoverImage {
    pub large: Option<String>,
    pub extra_large: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniversalCharacter {
    pub name: String,
    pub voice_actor: Option<String>,
    pub role: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniversalEpisode {
    pub number: i32,
    pub title: Option<String>,
    pub air_date: Option<String>,
    pub overview: Option<String>,
    pub runtime: Option<i32>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniversalStaff {
    pub name: String,
    pub role: String,
    pub department: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_site_metadata() {
        let json = r#"{
            "title": "番组计划",
            "urlTemplate": "https://bangumi.tv/subject/{{id}}",
            "type": "info"
        }"#;
        let meta: SiteMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(meta.type_field, Some(SiteType::Info));
    }

    #[test]
    fn test_deserialize_item() {
        let json = r#"{
            "title": "海賊王",
            "titleTranslate": {
                "zh-Hans": ["航海王"],
                "en": ["One Piece"]
            },
            "type": "tv",
            "lang": "ja",
            "officialSite": "http://www.one-piece.com/",
            "begin": "1999-10-20T00:00:00.000Z",
            "end": "",
            "sites": [
                {
                    "site": "bangumi",
                    "id": "975"
                }
            ]
        }"#;
        let item: Item = serde_json::from_str(json).unwrap();
        assert_eq!(item.type_field, ItemType::Tv);
        assert_eq!(item.lang, Language::Ja);
    }
}
