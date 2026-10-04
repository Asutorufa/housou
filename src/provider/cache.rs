use super::{MetadataArgs, MetadataRequest, ProviderFetch};
use crate::config;
use crate::db::{AppDatabase, Database, MetadataCacheEntry, MetadataCacheWrite};
use crate::model::{MetadataSource, UnifiedMetadata};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use worker::{Cache, Context, D1Database, Env, Error, Response, Result, RouteContext};

const EDGE_CACHE_NAME: &str = "housou-metadata-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RefreshReason {
    Full,
    TmdbOnly,
}

fn ttl_ms(seconds: i32) -> i64 {
    i64::from(seconds) * 1_000
}

fn now_ms() -> i64 {
    crate::utils::now_utc_ms()
}

fn metadata_cache_key(req: &MetadataRequest) -> String {
    let identity = if let Some(tmdb_id) = req.tmdb_id.as_deref() {
        serde_json::json!(["tmdb", tmdb_id])
    } else if let Some(mal_id) = req.mal_id.as_deref() {
        serde_json::json!(["mal", mal_id])
    } else if let Some(anilist_id) = req.anilist_id.as_deref() {
        serde_json::json!(["anilist", anilist_id])
    } else {
        serde_json::json!([
            "title",
            req.title.as_deref().unwrap_or("").trim(),
            req.year
        ])
    };

    let encoded = serde_json::to_vec(&identity).unwrap_or_default();
    format!("v2-{}", hex::encode(Sha256::digest(encoded)))
}

fn edge_cache_key(cache_origin: &str, cache_key: &str) -> String {
    format!("{cache_origin}/__metadata_cache/{cache_key}")
}

fn source_name(metadata: &UnifiedMetadata) -> &'static str {
    match &metadata.source {
        MetadataSource::Tmdb(_) => "tmdb",
        MetadataSource::Mal(_) => "mal",
        MetadataSource::Anilist(_) => "anilist",
        MetadataSource::Bangumi(_) => "bangumi",
        MetadataSource::AniDB(_) => "anidb",
        MetadataSource::Bilibili(_) => "bilibili",
        MetadataSource::AcFun(_) => "acfun",
    }
}

fn cached_metadata(entry: &MetadataCacheEntry) -> Option<UnifiedMetadata> {
    entry
        .metadata_json
        .as_deref()
        .and_then(|json| serde_json::from_str(json).ok())
}

fn refresh_reason(entry: &MetadataCacheEntry, now: i64) -> Option<RefreshReason> {
    entry.metadata_json.as_ref()?;

    let retry_active = entry.retry_after.is_some_and(|deadline| deadline > now);
    let refresh_due = entry.refresh_after.is_none_or(|deadline| deadline <= now);

    if refresh_due {
        return (!retry_active).then_some(RefreshReason::Full);
    }

    let tmdb_retry_due = entry.source.as_deref() != Some("tmdb")
        && entry.retry_after.is_some_and(|deadline| deadline <= now);

    tmdb_retry_due.then_some(RefreshReason::TmdbOnly)
}

fn edge_ttl_seconds(entry: &MetadataCacheEntry, now: i64) -> i32 {
    let mut deadline = now + ttl_ms(config::CACHE_TTL_METADATA_L1);

    if let Some(refresh_after) = entry.refresh_after
        && refresh_after > now
    {
        deadline = deadline.min(refresh_after);
    }

    if let Some(retry_after) = entry.retry_after
        && retry_after > now
    {
        deadline = deadline.min(retry_after);
    }

    ((deadline - now) / 1_000).clamp(1, i64::from(config::CACHE_TTL_METADATA_L1)) as i32
}

fn metadata_args(req: &MetadataRequest) -> MetadataArgs<'_> {
    MetadataArgs {
        tmdb_id: req.tmdb_id.as_deref(),
        mal_id: req.mal_id.as_deref(),
        anilist_id: req.anilist_id.as_deref(),
        title: req.title.as_deref(),
        year: req.year,
    }
}

