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

export function MetadataProvider({ children }: MetadataProviderProps) {
  const queue = useRef<QueuedRequest[]>([]);
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const flush = useCallback(async () => {
    // Process only up to MAX_BATCH_SIZE
    const pending = queue.current;
    if (pending.length === 0) {
      timeoutRef.current = null;
      return;
    }

    const currentBatch = pending.slice(0, MAX_BATCH_SIZE);
    const remaining = pending.slice(MAX_BATCH_SIZE);

    // Update queue to remaining items
    queue.current = remaining;

    // Reschedule flush if there are remaining items
    if (remaining.length > 0) {
      timeoutRef.current = setTimeout(flush, 100);
    } else {
      timeoutRef.current = null;
    }

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
          // If backend didn't return this ID (or returned old array format), fallback or resolve null
          // Assuming strict ID matching now for robustness
          item.resolve(null);
        }
      });
    } catch (err) {
      console.error("Batch fetch error:", err);
      // Resolve with null to prevent hanging promises
      currentBatch.forEach((item) => item.resolve(null));
    }
  }, []);

  const fetchMetadata = useCallback(
    (req: MetadataRequest): Promise<UnifiedMetadata | null> => {
      return new Promise((resolve, reject) => {
        const id = crypto.randomUUID();
        queue.current.push({ id, req, resolve, reject });

        if (!timeoutRef.current) {
          timeoutRef.current = setTimeout(flush, 100);
        }
      });
    },
    [flush],
  );

  const value = useMemo(() => ({ fetchMetadata }), [fetchMetadata]);

  return (
    <MetadataContext.Provider value={value}>
      {children}
    </MetadataContext.Provider>
  );
}
