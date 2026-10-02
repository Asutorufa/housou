use crate::model::{MetadataSource, UnifiedMetadata};

use super::{cover_url, escape_html, japanese_synopsis, plain_text};

/// Visible HTML for readers without JavaScript and Telegram's Instant View bot.
/// Only non-personal metadata is rendered; React replaces it on app startup.
pub(super) fn render(title: &str, url: &url::Url, metadata: Option<&UnifiedMetadata>) -> String {
    let title = escape_html(title);
    let eligible = if metadata.is_some() { "true" } else { "false" };
    let mut html = format!(
        "<article id=\"anime-article\" lang=\"ja\" data-instant-view=\"{eligible}\">\
         <h1>{title}</h1>"
    );
    if let Some(image) = metadata.and_then(cover_url) {
        html.push_str(&format!(
            "<figure id=\"anime-cover\"><img src=\"{}\" alt=\"{title}\"></figure>",
            escape_html(image)
        ));
    }
    html.push_str("<div id=\"anime-body\">");
    if let Some(metadata) = metadata {
        html.push_str("<h2>あらすじ</h2>");
        let synopsis = japanese_synopsis(metadata)
            .map(plain_text)
            .filter(|text| !text.is_empty())
            .unwrap_or_else(|| "日本語のあらすじはまだ登録されていません。".into());
        html.push_str(&format!("<p>{}</p>", escape_html(&synopsis)));
        html.push_str(&basic_info(metadata));
        html.push_str(&list_section(
            "スタジオ",
            metadata.studios.iter().map(|studio| studio.name.clone()),
        ));
        html.push_str(&list_section(
            "キャスト",
            metadata.characters.iter().map(|character| {
                if let Some(actor) = &character.voice_actor {
                    format!("{}（声：{actor}）", character.name)
                } else {
                    character.name.clone()
                }
            }),
        ));
        html.push_str(&list_section(
            "スタッフ",
            metadata.staff.iter().map(|staff| {
                if staff.role.trim().is_empty() {
                    staff.name.clone()
                } else {
                    format!("{}：{}", staff.role, staff.name)
                }
            }),
        ));
        if !metadata.episodes_list.is_empty() {
            html.push_str("<h2>エピソード</h2>");
            for episode in &metadata.episodes_list {
                let heading = if let Some(title) = &episode.title {
                    format!("第{}話：{title}", episode.number)
                } else {
                    format!("第{}話", episode.number)
                };
                html.push_str(&format!("<h3>{}</h3>", escape_html(&heading)));
                if let Some(date) = &episode.air_date {
                    html.push_str(&format!("<p>放送日：{}</p>", escape_html(date)));
                }
                if let Some(runtime) = episode.runtime.filter(|runtime| *runtime > 0) {
                    html.push_str(&format!("<p>再生時間：{runtime}分</p>"));
                }
                if matches!(metadata.source, MetadataSource::Tmdb(_))
                    && let Some(overview) = &episode.overview
                {
                    let overview = plain_text(overview);
                    if !overview.is_empty() {
                        html.push_str(&format!("<p>{}</p>", escape_html(&overview)));
                    }
                }
            }
        }
        if let Some((label, link)) = source_link(&metadata.source) {
            html.push_str(&format!(
                "<h2>作品情報</h2><p><a href=\"{}\">{label}で見る</a></p>",
                escape_html(link.as_str())
            ));
        }
    } else {
        html.push_str(
            "<p>作品情報を取得できませんでした。ウェブサイトから再度お試しください。</p>",
        );
    }
    html.push_str(&format!(
        "<h2>放送</h2><p><a href=\"{}\">配信情報・コメントをウェブサイトで見る</a></p>\
         </div></article>",
        escape_html(url.as_str())
    ));
    html
}

fn basic_info(metadata: &UnifiedMetadata) -> String {
    let status = if metadata.is_finished {
        "放送終了"
    } else {
        "放送中・予定"
    };
    let mut html = format!("<h2>基本情報</h2><ul><li>放送状況：{status}</li>");
    if let Some(episodes) = metadata.episodes.filter(|episodes| *episodes > 0) {
        html.push_str(&format!("<li>話数：{episodes}話</li>"));
    }
    if let Some(runtime) = metadata.runtime.filter(|runtime| *runtime > 0) {
        html.push_str(&format!("<li>再生時間：{runtime}分</li>"));
    }
    if let Some(score) = metadata.average_score.filter(|score| *score > 0) {
        html.push_str(&format!("<li>評価：{score}%</li>"));
    }
    if let Some(seasons) = metadata.total_seasons.filter(|seasons| *seasons > 0) {
        let current = metadata.current_season.unwrap_or(1);
        html.push_str(&format!(
            "<li>シーズン：{current} / 全{seasons}シーズン</li>"
        ));
    }
    if let Some(rating) = &metadata.content_rating {
        html.push_str(&format!("<li>年齢区分：{}</li>", escape_html(rating)));
    }
    if matches!(metadata.source, MetadataSource::Tmdb(_)) && !metadata.genres.is_empty() {
        html.push_str(&format!(
            "<li>ジャンル：{}</li>",
            escape_html(&metadata.genres.join("、"))
        ));
    }
    html.push_str("</ul>");
    html
}

