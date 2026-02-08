use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub site_meta: SiteMeta,
    pub items: Vec<Item>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteMeta {
    pub bangumi: Bangumi,
    pub tmdb: Tmdb,
    pub anidb: Anidb,
    pub ani_list: AniList,
    pub mal: Mal,
    pub acfun: Acfun,
    pub bilibili: Bilibili,
    #[serde(rename = "bilibili_hk_mo_tw")]
    pub bilibili_hk_mo_tw: BilibiliHkMoTw,
    #[serde(rename = "bilibili_hk_mo")]
    pub bilibili_hk_mo: BilibiliHkMo,
    #[serde(rename = "bilibili_tw")]
    pub bilibili_tw: BilibiliTw,
    pub youku: Youku,
    pub qq: Qq,
    pub iqiyi: Iqiyi,
    pub letv: Letv,
    pub mgtv: Mgtv,
    pub nicovideo: Nicovideo,
    pub netflix: Netflix,
    pub gamer: Gamer,
    #[serde(rename = "gamer_hk")]
    pub gamer_hk: GamerHk,
    #[serde(rename = "muse_hk")]
    pub muse_hk: MuseHk,
    #[serde(rename = "muse_tw")]
    pub muse_tw: MuseTw,
    #[serde(rename = "ani_one")]
    pub ani_one: AniOne,
    #[serde(rename = "ani_one_asia")]
    pub ani_one_asia: AniOneAsia,
    pub viu: Viu,
    pub mytv: Mytv,
    pub disneyplus: Disneyplus,
    pub abema: Abema,
    pub unext: Unext,
    pub crunchyroll: Crunchyroll,
    pub danime: Danime,
    pub tropics: Tropics,
    pub prime: Prime,
    pub youtube: Youtube,
    pub dmhy: Dmhy,
    pub mikan: Mikan,
    #[serde(rename = "bangumi_moe")]
    pub bangumi_moe: BangumiMoe,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bangumi {
    pub title: String,
    pub url_template: String,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tmdb {
    pub title: String,
    pub url_template: String,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Anidb {
    pub title: String,
    pub url_template: String,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AniList {
    pub title: String,
    pub url_template: String,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mal {
    pub title: String,
    pub url_template: String,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Acfun {
    pub title: String,
    pub url_template: String,
    pub regions: Vec<String>,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bilibili {
    pub title: String,
    pub url_template: String,
    pub regions: Vec<String>,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BilibiliHkMoTw {
    pub title: String,
    pub url_template: String,
    pub regions: Vec<String>,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BilibiliHkMo {
    pub title: String,
    pub url_template: String,
    pub regions: Vec<String>,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BilibiliTw {
    pub title: String,
    pub url_template: String,
    pub regions: Vec<String>,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Youku {
    pub title: String,
    pub url_template: String,
    pub regions: Vec<String>,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Qq {
    pub title: String,
    pub url_template: String,
    pub regions: Vec<String>,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Iqiyi {
    pub title: String,
    pub url_template: String,
    pub regions: Vec<String>,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Letv {
    pub title: String,
    pub url_template: String,
    pub regions: Vec<String>,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mgtv {
    pub title: String,
    pub url_template: String,
    pub regions: Vec<String>,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Nicovideo {
    pub title: String,
    pub url_template: String,
    pub regions: Vec<String>,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Netflix {
    pub title: String,
    pub url_template: String,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Gamer {
    pub title: String,
    pub url_template: String,
    pub regions: Vec<String>,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GamerHk {
    pub title: String,
    pub url_template: String,
    pub regions: Vec<String>,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MuseHk {
    pub title: String,
    pub url_template: String,
    pub regions: Vec<String>,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MuseTw {
    pub title: String,
    pub url_template: String,
    pub regions: Vec<String>,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AniOne {
    pub title: String,
    pub url_template: String,
    pub regions: Vec<String>,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AniOneAsia {
    pub title: String,
    pub url_template: String,
    pub regions: Vec<String>,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Viu {
    pub title: String,
    pub url_template: String,
    pub regions: Vec<String>,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mytv {
    pub title: String,
    pub url_template: String,
    pub regions: Vec<String>,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Disneyplus {
    pub title: String,
    pub url_template: String,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Abema {
    pub title: String,
    pub url_template: String,
    pub regions: Vec<String>,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Unext {
    pub title: String,
    pub url_template: String,
    pub regions: Vec<String>,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Crunchyroll {
    pub title: String,
    pub url_template: String,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Danime {
    pub title: String,
    pub url_template: String,
    pub regions: Vec<String>,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tropics {
    pub title: String,
    pub url_template: String,
    pub regions: Vec<String>,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Prime {
    pub title: String,
    pub url_template: String,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Youtube {
    pub title: String,
    pub url_template: String,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dmhy {
    pub title: String,
    pub url_template: String,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mikan {
    pub title: String,
    pub url_template: String,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BangumiMoe {
    pub title: String,
    pub url_template: String,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub title: String,
    pub title_translate: TitleTranslate,
    #[serde(rename = "type")]
    pub type_field: String,
    pub lang: String,
    pub official_site: String,
    pub begin: String,
    pub broadcast: Option<String>,
    pub end: String,
    pub comment: Option<String>,
    pub sites: Vec<Site>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TitleTranslate {
    #[serde(rename = "zh-Hans")]
    #[serde(default)]
    pub zh_hans: Vec<String>,
    #[serde(default)]
    pub en: Vec<String>,
    #[serde(rename = "zh-Hant")]
    #[serde(default)]
    pub zh_hant: Vec<String>,
    #[serde(default)]
    pub ja: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Site {
    pub site: String,
    pub id: Option<String>,
    pub begin: Option<String>,
    pub broadcast: Option<String>,
    pub comment: Option<String>,
    pub url: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnifiedMetadata {
    pub id: String,
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