fn schedule_edge_put(
    ctx: &RouteContext<Context>,
    cache: Cache,
    cache_key: String,
    metadata: &UnifiedMetadata,
    ttl: i32,
) -> Result<()> {
    let mut response = Response::from_json(metadata)?;
    response
        .headers_mut()
        .set("Cache-Control", &format!("public, max-age={ttl}"))?;

    ctx.data.wait_until(async move {
        if let Err(error) = cache.put(&cache_key, response).await {
            worker::console_warn!("Failed to populate metadata edge cache: {:?}", error);
        }
    });
    Ok(())
}

async fn store_positive(
    db: &AppDatabase<D1Database>,
    cache_key: &str,
    refresh_token: &str,
    outcome: &ProviderFetch,
    now: i64,
) -> Result<Option<MetadataCacheEntry>> {
    let metadata_json = serde_json::to_string(&outcome.metadata)
        .map_err(|error| Error::RustError(format!("Failed to serialize metadata: {error}")))?;
    let source = source_name(&outcome.metadata).to_string();
    let refresh_after = now + ttl_ms(config::CACHE_TTL_METADATA_D1);
    let retry_after = outcome
        .tmdb_failed
        .then_some(now + ttl_ms(config::CACHE_TTL_METADATA_MISS));

    db.store_metadata_cache(
        cache_key,
        refresh_token,
        MetadataCacheWrite {
            metadata_json: Some(&metadata_json),
            source: Some(&source),
            fetched_at: Some(now),
            refresh_after: Some(refresh_after),
            retry_after,
        },
    )
    .await
}

async fn store_negative(
    db: &AppDatabase<D1Database>,
    cache_key: &str,
    refresh_token: &str,
    now: i64,
) -> Result<Option<MetadataCacheEntry>> {
    db.store_metadata_cache(
        cache_key,
        refresh_token,
        MetadataCacheWrite {
            metadata_json: None,
            source: None,
            fetched_at: Some(now),
            refresh_after: None,
            retry_after: Some(now + ttl_ms(config::CACHE_TTL_METADATA_MISS)),
        },
    )
    .await
}

async fn acquire_refresh_lease(
    db: &AppDatabase<D1Database>,
    cache_key: &str,
    now: i64,
) -> Result<Option<String>> {
    let refresh_token = Uuid::new_v4().to_string();
    let refreshing_until = now + ttl_ms(config::CACHE_TTL_METADATA_REFRESH_LEASE);
    if db
        .try_acquire_metadata_refresh(cache_key, &refresh_token, now, refreshing_until)
        .await?
    {
        Ok(Some(refresh_token))
    } else {
        Ok(None)
    }
}

async fn fetch_with_lease(
    req: &MetadataRequest,
    ctx: &RouteContext<Context>,
    db: &AppDatabase<D1Database>,
    cache: Cache,
    d1_cache_key: &str,
    edge_key: String,
) -> Result<UnifiedMetadata> {
    let now = now_ms();
    let Some(refresh_token) = acquire_refresh_lease(db, d1_cache_key, now).await? else {
        if let Some(entry) = db.get_metadata_cache(d1_cache_key).await? {
            if let Some(metadata) = cached_metadata(&entry) {
                return Ok(metadata);
            }
            if entry.retry_after.is_some_and(|deadline| deadline > now) {
                return Err(Error::RustError(
                    "Metadata lookup is temporarily cached as unavailable".into(),
                ));
            }
        }
        return Err(Error::RustError(
            "Metadata lookup is already in progress".into(),
        ));
    };

    match super::fetch_metadata_from_providers(metadata_args(req), &ctx.env).await {
        Ok(outcome) => {
            let metadata = outcome.metadata.clone();
            match store_positive(db, d1_cache_key, &refresh_token, &outcome, now).await {
                Ok(Some(entry)) => {
                    schedule_edge_put(
                        ctx,
                        cache,
                        edge_key,
                        &metadata,
                        edge_ttl_seconds(&entry, now),
                    )?;
                }
                Ok(None) => {
                    worker::console_log!(
                        "Metadata refresh lease expired before the result could be stored"
                    );
                }
                Err(error) => {
                    worker::console_warn!("Failed to persist metadata in D1: {:?}", error);
                    let _ = db
                        .release_metadata_refresh(d1_cache_key, &refresh_token)
                        .await;
                }
            }
            Ok(metadata)
        }
        Err(error) => {
            if let Err(cache_error) = store_negative(db, d1_cache_key, &refresh_token, now).await {
                worker::console_warn!(
                    "Failed to persist negative metadata lookup in D1: {:?}",
                    cache_error
                );
                let _ = db
                    .release_metadata_refresh(d1_cache_key, &refresh_token)
                    .await;
            }
            Err(error)
        }
    }
}

