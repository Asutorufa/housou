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
