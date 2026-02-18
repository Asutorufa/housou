import { createContext, useCallback, useContext, useMemo, useRef } from "react";
import type { MetadataRequest, UnifiedMetadata } from "../types";

interface MetadataContextType {
  fetchMetadata: (req: MetadataRequest) => Promise<UnifiedMetadata | null>;
}

const MetadataContext = createContext<MetadataContextType | null>(null);

// eslint-disable-next-line react-refresh/only-export-components
export function useMetadata() {
  const context = useContext(MetadataContext);
  if (!context) {
    throw new Error("useMetadata must be used within a MetadataProvider");
  }
  return context;
}

interface MetadataProviderProps {
  children: React.ReactNode;
}

interface QueuedRequest {
  id: string;
  req: MetadataRequest;
  resolve: (value: UnifiedMetadata | null) => void;
  reject: (reason?: unknown) => void;
}

interface BatchResponseItem {
  request_id?: string;
  metadata?: UnifiedMetadata | null;
}

const MAX_BATCH_SIZE = 10;
const DEBOUNCE_MS = 120;

/** Generate a stable cache key from request parameters */
function makeCacheKey(req: MetadataRequest): string {
  return JSON.stringify([
    req.title,
    req.tmdb_id ?? "",
    req.mal_id ?? "",
    req.anilist_id ?? "",
    req.year ?? "",
  ]);
}

export function MetadataProvider({ children }: MetadataProviderProps) {
  const queue = useRef<QueuedRequest[]>([]);
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const flushingRef = useRef(false);
  // Cache: maps request key → promise, so duplicate requests reuse the same fetch
  const cacheRef = useRef<Map<string, Promise<UnifiedMetadata | null>>>(
    new Map(),
  );

  const scheduleFlush = useCallback((fn: () => Promise<void>) => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }
    timeoutRef.current = setTimeout(fn, DEBOUNCE_MS);
  }, []);

  const flush = useCallback(async () => {
    timeoutRef.current = null;

    // If a flush is already in progress, don't start another one.
    // The in-progress flush will re-check the queue when it completes.
    if (flushingRef.current) {
      return;
    }

    const pending = queue.current;
    if (pending.length === 0) {
      return;
    }

    flushingRef.current = true;

    const currentBatch = pending.slice(0, MAX_BATCH_SIZE);
    const remaining = pending.slice(MAX_BATCH_SIZE);

    // Update queue to remaining items
    queue.current = remaining;

    try {
      const requests = currentBatch.map((item) => ({
        ...item.req,
        request_id: item.id,
      }));

      const response = await fetch("/api/metadata", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(requests),
      });

      if (!response.ok) {
        throw new Error("Batch metadata fetch failed");
      }

      const results: BatchResponseItem[] = await response.json();

      // Create map for O(1) lookup
      const resultMap = new Map<string, UnifiedMetadata | null>();
      results.forEach((res) => {
        if (res.request_id) {
          resultMap.set(res.request_id, res.metadata || null);
        }
      });

      currentBatch.forEach((item) => {
        if (resultMap.has(item.id)) {
          item.resolve(resultMap.get(item.id) || null);
        } else {
          item.resolve(null);
        }
      });
    } catch (err) {
      console.error("Batch fetch error:", err);
      // Resolve with null to prevent hanging promises
      currentBatch.forEach((item) => item.resolve(null));
    } finally {
      flushingRef.current = false;

      // After completing a flush, check if more requests accumulated
      // during the fetch (e.g. user scrolled while we were waiting).
      // Use a short debounce to catch any final stragglers.
      if (queue.current.length > 0) {
        scheduleFlush(flush);
      }
    }
  }, [scheduleFlush]);

  const fetchMetadata = useCallback(
    (req: MetadataRequest): Promise<UnifiedMetadata | null> => {
      // Deduplicate: if the same request is already in-flight or cached, reuse it
      const key = makeCacheKey(req);
      const cached = cacheRef.current.get(key);
      if (cached) {
        return cached;
      }

      const promise = new Promise<UnifiedMetadata | null>((resolve) => {
        const id =
          Date.now().toString(36) + Math.random().toString(36).substring(2);
        queue.current.push({ id, req, resolve, reject: () => resolve(null) });

        // Only schedule a flush if one isn't already in progress.
        // If a flush IS in progress, it will pick up queued items
        // when it completes (see the finally block in flush).
        if (!flushingRef.current) {
          if (queue.current.length >= MAX_BATCH_SIZE) {
            // Full batch ready — flush immediately
            if (timeoutRef.current) {
              clearTimeout(timeoutRef.current);
              timeoutRef.current = null;
            }
            flush();
          } else {
            scheduleFlush(flush);
          }
        }
      });

      cacheRef.current.set(key, promise);
      return promise;
    },
    [flush, scheduleFlush],
  );

  const value = useMemo(() => ({ fetchMetadata }), [fetchMetadata]);

  return (
    <MetadataContext.Provider value={value}>
      {children}
    </MetadataContext.Provider>
  );
}
