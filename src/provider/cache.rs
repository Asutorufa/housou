use super::{MetadataArgs, MetadataRequest, ProviderFetch};
use crate::config;
use crate::db::{AppDatabase, Database, MetadataCacheEntry, MetadataCacheWrite};
use crate::model::{MetadataSource, UnifiedMetadata};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use worker::{Cache, Context, D1Database, Env, Error, Response, Result, RouteContext};

const EDGE_CACHE_NAME: &str = "housou-metadata-v3";
const EDGE_EXPIRY_HEADER: &str = "X-Housou-Metadata-Expires-At";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RefreshReason {
    Full,
    TmdbOnly,
}

struct RefreshLease {
    token: String,
    entry: Option<MetadataCacheEntry>,
}

fn ttl_ms(seconds: i32) -> i64 {
    i64::from(seconds) * 1_000
}

fn now_ms() -> i64 {
    crate::utils::now_utc_ms()
}

fn metadata_cache_key(req: &MetadataRequest) -> String {
    let (version, identity) = if let Some(tmdb_id) = req.tmdb_id.as_deref() {
        ("v2", serde_json::json!(["tmdb", tmdb_id]))
    } else if let Some(mal_id) = req.mal_id.as_deref() {
        ("v2", serde_json::json!(["mal", mal_id]))
    } else if let Some(anilist_id) = req.anilist_id.as_deref() {
        ("v2", serde_json::json!(["anilist", anilist_id]))
    } else if let Some(bangumi_id) = req.bangumi_id.as_deref() {
        ("v3", serde_json::json!(["bangumi", bangumi_id]))
    } else {
        (
            "v4",
            serde_json::json!([
                "title",
                req.title.as_deref().unwrap_or("").trim(),
                req.year,
                req.media_type.as_deref().unwrap_or(""),
                req.aliases
            ]),
        )
    };

    let encoded = serde_json::to_vec(&identity).unwrap_or_default();
    format!("{version}-{}", hex::encode(Sha256::digest(encoded)))
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

fn edge_expires_at(entry: &MetadataCacheEntry, now: i64) -> i64 {
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

    deadline
}

fn remaining_edge_ttl(expires_at: i64, now: i64) -> Option<i32> {
    let seconds = (expires_at - now) / 1_000;
    (seconds > 0).then(|| seconds.min(i64::from(config::CACHE_TTL_METADATA_L1)) as i32)
}

fn metadata_args(req: &MetadataRequest) -> MetadataArgs<'_> {
    MetadataArgs {
        tmdb_id: req.tmdb_id.as_deref(),
        mal_id: req.mal_id.as_deref(),
        anilist_id: req.anilist_id.as_deref(),
        bangumi_id: req.bangumi_id.as_deref(),
        title: req.title.as_deref(),
        aliases: &req.aliases,
        year: req.year,
        media_type: req.media_type.as_deref(),
    }
}

fn request_for_refresh(req: &MetadataRequest, metadata: &UnifiedMetadata) -> MetadataRequest {
    let mut req = req.clone();
    match &metadata.source {
        MetadataSource::Tmdb(id) => req.tmdb_id = Some(id.clone()),
        MetadataSource::Mal(id) if req.mal_id.is_none() => req.mal_id = Some(id.clone()),
        MetadataSource::Anilist(id) if req.anilist_id.is_none() => {
            req.anilist_id = Some(id.clone());
        }
        MetadataSource::Bangumi(id) if req.bangumi_id.is_none() => {
            req.bangumi_id = Some(id.clone());
        }
        _ => {}
    }
    req
}

fn schedule_edge_put(
    ctx: &RouteContext<Context>,
    cache: Cache,
    cache_key: String,
    metadata: &UnifiedMetadata,
    expires_at: i64,
) -> Result<()> {
    let mut response = Response::from_json(metadata)?;
    response
        .headers_mut()
        .set(EDGE_EXPIRY_HEADER, &expires_at.to_string())?;

    ctx.data.wait_until(async move {
        // Cache-Control starts counting at insertion, so a deferred write must
        // keep the absolute deadline from the D1 row it originally read.
        let Some(ttl) = remaining_edge_ttl(expires_at, now_ms()) else {
            return;
        };
        if let Err(error) = response
            .headers_mut()
            .set("Cache-Control", &format!("public, max-age={ttl}"))
        {
            worker::console_warn!("Failed to set metadata edge TTL: {:?}", error);
            return;
        }
        if let Err(error) = cache.put(&cache_key, response).await {
            worker::console_warn!("Failed to populate metadata edge cache: {:?}", error);
        }
    });
    Ok(())
}

async fn store_positive(
    db: &impl Database,
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
        .tmdb_retry_after_seconds
        .map(|seconds| now + ttl_ms(seconds));

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
    db: &impl Database,
    cache_key: &str,
    refresh_token: &str,
    now: i64,
    retry_after_seconds: i32,
) -> Result<Option<MetadataCacheEntry>> {
    db.store_metadata_cache(
        cache_key,
        refresh_token,
        MetadataCacheWrite {
            metadata_json: None,
            source: None,
            fetched_at: Some(now),
            refresh_after: None,
            retry_after: Some(now + ttl_ms(retry_after_seconds)),
        },
    )
    .await
}

