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

export function MetadataProvider({ children }: MetadataProviderProps) {
  const queue = useRef<
    {
      req: MetadataRequest;
      resolve: (value: UnifiedMetadata | null) => void;
      reject: (reason?: unknown) => void;
    }[]
  >([]);
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const flush = useCallback(async () => {
    const currentQueue = [...queue.current];
    // Clear queue immediately
    queue.current = [];
    timeoutRef.current = null;

    if (currentQueue.length === 0) return;

    try {
      const requests = currentQueue.map((item) => item.req);
      const response = await fetch("/api/metadata", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(requests),
      });

      if (!response.ok) {
        throw new Error("Batch metadata fetch failed");
      }

      const results: (UnifiedMetadata | null)[] = await response.json();

      if (results.length !== currentQueue.length) {
        console.warn(
          `Metadata batch result length mismatch: sent ${currentQueue.length}, got ${results.length}`,
        );
      }

      currentQueue.forEach((item, index) => {
        if (index < results.length) {
          item.resolve(results[index]);
        } else {
          item.resolve(null);
        }
      });
    } catch (err) {
      console.error("Batch fetch error:", err);
      // Resolve with null to prevent hanging promises
      currentQueue.forEach((item) => item.resolve(null));
    }
  }, []);

  const fetchMetadata = useCallback(
    (req: MetadataRequest): Promise<UnifiedMetadata | null> => {
      return new Promise((resolve, reject) => {
        queue.current.push({ req, resolve, reject });

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