fn list_section(heading: &str, values: impl Iterator<Item = String>) -> String {
    let entries: String = values
        .filter(|value| !value.trim().is_empty())
        .map(|value| format!("<li>{}</li>", escape_html(&value)))
        .collect();
    if entries.is_empty() {
        String::new()
    } else {
        format!("<h2>{heading}</h2><ul>{entries}</ul>")
    }
}

fn source_link(source: &MetadataSource) -> Option<(&'static str, url::Url)> {
    let (label, base, id) = match source {
        MetadataSource::Tmdb(id) => ("TMDb", "https://www.themoviedb.org/", id),
        MetadataSource::Mal(id) => ("MyAnimeList", "https://myanimelist.net/anime/", id),
        MetadataSource::Anilist(id) => ("AniList", "https://anilist.co/anime/", id),
        _ => return None,
    };
    if id.trim().is_empty() {
        return None;
    }
    let mut url = url::Url::parse(base).ok()?;
    url.path_segments_mut()
        .ok()?
        .pop_if_empty()
        .extend(id.split('/'));
    Some((label, url))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        Studio, UniversalCharacter, UniversalCoverImage, UniversalEpisode, UniversalStaff,
    };

    #[test]
    fn article_keeps_full_japanese_synopsis_and_metadata() {
        let synopsis = format!("<p>{}最後まで読める。</p>", "物語。".repeat(100));
        let metadata = UnifiedMetadata {
            source: MetadataSource::Tmdb("tv/123/season/2".into()),
            description: Some(synopsis),
            cover_image: UniversalCoverImage {
                large: Some("https://example.com/cover.jpg".into()),
                ..Default::default()
            },
            episodes: Some(12),
            runtime: Some(24),
            average_score: Some(88),
            total_seasons: Some(2),
            current_season: Some(2),
            genres: vec!["アニメーション".into()],
            studios: vec![Studio {
                name: "制作スタジオ".into(),
                ..Default::default()
            }],
            characters: vec![UniversalCharacter {
                name: "主人公".into(),
                voice_actor: Some("声優".into()),
                ..Default::default()
            }],
            staff: vec![UniversalStaff {
                name: "監督名".into(),
                role: "監督".into(),
                ..Default::default()
            }],
            episodes_list: vec![UniversalEpisode {
                number: 1,
                title: Some("旅の始まり".into()),
                overview: Some("<b>最初の物語。</b>".into()),
                ..Default::default()
            }],
            ..Default::default()
        };
        let url = url::Url::parse("https://example.com/?day=1&anime=Test").unwrap();
        let html = render("作品名", &url, Some(&metadata));
        assert!(html.contains("<h1>作品名</h1>"));
        assert!(html.contains("data-instant-view=\"true\""));
        assert!(html.contains("<figure id=\"anime-cover\">"));
        assert!(html.contains("最後まで読める。</p>"));
        assert!(!html.contains('…'));
        for value in [
            "12話",
            "24分",
            "88%",
            "全2シーズン",
            "制作スタジオ",
            "主人公（声：声優）",
            "監督：監督名",
            "第1話：旅の始まり",
            "最初の物語。",
        ] {
            assert!(html.contains(value), "Missing {value}");
        }
        assert!(html.contains("href=\"https://www.themoviedb.org/tv/123/season/2\""));
        assert!(html.contains("href=\"https://example.com/?day=1&amp;anime=Test\""));
    }

    #[test]
    fn missing_metadata_remains_readable_without_enabling_instant_view() {
        let url = url::Url::parse("https://example.com/?anime=Test").unwrap();
        let html = render("作品名", &url, None);
        assert!(html.contains("<h1>作品名</h1>"));
        assert!(html.contains("data-instant-view=\"false\""));
        assert!(html.contains("作品情報を取得できませんでした。"));
        assert!(!html.contains("基本情報"));
        assert!(!html.contains("<figure"));
    }

    #[test]
    fn all_metadata_fields_and_links_are_escaped() {
        let attack = "\"><script>alert(1)</script>";
        let metadata = UnifiedMetadata {
            source: MetadataSource::Anilist(attack.into()),
            studios: vec![Studio {
                name: attack.into(),
                ..Default::default()
            }],
            description: Some("English synopsis".into()),
            ..Default::default()
        };
        let url = url::Url::parse("https://example.com/").unwrap();
        let html = render(attack, &url, Some(&metadata));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("English synopsis"));
        assert!(html.contains("日本語のあらすじはまだ登録されていません。"));
        assert!(html.contains("https://anilist.co/anime/%22%3E%3Cscript%3E"));
    }
}
