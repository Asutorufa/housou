import { lazy, useCallback, useEffect, useRef, useState } from "react";
import { useLocation, useSearch } from "wouter";
import CardGridSkeleton from "./components/CardGridSkeleton";
import Footer from "./components/Footer";
import DeferredDialog from "./components/DeferredDialog";
import Header from "./components/Header";
import TabbedGrid from "./components/TabbedGrid";
import { useAnimeData } from "./hooks/useAnimeData";
import type { UnifiedMetadata } from "./types";

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

  // We delay opening the modal on initial deep-link load until the grid is mounted.
  // This ensures Framer Motion's `layoutId` animation originates from the card rather than conflicting.
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

  // The modal's cover flies out of (and back into) the card, so the card's own cover is hidden
  // meanwhile. It comes back just before the closing panel fades out on top of it.
  const [hiddenCoverTitle, setHiddenCoverTitle] = useState<string | null>(null);
  if (animeTitle && hiddenCoverTitle !== animeTitle) {
    setHiddenCoverTitle(animeTitle);
  }
  useEffect(() => {
    if (animeTitle || !hiddenCoverTitle) return;
    const timer = setTimeout(() => setHiddenCoverTitle(null), 240);
    return () => clearTimeout(timer);
  }, [animeTitle, hiddenCoverTitle]);

  const handleOpenModal = useCallback(
    (title: string, info: UnifiedMetadata | null) => {
      // Cache info for immediate presentation in the modal
      setCachedInfo({ title, info });

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
              hiddenCoverTitle={hiddenCoverTitle}
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
