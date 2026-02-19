import { Search, User as UserIcon, X } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useMemo, useState } from "react";
import { useAuth } from "../contexts/AuthContext";
import type { Config } from "../types";
import { getSeasonOptions } from "../utils/season";
import { USER_STATUS_LABELS } from "../types";
import AuthModal from "./AuthModal";
import CustomSelect from "./CustomSelect";
import ProfileModal from "./ProfileModal";
import UserMenu from "./UserMenu";

const filterSelectTriggerClassName =
  "min-w-[80px] sm:min-w-[90px] max-w-[120px]";

interface HeaderProps {
  config: Config | null;
  selectedYear: string;
  setSelectedYear: (year: string) => void;
  selectedSeason: string;
  setSelectedSeason: (season: string) => void;
  selectedSite: string;
  setSelectedSite: (site: string) => void;
  selectedStatus: string;
  setSelectedStatus: (status: string) => void;
  searchQuery: string;
  setSearchQuery: (query: string) => void;
}

export default function Header({
  config,
  selectedYear,
  setSelectedYear,
  selectedSeason,
  setSelectedSeason,
  selectedSite,
  setSelectedSite,
  selectedStatus,
  setSelectedStatus,
  searchQuery,
  setSearchQuery,
}: HeaderProps) {
  const { loggedIn } = useAuth();
  const [isSearchFocused, setIsSearchFocused] = useState(false);
  const [activeDropdown, setActiveDropdown] = useState<string | null>(null);
  const [isAuthModalOpen, setIsAuthModalOpen] = useState(false);
  const [isProfileModalOpen, setIsProfileModalOpen] = useState(false);

  // Helper to handle dropdown state
  const handleDropdownChange = (key: string, isOpen: boolean) => {
    setActiveDropdown(isOpen ? key : null);
  };

  const currentYear = new Date().getFullYear();
  const currentMonth = new Date().getMonth() + 1;

  const seasonOptions = useMemo(
    () => getSeasonOptions(selectedYear, currentYear, currentMonth),
    [selectedYear, currentYear, currentMonth],
  );

  const siteOptions = useMemo(
    () => [
      { value: "all", label: "全て" },
      ...Object.entries(config?.site_meta || {}).map(([key, meta]) => ({
        value: key,
        label: meta?.title || key,
      })),
    ],
    [config?.site_meta],
  );

  return (
    <header className="group/header pointer-events-none sticky top-2 z-50 w-full px-2 md:px-4">
      <div className="relative mx-auto flex h-14 max-w-7xl items-center justify-between gap-3">
        <AnimatePresence mode="wait">
          {isSearchFocused ? (
            <motion.div
              key="search-bar"
              initial={{ width: 40, opacity: 0 }}
              animate={{ width: "100%", opacity: 1 }}
              exit={{ width: 40, opacity: 0 }}
              transition={{ type: "spring", stiffness: 350, damping: 30 }}
              className="pointer-events-auto absolute right-2 top-1/2 z-30 flex h-10 -translate-y-1/2 items-center md:right-4"
            >
              <div className="search-container relative h-full w-full overflow-hidden rounded-full border border-blue-500 bg-white/80 shadow-md backdrop-blur-md ring-2 ring-blue-500/20 dark:border-gray-700/50 dark:bg-gray-800/80">
                <Search
                  size={16}
                  className="absolute left-3 top-1/2 -translate-y-1/2 text-blue-500"
                />
                <input
                  type="text"
                  placeholder="検索..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  autoFocus
                  onBlur={(e) => {
                    // Use setTimeout to allow click events on the close button (or other elements) to fire first
                    // Check if the new focus is still within the search container if needed,
                    // but here we just want to close it if the user clicks outside or tabs away.
                    // However, immediate close prevents the close button's onClick from firing.
                    // relatedTarget check is better if the close button can receive focus.
                    if (
                      !e.relatedTarget ||
                      (e.relatedTarget as HTMLElement).closest(
                        ".search-container",
                      ) === null
                    ) {
                      setTimeout(() => setIsSearchFocused(false), 150);
                    }
                  }}
                  className="h-full w-full border-none bg-transparent py-2 pl-9 pr-10 text-sm text-gray-900 placeholder-gray-500 outline-none dark:text-gray-100 dark:placeholder-gray-400"
                />
                <button
                  onClick={() => {
                    setSearchQuery("");
                    setIsSearchFocused(false);
                  }}
                  aria-label="検索をクリアして閉じる"
                  className="absolute right-3 top-1/2 flex -translate-y-1/2 items-center text-gray-400 hover:text-gray-600 dark:hover:text-gray-200"
                >
                  <X size={16} />
                </button>
              </div>
            </motion.div>
          ) : (
            <motion.div
              key="default-toolbar"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.2 }}
              className="flex w-full items-center justify-between gap-3"
            >
              {/* Filter Group (Left) */}
              <div className="pointer-events-auto flex min-w-0 max-w-full items-center rounded-full border border-gray-200/50 bg-white/80 p-1 shadow-md backdrop-blur-md dark:border-gray-700/50 dark:bg-gray-800/80">
                <div className="scroll-mask-x no-scrollbar flex w-full items-center overflow-x-auto px-2">
                  {/* Year Select */}
                  <CustomSelect
                    value={selectedYear}
                    onValueChange={setSelectedYear}
                    options={
                      config?.years.map((y) => ({
                        value: y.toString(),
                        label: y > currentYear ? `${y} (予定)` : y.toString(),
                      })) || []
                    }
                    placeholder="年"
                    isOpen={activeDropdown === "year"}
                    onOpenChange={(open) => handleDropdownChange("year", open)}
                    triggerClassName={filterSelectTriggerClassName}
                    contentClassName="z-[60]"
                  />

                  <div className="h-4 w-px bg-gray-300 dark:bg-gray-600" />

                  {/* Season Select */}
                  <CustomSelect
                    value={selectedSeason}
                    onValueChange={setSelectedSeason}
                    options={seasonOptions}
                    placeholder="シーズン"
                    isOpen={activeDropdown === "season"}
                    onOpenChange={(open) =>
                      handleDropdownChange("season", open)
                    }
                    triggerClassName={filterSelectTriggerClassName}
                    contentClassName="z-[60]"
                  />

                  <div className="h-4 w-px bg-gray-300 dark:bg-gray-600" />

                  {/* Site Select */}
                  <CustomSelect
                    value={selectedSite}
                    onValueChange={setSelectedSite}
                    options={siteOptions}
                    placeholder="サイト"
                    isOpen={activeDropdown === "site"}
                    onOpenChange={(open) => handleDropdownChange("site", open)}
                    triggerClassName={filterSelectTriggerClassName}
                    contentClassName="z-[60]"
                  />

                  {/* Status Select (Only if logged in) */}
                  {loggedIn && (
                    <>
                      <div className="h-4 w-px bg-gray-300 dark:bg-gray-600" />
                      <CustomSelect
                        value={selectedStatus}
                        onValueChange={setSelectedStatus}
                        options={[
                          { value: "all", label: "全て" },
                          ...Object.entries(USER_STATUS_LABELS).map(
                            ([value, label]) => ({
                              value,
                              label: label as string,
                            }),
                          ),
                        ]}
                        placeholder="状態"
                        isOpen={activeDropdown === "status"}
                        onOpenChange={(open) =>
                          handleDropdownChange("status", open)
                        }
                        triggerClassName={filterSelectTriggerClassName}
                        contentClassName="z-[60]"
                      />
                    </>
                  )}
                </div>
              </div>

              {/* Right Group: Search Trigger + User Menu */}
              <div className="flex items-center gap-2">
                <button
                  onClick={() => setIsSearchFocused(true)}
                  aria-label="検索を開く"
                  className="pointer-events-auto flex h-10 w-10 items-center justify-center rounded-full border border-gray-200/50 bg-white/80 shadow-md backdrop-blur-md transition-colors hover:border-blue-500/50 hover:bg-white dark:border-gray-700/50 dark:bg-gray-800/80 dark:hover:bg-gray-800"
                >
                  <Search
                    size={16}
                    className="text-gray-500 dark:text-gray-400"
                  />
                </button>

                {config?.auth_enabled && (
                  <div className="pointer-events-auto relative shrink-0">
                    {loggedIn ? (
                      <UserMenu
                        isOpen={activeDropdown === "user"}
                        onOpenChange={(open) =>
                          handleDropdownChange("user", open)
                        }
                        onOpenProfile={() => setIsProfileModalOpen(true)}
                      />
                    ) : (
                      <button
                        onClick={() => setIsAuthModalOpen(true)}
                        className="flex h-10 w-10 items-center justify-center rounded-full border border-gray-200 bg-white shadow-sm transition-colors hover:bg-gray-50 dark:border-gray-700 dark:bg-gray-800 dark:hover:bg-gray-700"
                      >
                        <UserIcon
                          size={16}
                          className="text-gray-700 dark:text-gray-200"
                        />
                      </button>
                    )}
                  </div>
                )}
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </div>

      <AuthModal
        isOpen={isAuthModalOpen}
        onClose={() => setIsAuthModalOpen(false)}
        githubEnabled={config?.github_enabled}
      />
      <ProfileModal
        isOpen={isProfileModalOpen}
        onClose={() => setIsProfileModalOpen(false)}
        githubEnabled={config?.github_enabled}
      />
    </header>
  );
}
