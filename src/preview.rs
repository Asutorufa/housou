use std::sync::LazyLock;
use std::time::Duration;

use futures::future::{Either, select};
use regex::Regex;
use worker::*;

use crate::model::{MetadataSource, UnifiedMetadata};
use crate::provider::{self, MetadataRequest};

const SITE_NAME: &str = "放送";
const SITE_DESCRIPTION: &str =
    "毎週のアニメ放送スケジュール、作品情報、配信サービスを確認できます。";
static HTML_TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>]*>").unwrap());

pub async fn handle_page(req: Request, ctx: RouteContext<Context>) -> Result<Response> {
    let mut url = req.url()?;
    url.set_fragment(None);

    // Fetch the built Vite shell through the binding, preserving its JS/CSS URLs.
    // Do not forward conditional or range headers: metadata changes the document.
    let assets = ctx.env.get_binding::<Fetcher>("ASSETS")?;
    let mut asset = assets.fetch(url.as_str(), None).await?;
    if asset.status_code() != 200 {
        return Ok(asset);
    }

    let anime = anime_title(&url);
    let metadata = if let Some(title) = &anime {
        let request = MetadataRequest {
            title: Some(title.clone()),
            ..Default::default()
        };
        let origin = url.origin().ascii_serialization();
        let lookup = provider::fetch_metadata(&request, &ctx, &origin);
        let timeout = Delay::from(Duration::from_secs(5));
        futures::pin_mut!(lookup, timeout);
        match select(lookup, timeout).await {
            Either::Left((Ok(metadata), _)) => Some(metadata),
            Either::Left((Err(err), _)) => {
                console_warn!("Preview metadata unavailable: {}", err);
                None
            }
            Either::Right(_) => {
                console_warn!("Preview metadata lookup timed out");
                None
            }
        }
    } else {
        None
    };

    // This is the small, trusted app shell, rather than a remote document.
    let html = inject_preview(
        &asset.text().await?,
        &url,
        anime.as_deref(),
        metadata.as_ref(),
    );
    let headers = asset.headers().clone();
    // Asset validators and lengths describe the unmodified shell. Reusing them
    // could serve one anime's preview for a different query string.
    for name in [
        "ETag",
        "Last-Modified",
        "Content-Length",
        "Content-Encoding",
    ] {
        headers.delete(name)?;
    }
    headers.set("Cache-Control", "no-store")?;
    headers.set("X-Content-Type-Options", "nosniff")?;
    headers.set("X-Frame-Options", "DENY")?;

    let body = if req.method() == Method::Head {
        ""
    } else {
        &html
    };
    Ok(Response::from_html(body)?.with_headers(headers))
}

fn anime_title(url: &url::Url) -> Option<String> {
    url.query_pairs()
        .find(|(name, _)| name == "anime")
        .map(|(_, title)| title.into_owned())
        .filter(|title| !title.trim().is_empty())
}

fn plain_description(description: &str) -> String {
    let without_tags = HTML_TAG.replace_all(description, " ");
    let decoded = html_escape::decode_html_entities(&without_tags);
    let text = decoded.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut shortened: String = text.chars().take(240).collect();
    if text.chars().count() > 240 {
        shortened.push('…');
    }
    shortened
}

fn cover_url(metadata: &UnifiedMetadata) -> Option<&str> {
    [
        metadata.cover_image.extra_large.as_deref(),
        metadata.cover_image.large.as_deref(),
    ]
    .into_iter()
    .flatten()
    .find(|image| {
        url::Url::parse(image)
            .is_ok_and(|url| matches!(url.scheme(), "http" | "https") && url.host_str().is_some())
    })
}

