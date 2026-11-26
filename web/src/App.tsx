import { lazy, useCallback, useEffect, useRef, useState } from "react";
import { useLocation, useSearch } from "wouter";
import CardGridSkeleton from "./components/CardGridSkeleton";
import DeferredDialog from "./components/DeferredDialog";
import Footer from "./components/Footer";
import Header from "./components/Header";
import TabbedGrid from "./components/TabbedGrid";
import { useAnimeData } from "./hooks/useAnimeData";
import type { UnifiedMetadata } from "./types";
import {
  captureCardMorphSnapshot,
  type CardMorphSnapshot,
} from "./utils/coverCrop";

const AttributionModal = lazy(() => import("./components/AttributionModal"));
const loadDetailsModal = () => import("./components/DetailsModal");
const DetailsModal = lazy(loadDetailsModal);

export default function App() {
  const {
    config,
    selections,
    setSelections,
    searchQuery,
    setSearchQuery,
    items,
    filteredItems,
    loading,
    error,
    mutateStatuses,
  } = useAnimeData();

  const [location, setLocation] = useLocation();
  const search = useSearch();

  const [cachedInfo, setCachedInfo] = useState<{
    title: string;
    info: UnifiedMetadata | null;
    cardSnapshot: CardMorphSnapshot | null;
  } | null>(null);
  const [isAttributionOpen, setIsAttributionOpen] = useState(false);

  // Derive selected anime directly from URL
  const params = new URLSearchParams(search);
  const animeTitle = params.get("anime");
  const selectedAnime = animeTitle
    ? {
        title: animeTitle,
        info: cachedInfo?.title === animeTitle ? cachedInfo.info : null,
      }
    : null;

  // Delay deep-link modal opening by a frame until the grid has mounted.
  const [isGridReady, setIsGridReady] = useState(false);
  useEffect(() => {
    if (!loading) {
      const timer = requestAnimationFrame(() => setIsGridReady(true));
      return () => cancelAnimationFrame(timer);
    }
  }, [loading]);

  // Fetch the modal's code while idle, so the first card click opens it straight away instead
  // of waiting on the network before the animation can start.
  useEffect(() => {
    if (!isGridReady) return;
    const timer = setTimeout(() => void loadDetailsModal(), 1500);
    return () => clearTimeout(timer);
  }, [isGridReady]);

  // Read the URL through a ref so a URL change does not re-render every card.
  const urlRef = useRef({ location, search });
  useEffect(() => {
    urlRef.current = { location, search };
  }, [location, search]);

  // Driven by the modal's own mount/exit hand-off so the card never hides while the modal
  // chunk is still loading, and always reappears in lockstep with the closing animation.
  const [morphedTitle, setMorphedTitle] = useState<string | null>(null);
  const handleMorphStart = useCallback((title: string) => {
    setMorphedTitle(title);
  }, []);
  const handleMorphEnd = useCallback((title: string) => {
    setMorphedTitle((prev) => (prev === title ? null : prev));
  }, []);

  const handleOpenModal = useCallback(
    (title: string, info: UnifiedMetadata | null) => {
      // Snapshot the clicked card's exact visual rect (including hover lift/scale) before
      // any state update hides it.
      const cardSnapshot = captureCardMorphSnapshot(title, true);
      setCachedInfo({ title, info, cardSnapshot });

      // Update URL
      const params = new URLSearchParams(urlRef.current.search);
      params.set("anime", title);
      setLocation(`${urlRef.current.location}?${params.toString()}`);
    },
    [setLocation],
  );

  const handleCloseModal = () => {
    // Clear cache
    setCachedInfo(null);

    // Update URL
    const params = new URLSearchParams(search);
    params.delete("anime");
    setLocation(`${location}?${params.toString()}`);
  };

  const selectedYear = selections.year;
  const setSelectedYear = (year: string) =>
    setSelections((prev) => ({ ...prev, year }));

  const selectedSeason = selections.season;
  const setSelectedSeason = (season: string) =>
    setSelections((prev) => ({ ...prev, season }));

  const selectedSite = selections.site;
  const setSelectedSite = (site: string) =>
    setSelections((prev) => ({ ...prev, site }));

  const selectedStatus = selections.status || "all";
  const setSelectedStatus = (status: string) =>
    setSelections((prev) => ({ ...prev, status }));

  return (
    <div className="min-h-dvh bg-gray-100 text-gray-900 transition-colors dark:bg-gray-900 dark:text-gray-100">
      <Header
        config={config}
        selectedYear={selectedYear}
        setSelectedYear={setSelectedYear}
        selectedSeason={selectedSeason}
        setSelectedSeason={setSelectedSeason}
        selectedSite={selectedSite}
        setSelectedSite={setSelectedSite}
        selectedStatus={selectedStatus}
        setSelectedStatus={setSelectedStatus}
        searchQuery={searchQuery}
        setSearchQuery={setSearchQuery}
      />

      <main className="mx-auto max-w-7xl p-4 sm:p-6 lg:p-8">
        {loading ? (
          <CardGridSkeleton />
        ) : error && items.length === 0 ? (
          <div className="flex min-h-[50dvh] items-center justify-center p-4 text-red-500">
            <div className="text-center">
              <h2 className="mb-2 text-xl font-bold">
                Unable to load anime list
              </h2>
              <p className="opacity-80">{error}</p>
            </div>
          </div>
        ) : (
          <>
            {error && (
              <div className="mb-6 rounded-lg bg-red-50 p-4 text-sm text-red-800 dark:bg-red-900/20 dark:text-red-300">
                <span className="font-semibold">Note:</span> Failed to refresh
                data ({error}). Showing cached content.
              </div>
            )}
            <TabbedGrid
              items={filteredItems}
              siteMeta={config?.site_meta}
              selectedSite={selectedSite}
              onOpenModal={handleOpenModal}
              morphedTitle={morphedTitle}
            />
          </>
        )}
        <Footer onOpenAttribution={() => setIsAttributionOpen(true)} />
      </main>

      <DeferredDialog open={!!selectedAnime}>
        <DetailsModal
          isOpen={!!selectedAnime && isGridReady}
          onClose={handleCloseModal}
          anime={selectedAnime}
          fromCard={!!animeTitle && cachedInfo?.title === animeTitle}
          cardSnapshot={
            animeTitle && cachedInfo?.title === animeTitle
              ? cachedInfo.cardSnapshot
              : null
          }
          onMorphStart={handleMorphStart}
          onMorphEnd={handleMorphEnd}
          items={items}
          siteMeta={config?.site_meta}
          authEnabled={!!config?.auth_enabled}
          onUpdate={() => mutateStatuses()}
        />
      </DeferredDialog>
      <DeferredDialog open={isAttributionOpen}>
        <AttributionModal
          isOpen={isAttributionOpen}
          onClose={() => setIsAttributionOpen(false)}
          config={config}
        />
      </DeferredDialog>
    </div>
  );
}