async fn acquire_refresh_lease(
    db: &impl Database,
    cache_key: &str,
    now: i64,
) -> Result<Option<RefreshLease>> {
    let refresh_token = Uuid::new_v4().to_string();
    let refreshing_until = now + ttl_ms(config::CACHE_TTL_METADATA_REFRESH_LEASE);
    if db
        .try_acquire_metadata_refresh(cache_key, &refresh_token, now, refreshing_until)
        .await?
    {
        // A queued lookup may acquire the lease after another lookup has
        // already populated the cache or installed retry backoff.
        let entry = match db.get_metadata_cache(cache_key).await {
            Ok(entry) => entry,
            Err(error) => {
                let _ = db.release_metadata_refresh(cache_key, &refresh_token).await;
                return Err(error);
            }
        };
        if let Some(entry) = &entry {
            let lookup_due = if cached_metadata(entry).is_some() {
                refresh_reason(entry, now).is_some()
            } else {
                entry.retry_after.is_none_or(|deadline| deadline <= now)
            };
            if !lookup_due {
                db.release_metadata_refresh(cache_key, &refresh_token)
                    .await?;
                return Ok(None);
            }
        }
        Ok(Some(RefreshLease {
            token: refresh_token,
            entry,
        }))
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
    let Some(lease) = acquire_refresh_lease(db, d1_cache_key, now).await? else {
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

    let refresh_token = lease.token;
    let previous_metadata = lease.entry.as_ref().and_then(cached_metadata);
    let refresh_req = previous_metadata
        .as_ref()
        .map(|metadata| request_for_refresh(req, metadata));
    let req = refresh_req.as_ref().unwrap_or(req);

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
                        edge_expires_at(&entry, now),
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
        Err(failure) => {
            if let Some(metadata) = previous_metadata {
                if let Err(error) = db
                    .defer_metadata_retry(
                        d1_cache_key,
                        &refresh_token,
                        now_ms() + ttl_ms(failure.retry_after_seconds),
                    )
                    .await
                {
                    worker::console_warn!("Failed to defer metadata retry: {:?}", error);
                    let _ = db
                        .release_metadata_refresh(d1_cache_key, &refresh_token)
                        .await;
                }
                return Ok(metadata);
            }
            if let Err(cache_error) = store_negative(
                db,
                d1_cache_key,
                &refresh_token,
                now,
                failure.retry_after_seconds,
            )
            .await
            {
                worker::console_warn!(
                    "Failed to persist negative metadata lookup in D1: {:?}",
                    cache_error
                );
                let _ = db
                    .release_metadata_refresh(d1_cache_key, &refresh_token)
                    .await;
            }
            Err(failure.error)
        }
    }
}

async fn fetch_without_d1(
    req: &MetadataRequest,
    ctx: &RouteContext<Context>,
    cache: Cache,
    edge_key: String,
) -> Result<UnifiedMetadata> {
    let outcome = super::fetch_metadata_from_providers(metadata_args(req), &ctx.env)
        .await
        .map_err(|failure| failure.error)?;
    let metadata = outcome.metadata;
    let ttl = outcome
        .tmdb_retry_after_seconds
        .unwrap_or(config::CACHE_TTL_METADATA_L1)
        .min(config::CACHE_TTL_METADATA_L1);
    schedule_edge_put(ctx, cache, edge_key, &metadata, now_ms() + ttl_ms(ttl))?;
    Ok(metadata)
}

fn schedule_refresh(req: &MetadataRequest, ctx: &RouteContext<Context>, d1_cache_key: String) {
    let req = req.clone();
    let env = ctx.env.clone();
    ctx.data.wait_until(async move {
        if let Err(error) = refresh_cached_metadata(req, env, d1_cache_key).await {
            worker::console_warn!("Background metadata refresh failed: {:?}", error);
        }
    });
}

async fn refresh_cached_metadata(req: MetadataRequest, env: Env, cache_key: String) -> Result<()> {
    let d1 = env.d1("DB")?;
    let db = AppDatabase::new(d1);
    let now = now_ms();
    let Some(lease) = acquire_refresh_lease(&db, &cache_key, now).await? else {
        return Ok(());
    };
    // Use the current row under the lease, not the scheduler's earlier snapshot.
    let reason = lease
        .entry
        .as_ref()
        .and_then(|entry| refresh_reason(entry, now))
        .unwrap_or(RefreshReason::Full);
    let req = lease
        .entry
        .as_ref()
        .and_then(cached_metadata)
        .map(|metadata| request_for_refresh(&req, &metadata))
        .unwrap_or(req);
    let refresh_token = lease.token;

    match reason {
        RefreshReason::TmdbOnly => {
            match super::fetch_tmdb_metadata(metadata_args(&req), &env).await {
                Some(Ok(metadata)) => {
                    let outcome = ProviderFetch {
                        metadata,
                        tmdb_retry_after_seconds: None,
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
                        now + ttl_ms(super::provider_retry_after_seconds(&error)),
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
                Err(failure) => {
                    worker::console_log!(
                        "Metadata refresh failed; keeping stale D1 data: {:?}",
                        failure.error
                    );
                    db.defer_metadata_retry(
                        &cache_key,
                        &refresh_token,
                        now + ttl_ms(failure.retry_after_seconds),
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
        && response
            .headers()
            .get(EDGE_EXPIRY_HEADER)
            .ok()
            .flatten()
            .and_then(|value| value.parse::<i64>().ok())
            .is_some_and(|deadline| deadline > now_ms())
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
                    if refresh_reason(&entry, now).is_some() {
                        schedule_refresh(req, ctx, d1_cache_key);
                        return Ok(metadata);
                    }

                    schedule_edge_put(
                        ctx,
                        cache,
                        edge_key,
                        &metadata,
                        edge_expires_at(&entry, now),
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
    use d1_orm::sqlite::SqliteExecutor;

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
            media_type: Some("tv".into()),
            ..Default::default()
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
    fn refresh_reuses_resolved_tmdb_id() {
        let request = MetadataRequest {
            title: Some("Search Title".into()),
            ..Default::default()
        };
        let metadata = UnifiedMetadata {
            source: MetadataSource::Tmdb("tv/123/season/2".into()),
            ..Default::default()
        };

        let refreshed = request_for_refresh(&request, &metadata);
        assert_eq!(refreshed.tmdb_id.as_deref(), Some("tv/123/season/2"));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn aliases_can_retry_a_negatively_cached_title() -> Result<()> {
        let db = AppDatabase::new(SqliteExecutor::new_in_memory().unwrap());
        db.migrate().await?;
        let original = MetadataRequest {
            title: Some("Localized Title".into()),
            year: Some(2026),
            media_type: Some("tv".into()),
            ..Default::default()
        };
        let first_key = metadata_cache_key(&original);
        let token = acquire_refresh_lease(&db, &first_key, 1_000)
            .await?
            .unwrap()
            .token;
        store_negative(
            &db,
            &first_key,
            &token,
            1_000,
            config::CACHE_TTL_METADATA_MISS,
        )
        .await?;
        let mut enriched = original.clone();
        enriched.aliases = vec!["Original Japanese Title".into()];
        let enriched_key = metadata_cache_key(&enriched);
        assert!(db.get_metadata_cache(&enriched_key).await?.is_none());
        assert!(
            acquire_refresh_lease(&db, &enriched_key, 1_001)
                .await?
                .is_some()
        );
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn delayed_requests_respect_fresh_data_and_backoff() -> Result<()> {
        let db = AppDatabase::new(SqliteExecutor::new_in_memory().unwrap());
        db.migrate().await?;
        let token = acquire_refresh_lease(&db, "positive", 1_000)
            .await?
            .unwrap()
            .token;
        let outcome = ProviderFetch {
            metadata: UnifiedMetadata {
                source: MetadataSource::Tmdb("tv/1/season/2".into()),
                ..Default::default()
            },
            tmdb_retry_after_seconds: None,
        };
        store_positive(&db, "positive", &token, &outcome, 1_000).await?;
        // A delayed cold lookup or queued stale refresh must not fetch again.
        assert!(
            acquire_refresh_lease(&db, "positive", 1_001)
                .await?
                .is_none()
        );
        let refresh_at = 1_000 + ttl_ms(config::CACHE_TTL_METADATA_D1);
        let token = acquire_refresh_lease(&db, "positive", refresh_at)
            .await?
            .unwrap()
            .token;
        db.defer_metadata_retry("positive", &token, refresh_at + 60_000)
            .await?;
        assert!(
            acquire_refresh_lease(&db, "positive", refresh_at + 1)
                .await?
                .is_none()
        );
        assert!(
            acquire_refresh_lease(&db, "positive", refresh_at + 60_000)
                .await?
                .is_some()
        );

        let token = acquire_refresh_lease(&db, "negative", 1_000)
            .await?
            .unwrap()
            .token;
        store_negative(
            &db,
            "negative",
            &token,
            1_000,
            config::CACHE_TTL_METADATA_MISS,
        )
        .await?;
        assert!(
            acquire_refresh_lease(&db, "negative", 1_001)
                .await?
                .is_none()
        );
        assert!(
            acquire_refresh_lease(
                &db,
                "negative",
                1_000 + ttl_ms(config::CACHE_TTL_METADATA_MISS)
            )
            .await?
            .is_some()
        );
        Ok(())
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
        assert_eq!(
            remaining_edge_ttl(edge_expires_at(&cached, now), now),
            Some(30)
        );
    }

    #[test]
    fn deferred_edge_put_keeps_its_original_deadline() {
        let read_at = 1_000;
        let cached = entry(Some("mal"), Some(read_at + 60_000), Some(read_at + 30_000));
        let expires_at = edge_expires_at(&cached, read_at);
        assert_eq!(remaining_edge_ttl(expires_at, read_at + 20_000), Some(10));
        assert_eq!(remaining_edge_ttl(expires_at, expires_at), None);
        assert_eq!(remaining_edge_ttl(expires_at, expires_at + 5_000), None);
    }
}