fn inject_preview(
    html: &str,
    url: &url::Url,
    anime: Option<&str>,
    metadata: Option<&UnifiedMetadata>,
) -> String {
    let title = anime.map_or_else(
        || SITE_NAME.to_string(),
        |title| format!("{title} - {SITE_NAME}"),
    );
    let description = metadata
        // TMDb is queried in ja-JP. AniList and Jikan synopses are in English,
        // so use Japanese page copy while still showing their covers.
        .filter(|metadata| matches!(metadata.source, MetadataSource::Tmdb(_)))
        .and_then(|metadata| metadata.description.as_deref())
        .map(plain_description)
        .filter(|description| !description.is_empty())
        .unwrap_or_else(|| {
            anime.map_or_else(
                || SITE_DESCRIPTION.to_string(),
                |title| {
                    format!("「{title}」の作品情報、放送スケジュール、配信サービスを確認できます。")
                },
            )
        });
    let image = metadata.and_then(cover_url);

    // Escape both text and attribute values, including user-provided anime names.
    let escape = |value: &str| html_escape::encode_double_quoted_attribute(value).into_owned();
    let title = escape(&title);
    let description = escape(&description);
    let page_url = escape(url.as_str());
    let mut tags = format!(
        "<meta name=\"description\" content=\"{description}\">\n\
         <meta property=\"og:type\" content=\"website\">\n\
         <meta property=\"og:site_name\" content=\"{SITE_NAME}\">\n\
         <meta property=\"og:locale\" content=\"ja_JP\">\n\
         <meta property=\"og:title\" content=\"{title}\">\n\
         <meta property=\"og:description\" content=\"{description}\">\n\
         <meta property=\"og:url\" content=\"{page_url}\">\n\
         <meta name=\"twitter:title\" content=\"{title}\">\n\
         <meta name=\"twitter:description\" content=\"{description}\">\n"
    );
    if let Some(image) = image {
        let image = escape(image);
        tags.push_str(&format!(
            "<meta property=\"og:image\" content=\"{image}\">\n\
             <meta property=\"og:image:alt\" content=\"{title}\">\n\
             <meta name=\"twitter:card\" content=\"summary_large_image\">\n\
             <meta name=\"twitter:image\" content=\"{image}\">\n"
        ));
    } else {
        tags.push_str("<meta name=\"twitter:card\" content=\"summary\">\n");
    }

    html.replacen("<title>放送</title>", &format!("<title>{title}</title>"), 1)
        .replacen("</head>", &format!("{tags}</head>"), 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::UniversalCoverImage;

    const SHELL: &str = include_str!("../web/index.html");

    #[test]
    fn anime_link_includes_metadata_and_preserves_the_app() {
        let url = url::Url::parse(
            "https://housou.asutorufa.com/?day=1&anime=%E5%BD%BC%E5%A5%B3%E3%81%AE%E5%8F%8B%E9%81%94",
        ).unwrap();
        let anime = anime_title(&url).unwrap();
        assert_eq!(anime, "彼女の友達");
        let metadata = UnifiedMetadata {
            description: Some("<p>作品紹介<br>Tom &amp; Jerry &#39;test&#39;</p>".into()),
            cover_image: UniversalCoverImage {
                large: Some("https://example.com/cover.jpg".into()),
                extra_large: Some("https://example.com/large.jpg".into()),
            },
            ..Default::default()
        };
        let html = inject_preview(SHELL, &url, Some(&anime), Some(&metadata));
        assert!(html.contains("<title>彼女の友達 - 放送</title>"));
        assert!(html.contains("property=\"og:title\" content=\"彼女の友達 - 放送\""));
        assert!(html.contains("content=\"作品紹介 Tom &amp; Jerry 'test'\""));
        assert!(html.contains("<html lang=\"ja\">"));
        assert!(html.contains("property=\"og:locale\" content=\"ja_JP\""));
        assert!(html.contains("property=\"og:image\" content=\"https://example.com/large.jpg\""));
        assert!(html.contains("summary_large_image"));
        assert!(html.contains("?day=1&amp;anime="));
        assert!(html.contains("<div id=\"root\"></div>"));
        assert!(html.contains("src=\"/src/main.tsx\""));
        assert_eq!(html.matches("<title>").count(), 1);
    }

    #[test]
    fn unavailable_metadata_still_previews_the_anime() {
        let url = url::Url::parse("https://example.com/?anime=Test").unwrap();
        let html = inject_preview(SHELL, &url, Some("Test"), None);
        assert!(html.contains("content=\"Test - 放送\""));
        assert!(html.contains("「Test」の作品情報、放送スケジュール、配信サービス"));
        assert!(!html.contains("og:image\""));
        assert!(html.contains("content=\"summary\""));
    }

    #[test]
    fn non_japanese_provider_synopsis_uses_japanese_copy_with_cover() {
        let url = url::Url::parse("https://example.com/?anime=Test").unwrap();
        for source in [
            MetadataSource::Anilist("1".into()),
            MetadataSource::Mal("2".into()),
        ] {
            let metadata = UnifiedMetadata {
                source,
                description: Some("An English synopsis".into()),
                cover_image: UniversalCoverImage {
                    large: Some("https://example.com/cover.jpg".into()),
                    ..Default::default()
                },
                ..Default::default()
            };
            let html = inject_preview(SHELL, &url, Some("Test"), Some(&metadata));
            assert!(!html.contains("An English synopsis"));
            assert!(html.contains("「Test」の作品情報、放送スケジュール、配信サービス"));
            assert!(
                html.contains("property=\"og:image\" content=\"https://example.com/cover.jpg\"")
            );
        }
    }

    #[test]
    fn homepage_and_empty_anime_have_site_metadata() {
        for query in ["", "?anime=", "?anime=+++"] {
            let url = url::Url::parse(&format!("https://example.com/{query}")).unwrap();
            assert_eq!(anime_title(&url), None);
            let html = inject_preview(SHELL, &url, None, None);
            assert!(html.contains("<title>放送</title>"));
            assert!(html.contains(SITE_DESCRIPTION));
        }
    }

    #[test]
    fn preview_values_cannot_inject_html() {
        let title = "\"></title><script>alert('x')</script><meta content=\"";
        let mut url = url::Url::parse("https://example.com/").unwrap();
        url.query_pairs_mut().append_pair("anime", title);
        let metadata = UnifiedMetadata {
            description: Some("&lt;script&gt;alert(1)&lt;/script&gt; &quot;quote&quot;".into()),
            ..Default::default()
        };
        let html = inject_preview(SHELL, &url, Some(title), Some(&metadata));
        assert!(!html.contains("<script>alert"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("&quot;quote&quot;"));
        assert_eq!(html.matches("<title>").count(), 1);
    }

    #[test]
    fn invalid_covers_fall_back_to_a_valid_image_or_text() {
        let mut metadata = UnifiedMetadata {
            cover_image: UniversalCoverImage {
                extra_large: Some("javascript:alert(1)".into()),
                large: Some("https://example.com/cover.jpg".into()),
            },
            ..Default::default()
        };
        assert_eq!(cover_url(&metadata), Some("https://example.com/cover.jpg"));
        metadata.cover_image.large = Some("/cover.jpg".into());
        assert_eq!(cover_url(&metadata), None);
    }

    #[test]
    fn long_descriptions_are_shortened_without_breaking_unicode() {
        let description = "番".repeat(241);
        assert_eq!(
            plain_description(&description),
            format!("{}…", "番".repeat(240))
        );
        assert_eq!(plain_description("<br> &nbsp; \n"), "");
    }
}