async fn fetch_without_d1(
    req: &MetadataRequest,
    ctx: &RouteContext<Context>,
    cache: Cache,
    edge_key: String,
) -> Result<UnifiedMetadata> {
    let outcome = super::fetch_metadata_from_providers(metadata_args(req), &ctx.env).await?;
    let metadata = outcome.metadata;
    schedule_edge_put(
        ctx,
        cache,
        edge_key,
        &metadata,
        config::CACHE_TTL_METADATA_L1,
    )?;
    Ok(metadata)
}

fn schedule_refresh(
    req: &MetadataRequest,
    ctx: &RouteContext<Context>,
    d1_cache_key: String,
    reason: RefreshReason,
) {
    let req = req.clone();
    let env = ctx.env.clone();
    ctx.data.wait_until(async move {
        if let Err(error) = refresh_cached_metadata(req, env, d1_cache_key, reason).await {
            worker::console_warn!("Background metadata refresh failed: {:?}", error);
        }
    });
}

async fn refresh_cached_metadata(
    req: MetadataRequest,
    env: Env,
    cache_key: String,
    reason: RefreshReason,
) -> Result<()> {
    let d1 = env.d1("DB")?;
    let db = AppDatabase::new(d1);
    let now = now_ms();
    let Some(refresh_token) = acquire_refresh_lease(&db, &cache_key, now).await? else {
        return Ok(());
    };

    match reason {
        RefreshReason::TmdbOnly => {
            match super::fetch_tmdb_metadata(metadata_args(&req), &env).await {
                Some(Ok(metadata)) => {
                    let outcome = ProviderFetch {
                        metadata,
                        tmdb_failed: false,
                    };
                    match store_positive(&db, &cache_key, &refresh_token, &outcome, now).await {
                        Ok(Some(_)) | Ok(None) => {}
                        Err(error) => {
                            let _ = db
                                .release_metadata_refresh(&cache_key, &refresh_token)
                                .await;
                            return Err(error);
                        }
                    }
                }
                Some(Err(error)) => {
                    worker::console_log!("TMDb preferred-source retry failed: {:?}", error);
                    db.defer_metadata_retry(
                        &cache_key,
                        &refresh_token,
                        now + ttl_ms(config::CACHE_TTL_METADATA_MISS),
                    )
                    .await?;
                }
                None => {
                    db.defer_metadata_retry(
                        &cache_key,
                        &refresh_token,
                        now + ttl_ms(config::CACHE_TTL_METADATA_MISS),
                    )
                    .await?;
                }
            }
        }
        RefreshReason::Full => {
            match super::fetch_metadata_from_providers(metadata_args(&req), &env).await {
                Ok(outcome) => {
                    match store_positive(&db, &cache_key, &refresh_token, &outcome, now).await {
                        Ok(Some(_)) | Ok(None) => {}
                        Err(error) => {
                            let _ = db
                                .release_metadata_refresh(&cache_key, &refresh_token)
                                .await;
                            return Err(error);
                        }
                    }
                }
                Err(error) => {
                    worker::console_log!(
                        "Metadata refresh failed; keeping stale D1 data: {:?}",
                        error
                    );
                    db.defer_metadata_retry(
                        &cache_key,
                        &refresh_token,
                        now + ttl_ms(config::CACHE_TTL_METADATA_MISS),
                    )
                    .await?;
                }
            }
        }
    }

    Ok(())
}

