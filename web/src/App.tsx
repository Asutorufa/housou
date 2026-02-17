import { useState } from "react";
import AttributionModal from "./components/AttributionModal";
import DetailsModal from "./components/DetailsModal";
import Footer from "./components/Footer";
import Header from "./components/Header";
import TabbedGrid from "./components/TabbedGrid";
import { useAnimeData } from "./hooks/useAnimeData";
import type { UnifiedMetadata } from "./types";

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

  const [selectedAnime, setSelectedAnime] = useState<{
    title: string;
    info: UnifiedMetadata | null;
  } | null>(null);
  const [isAttributionOpen, setIsAttributionOpen] = useState(false);

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

  if (error) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-gray-100 p-4 text-red-500 dark:bg-gray-900">
        <div className="text-center">
          <h1 className="mb-2 text-2xl font-bold">Error</h1>
          <p>{error}</p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-100 text-gray-900 transition-colors dark:bg-gray-900 dark:text-gray-100">
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
          <div className="flex justify-center py-12">
            <div className="h-12 w-12 animate-spin rounded-full border-b-2 border-blue-500"></div>
          </div>
        ) : (
          <TabbedGrid
            items={filteredItems}
            siteMeta={config?.site_meta}
            selectedSite={selectedSite}
            onOpenModal={(title: string, info: UnifiedMetadata | null) =>
              setSelectedAnime({ title, info })
            }
          />
        )}
        <Footer onOpenAttribution={() => setIsAttributionOpen(true)} />
      </main>

      <DetailsModal
        isOpen={!!selectedAnime}
        onClose={() => setSelectedAnime(null)}
        anime={selectedAnime}
        items={items}
        siteMeta={config?.site_meta}
        onUpdate={() => mutateStatuses()}
      />

      <AttributionModal
        isOpen={isAttributionOpen}
        onClose={() => setIsAttributionOpen(false)}
        config={config}
      />
    </div>
  );
}
