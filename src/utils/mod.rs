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

pub fn normalize_title_translate(tt: &mut crate::model::TitleTranslate) {
    let targets = [
        ("zh-Hans", "CN"),
        ("zh-Hant", "TW"),
        ("ja", "JP"),
        ("en", "US"),
    ];

    for (old_lang, new_lang) in targets {
        if let Some(titles) = tt.remove(old_lang) {
            if let Some(existing) = tt.get_mut(new_lang) {
                existing.extend(titles);
            } else {
                tt.insert(new_lang.to_string(), titles);
            }
        }
    }

    for titles in tt.values_mut() {
        titles.sort();
        titles.dedup();
    }
}

pub fn now_utc() -> Result<time::OffsetDateTime> {
    let millis = Date::now().as_millis();
    time::OffsetDateTime::from_unix_timestamp_nanos((millis as i128) * 1_000_000)
        .map_err(|e| Error::RustError(format!("Invalid timestamp: {}", e)))
}

pub fn now_utc_ms() -> i64 {
    Date::now().as_millis() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_normalize_title_translate() {
        let mut tt = HashMap::new();
        tt.insert("zh-Hans".to_string(), vec!["A".to_string()]);
        tt.insert("CN".to_string(), vec!["B".to_string()]);
        tt.insert("ja".to_string(), vec!["C".to_string()]);
        tt.insert("en".to_string(), vec!["D".to_string()]);
        tt.insert("other".to_string(), vec!["E".to_string()]);

        normalize_title_translate(&mut tt);

        assert_eq!(
            tt.get("CN").unwrap(),
            &vec!["A".to_string(), "B".to_string()]
        );
        assert_eq!(tt.get("JP").unwrap(), &vec!["C".to_string()]);
        assert_eq!(tt.get("US").unwrap(), &vec!["D".to_string()]);
        assert_eq!(tt.get("other").unwrap(), &vec!["E".to_string()]);
        assert!(!tt.contains_key("zh-Hans"));
        assert!(!tt.contains_key("ja"));
        assert!(!tt.contains_key("en"));
    }
}

pub mod season;