pub(super) async fn fetch_metadata(
    req: &MetadataRequest,
    ctx: &RouteContext<Context>,
    cache_origin: &str,
) -> Result<UnifiedMetadata> {
    let d1_cache_key = metadata_cache_key(req);
    let edge_key = edge_cache_key(cache_origin, &d1_cache_key);
    let cache = Cache::open(EDGE_CACHE_NAME.to_string()).await;

    if let Ok(Some(mut response)) = cache.get(&edge_key, true).await
        && let Ok(metadata) = response.json::<UnifiedMetadata>().await
    {
        return Ok(metadata);
    }

    if let Ok(d1) = ctx.env.d1("DB") {
        let db = AppDatabase::new(d1);
        match db.get_metadata_cache(&d1_cache_key).await {
            Ok(Some(entry)) => {
                let now = now_ms();

                if let Some(metadata) = cached_metadata(&entry) {
                    if let Some(reason) = refresh_reason(&entry, now) {
                        schedule_refresh(req, ctx, d1_cache_key, reason);
                        return Ok(metadata);
                    }

                    schedule_edge_put(
                        ctx,
                        cache,
                        edge_key,
                        &metadata,
                        edge_ttl_seconds(&entry, now),
                    )?;
                    return Ok(metadata);
                }

                if entry.metadata_json.is_none()
                    && entry.retry_after.is_some_and(|deadline| deadline > now)
                {
                    return Err(Error::RustError(
                        "Metadata lookup is temporarily cached as unavailable".into(),
                    ));
                }

                return fetch_with_lease(req, ctx, &db, cache, &d1_cache_key, edge_key).await;
            }
            Ok(None) => {
                return fetch_with_lease(req, ctx, &db, cache, &d1_cache_key, edge_key).await;
            }
            Err(error) => {
                worker::console_warn!(
                    "D1 metadata cache read failed; falling back to providers: {:?}",
                    error
                );
            }
        }
    }

    fetch_without_d1(req, ctx, cache, edge_key).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(
        source: Option<&str>,
        refresh_after: Option<i64>,
        retry_after: Option<i64>,
    ) -> MetadataCacheEntry {
        MetadataCacheEntry {
            cache_key: "key".into(),
            metadata_json: Some("{}".into()),
            source: source.map(str::to_string),
            fetched_at: Some(1),
            refresh_after,
            retry_after,
            refreshing_until: None,
            refresh_token: None,
        }
    }

    #[test]
    fn cache_key_ignores_request_id() {
        let mut request = MetadataRequest {
            request_id: Some("one".into()),
            title: Some("Test".into()),
            tmdb_id: Some("tv/1".into()),
            mal_id: Some("2".into()),
            anilist_id: Some("3".into()),
            year: Some(2026),
        };
        let first = metadata_cache_key(&request);
        request.request_id = Some("two".into());
        assert_eq!(first, metadata_cache_key(&request));

        request.year = Some(2027);
        request.title = Some("Renamed".into());
        assert_eq!(first, metadata_cache_key(&request));

        request.tmdb_id = None;
        let id_based = metadata_cache_key(&request);
        request.mal_id = None;
        request.anilist_id = None;
        let title_based = metadata_cache_key(&request);
        assert_ne!(id_based, title_based);

        request.title = Some("Another title".into());
        assert_ne!(title_based, metadata_cache_key(&request));
    }

    #[test]
    fn fallback_retries_tmdb_after_negative_ttl() {
        let cached = entry(Some("mal"), Some(10_000), Some(5_000));
        assert_eq!(refresh_reason(&cached, 4_999), None);
        assert_eq!(
            refresh_reason(&cached, 5_000),
            Some(RefreshReason::TmdbOnly)
        );
    }

    #[test]
    fn full_refresh_waits_for_backoff() {
        let cached = entry(Some("tmdb"), Some(5_000), Some(6_000));
        assert_eq!(refresh_reason(&cached, 5_500), None);
        assert_eq!(refresh_reason(&cached, 6_000), Some(RefreshReason::Full));
    }

    #[test]
    fn edge_ttl_is_capped_by_earliest_refresh_deadline() {
        let now = 1_000;
        let cached = entry(Some("mal"), Some(now + 60_000), Some(now + 30_000));
        assert_eq!(edge_ttl_seconds(&cached, now), 30);
    }
}
