use worker::*;

pub async fn fetch_json<T: for<'de> serde::Deserialize<'de>>(url: &str) -> Result<Option<T>> {
    let mut init = RequestInit::new();
    init.with_method(Method::Get);

    let mut cf = CfProperties::new();
    let ttl_by_status = std::collections::HashMap::from([
        ("200".to_string(), crate::config::CACHE_TTL_SECONDS),
        ("404".to_string(), crate::config::CACHE_TTL_404),
    ]);
    cf.cache_ttl_by_status = Some(ttl_by_status);
    init.with_cf_properties(cf);

    let request = Request::new_with_init(url, &init)?;
    let mut response = Fetch::Request(request).send().await?;

    if response.status_code() == 404 {
        return Ok(None);
    }

    if response.status_code() != 200 {
        return Err(Error::RustError(format!(
            "Failed to fetch {}: status {}",
            url,
            response.status_code()
        )));
    }

    match response.json().await {
        Ok(json) => Ok(Some(json)),
        Err(e) => Err(e),
    }
}

pub fn normalize_title_translate(
    tt: &crate::model::TitleTranslate,
) -> crate::model::TitleTranslate {
    let mut normalized = crate::model::TitleTranslate::new();
    for (lang, titles) in tt {
        let iso_lang = match lang.as_str() {
            "zh-Hans" => "CN",
            "zh-Hant" => "TW",
            "ja" => "JP",
            "en" => "US",
            _ => lang.as_str(),
        };
        normalized
            .entry(iso_lang.to_string())
            .or_default()
            .extend(titles.clone());
    }
    for titles in normalized.values_mut() {
        titles.sort();
        titles.dedup();
    }
    normalized
}

pub fn now_utc() -> time::OffsetDateTime {
    let millis = Date::now().as_millis();
    time::OffsetDateTime::from_unix_timestamp_nanos((millis as i128) * 1_000_000).unwrap()
}

pub fn now_utc_ms() -> i64 {
    Date::now().as_millis() as i64
}

pub mod season;
