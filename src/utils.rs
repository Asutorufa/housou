use worker::*;

const CACHE_TTL_SECONDS: i32 = 21600; // 6 hours cache

pub async fn fetch_json<T: for<'de> serde::Deserialize<'de>>(url: &str) -> Result<T> {
    let mut init = RequestInit::new();
    init.with_method(Method::Get);

    let mut cf = CfProperties::new();
    let mut ttl_by_status = std::collections::HashMap::new();
    ttl_by_status.insert("200".to_string(), CACHE_TTL_SECONDS);
    ttl_by_status.insert("404".to_string(), 3600); // Cache 404s for 1 hour
    cf.cache_ttl_by_status = Some(ttl_by_status);
    init.with_cf_properties(cf);

    let request = Request::new_with_init(url, &init)?;
    let mut response = Fetch::Request(request).send().await?;

    if response.status_code() != 200 {
        return Err(Error::RustError(format!(
            "Failed to fetch {}: status {}",
            url,
            response.status_code()
        )));
    }

    response.json().await
}
