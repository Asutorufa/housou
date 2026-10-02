import {
  createContext,
  useContext,
  useEffect,
  useMemo,
  type ReactNode,
} from "react";
import { createMetadataClient } from "../api/metadata";

const MetadataContext = createContext<Pick<
  ReturnType<typeof createMetadataClient>,
  "fetchMetadata"
> | null>(null);

// eslint-disable-next-line react-refresh/only-export-components
export function useMetadata() {
  const context = useContext(MetadataContext);
  if (!context)
    throw new Error("useMetadata must be used within a MetadataProvider");
  return context;
}
export function MetadataProvider({ children }: { children: ReactNode }) {
  const client = useMemo(() => createMetadataClient(), []);
  useEffect(() => () => client.dispose(), [client]);
  return (
    <MetadataContext.Provider value={client}>
      {children}
    </MetadataContext.Provider>
  );
}
