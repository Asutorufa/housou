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
const CONCURRENCY = 2;
const CACHE_TTL_MS = 30 * 60_000;
const CACHE_SIZE = 500;

// Owns batching and caching independently of React. In-flight requests are
// deduplicated; failed/empty responses are never retained as successful data.
export function createMetadataClient() {
  const cache = new Map<string, CacheEntry>();
  const controllers = new Set<AbortController>();
  let queue: Pending[] = [];
  let timer: ReturnType<typeof setTimeout> | undefined;
  let active = 0;
  let nextId = 0;

  function trimCache() {
    for (const [key, entry] of cache) {
      if (cache.size <= CACHE_SIZE) break;
      if (entry.expiresAt !== Infinity) cache.delete(key);
    }
  }

  function schedule() {
    if (queue.length === 0 || active >= CONCURRENCY) return;
    if (
      queue.length >= BATCH_SIZE ||
      queue.some((item) => item.priority === "detail")
    ) {
      pump();
    } else if (!timer) {
      // Use a fixed batching window; continuous scrolling cannot postpone it.
      timer = setTimeout(pump, 120);
    }
  }

  function pump() {
    clearTimeout(timer);
    timer = undefined;
    while (active < CONCURRENCY && queue.length > 0) {
      queue.sort(
        (a, b) =>
          Number(b.priority === "detail") - Number(a.priority === "detail"),
      );
      const batch = queue.splice(0, BATCH_SIZE);
      active++;
      void send(batch);
    }
  }

  async function send(batch: Pending[]) {
    const controller = new AbortController();
    controllers.add(controller);
    const timeout = setTimeout(() => controller.abort(), 15_000);
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
      active--;
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
      request.year ?? "",
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
