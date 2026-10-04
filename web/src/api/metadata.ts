import type { MetadataRequest, UnifiedMetadata } from "../types";
import { checkResponse } from "../utils/fetcher";

type Priority = "normal" | "detail";
interface BatchResult {
  request_id: string;
  metadata: UnifiedMetadata | null;
}
interface Pending {
  id: string;
  key: string;
  request: MetadataRequest;
  priority: Priority;
  resolve: (metadata: UnifiedMetadata | null) => void;
  reject: (error: unknown) => void;
}
interface CacheEntry {
  id: string;
  promise: Promise<UnifiedMetadata | null>;
  expiresAt: number;
}
const BATCH_SIZE = 10;
const NORMAL_CONCURRENCY = 1;
const DETAIL_CONCURRENCY = 1;
const REQUEST_TIMEOUT_MS = 30_000;
const CACHE_TTL_MS = 30 * 60_000;
const CACHE_SIZE = 500;

// Owns batching and caching independently of React. In-flight requests are
// deduplicated; failed/empty responses are never retained as successful data.
export function createMetadataClient() {
  const cache = new Map<string, CacheEntry>();
  const controllers = new Set<AbortController>();
  let queue: Pending[] = [];
  let timer: ReturnType<typeof setTimeout> | undefined;
  let activeNormal = 0;
  let activeDetail = 0;
  let nextId = 0;

  function trimCache() {
    for (const [key, entry] of cache) {
      if (cache.size <= CACHE_SIZE) break;
      if (entry.expiresAt !== Infinity) cache.delete(key);
    }
  }

  function takeBatch(priority: Priority) {
    const batch: Pending[] = [];
    const remaining: Pending[] = [];
    for (const item of queue) {
      if (item.priority === priority && batch.length < BATCH_SIZE) {
        batch.push(item);
      } else {
        remaining.push(item);
      }
    }
    queue = remaining;
    return batch;
  }

  function schedule() {
    if (queue.length === 0) return;

    const hasDetail = queue.some((item) => item.priority === "detail");
    const normalCount = queue.reduce(
      (count, item) => count + Number(item.priority === "normal"),
      0,
    );

    if (hasDetail || normalCount >= BATCH_SIZE) {
      pump();
    } else if (!timer) {
      // Use a fixed batching window; continuous scrolling cannot postpone it.
      timer = setTimeout(pump, 120);
    }
  }

  function pump() {
    clearTimeout(timer);
    timer = undefined;

    // Keep ordinary card loading serialized, matching the pre-refactor behavior.
    // A detail request gets its own lane so opening a card is never stuck behind
    // a large list batch, while we still avoid firing two heavy list batches at once.
    if (activeDetail < DETAIL_CONCURRENCY) {
      const batch = takeBatch("detail");
      if (batch.length > 0) {
        activeDetail++;
        void send(batch, "detail");
      }
    }

    if (activeNormal < NORMAL_CONCURRENCY) {
      const batch = takeBatch("normal");
      if (batch.length > 0) {
        activeNormal++;
        void send(batch, "normal");
      }
    }
  }

  async function send(batch: Pending[], priority: Priority) {
    const controller = new AbortController();
    controllers.add(controller);
    const timeout = setTimeout(() => controller.abort(), REQUEST_TIMEOUT_MS);
    try {
      const response = await checkResponse(
        await fetch("/api/metadata", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(
            batch.map((item) => ({ ...item.request, request_id: item.id })),
          ),
          signal: controller.signal,
        }),
      );
      const results: BatchResult[] = await response.json();
      const byId = new Map(
        results.map((item) => [item.request_id, item.metadata]),
      );
      for (const item of batch) {
        if (!byId.has(item.id)) throw new Error("Incomplete metadata response");
      }
      for (const item of batch) {
        const metadata = byId.get(item.id) ?? null;
        const entry = cache.get(item.key);
        if (metadata && entry?.id === item.id)
          entry.expiresAt = Date.now() + CACHE_TTL_MS;
        else if (entry?.id === item.id) cache.delete(item.key);
        item.resolve(metadata);
      }
      trimCache();
    } catch (error) {
      for (const item of batch) {
        if (cache.get(item.key)?.id === item.id) cache.delete(item.key);
        item.reject(error);
      }
    } finally {
      clearTimeout(timeout);
      controllers.delete(controller);
      if (priority === "detail") activeDetail--;
      else activeNormal--;
      // Queued work has already waited for a batch; do not add another debounce.
      pump();
    }
  }

  function fetchMetadata(
    request: MetadataRequest,
    priority: Priority = "normal",
  ): Promise<UnifiedMetadata | null> {
    const key = JSON.stringify([
      request.title ?? "",
      request.tmdb_id ?? "",
      request.mal_id ?? "",
      request.anilist_id ?? "",
      request.bangumi_id ?? "",
      request.year ?? "",
      request.media_type ?? "",
      request.aliases ?? [],
    ]);
    const existing = cache.get(key);
    if (existing && existing.expiresAt > Date.now()) {
      if (priority === "detail") {
        const pending = queue.find((item) => item.key === key);
        if (pending) {
          pending.priority = priority;
          schedule();
        }
      }
      return existing.promise;
    }
    cache.delete(key);
    const id = String(++nextId);
    const promise = new Promise<UnifiedMetadata | null>((resolve, reject) => {
      queue.push({ id, key, request, priority, resolve, reject });
    });
    cache.set(key, { id, promise, expiresAt: Infinity });
    // Evict completed entries only, preserving in-flight deduplication.
    trimCache();
    schedule();
    return promise;
  }

  function dispose() {
    clearTimeout(timer);
    timer = undefined;
    const error = new Error("Metadata provider unmounted");
    for (const item of queue) item.reject(error);
    queue = [];
    for (const controller of controllers) controller.abort();
    cache.clear();
  }
  return { fetchMetadata, dispose };
}
