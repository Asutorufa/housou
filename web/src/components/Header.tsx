import { clsx, type ClassValue } from "clsx";
import { Search, User as UserIcon, X } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useMemo, useState } from "react";
import { twMerge } from "tailwind-merge";
import { useAuth } from "../contexts/AuthContext";
import type { Config } from "../types";
import { USER_STATUS_LABELS } from "../types";
import AuthModal from "./AuthModal";
import CustomSelect from "./CustomSelect";
import ProfileModal from "./ProfileModal";
import UserMenu from "./UserMenu";

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

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
      <div className="mx-auto flex max-w-7xl items-center justify-between gap-3">
        {/* Filter Group (Left) */}
        <AnimatePresence mode="popLayout" initial={false}>
          {!isSearchFocused && (
            <motion.div
              key="filters"
              initial={{ opacity: 0, x: -20 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: -20 }}
              transition={{ duration: 0.2 }}
              className="pointer-events-auto flex min-w-0 max-w-full items-center rounded-full border border-gray-200/50 bg-white/80 p-1 shadow-md backdrop-blur-md dark:border-gray-700/50 dark:bg-gray-800/80"
            >
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
                  triggerClassName="min-w-[80px] sm:min-w-[90px]"
                  contentClassName="z-[60]"
                />

                <div className="h-4 w-px bg-gray-300 dark:bg-gray-600" />

                {/* Season Select */}
                <CustomSelect
                  value={selectedSeason}
                  onValueChange={setSelectedSeason}
                  options={[
                    { value: "all", label: "全て" },
                    { value: "Winter", label: "冬" },
                    { value: "Spring", label: "春" },
                    { value: "Summer", label: "夏" },
                    { value: "Autumn", label: "秋" },
                  ]}
                  placeholder="シーズン"
                  isOpen={activeDropdown === "season"}
                  onOpenChange={(open) => handleDropdownChange("season", open)}
                  triggerClassName="min-w-[80px] sm:min-w-[90px]"
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
                  triggerClassName="min-w-[80px] sm:min-w-[90px]"
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
                      triggerClassName="min-w-[80px] sm:min-w-[90px]"
                      contentClassName="z-[60]"
                    />
                  </>
                )}
              </div>
            </motion.div>
          )}
        </AnimatePresence>

        {/* Right Group: Search + User Menu */}
        <div
          className={cn(
            "flex items-center gap-2 transition-all duration-300",
            isSearchFocused ? "flex-1 ml-0" : "ml-auto",
          )}
        >
          {/* Search Input */}
          <motion.div
            layout
            transition={{
              type: "spring",
              stiffness: 300,
              damping: 35,
              mass: 0.8,
            }}
            className={cn(
              "pointer-events-auto group relative shrink-0 overflow-hidden rounded-full border border-gray-200/50 bg-white/80 shadow-md backdrop-blur-md hover:border-blue-500/50 dark:border-gray-700/50 dark:bg-gray-800/80",
              isSearchFocused
                ? "w-full border-blue-500 ring-2 ring-blue-500/20"
                : "w-10 md:w-64",
            )}
          >
            <div
              className={cn(
                "pointer-events-none absolute inset-0 flex items-center justify-center text-gray-400 md:inset-y-0 md:right-auto md:left-3 md:w-auto md:justify-start",
                isSearchFocused &&
                  "inset-y-0 right-auto left-3 w-auto justify-start text-blue-500",
              )}
            >
              <Search size={16} />
            </div>
            <input
              type="text"
              placeholder="検索..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              onFocus={() => setIsSearchFocused(true)}
              onBlur={() => {
                setIsSearchFocused(false);
                setActiveDropdown(null); // Close dropdowns on blur just in case
              }}
              className="w-full border-none bg-transparent py-2.5 pr-8 pl-9 text-sm text-gray-900 placeholder-transparent outline-none focus:placeholder-gray-500 md:placeholder-gray-500 dark:text-gray-100 dark:focus:placeholder-gray-400 md:dark:placeholder-gray-400"
            />
            <AnimatePresence>
              {searchQuery && (
                <motion.button
                  initial={{ opacity: 0, scale: 0.8 }}
                  animate={{ opacity: 1, scale: 1 }}
                  exit={{ opacity: 0, scale: 0.8 }}
                  onClick={() => setSearchQuery("")}
                  className={cn(
                    "absolute inset-y-0 right-2 hidden items-center text-gray-400 hover:text-gray-600 md:flex dark:hover:text-gray-200",
                    isSearchFocused && "flex",
                  )}
                >
                  <X size={14} />
                </motion.button>
              )}
            </AnimatePresence>
          </motion.div>

          {/* User Menu */}
          {config?.auth_enabled && (
            <div className="pointer-events-auto shrink-0 relative">
              {loggedIn ? (
                <UserMenu
                  isOpen={activeDropdown === "user"}
                  onOpenChange={(open) => handleDropdownChange("user", open)}
                  onOpenProfile={() => setIsProfileModalOpen(true)}
                />
              ) : (
                <motion.button
                  onClick={() => setIsAuthModalOpen(true)}
                  className="flex h-10 w-10 items-center justify-center rounded-full border border-gray-200 bg-white shadow-sm transition-colors hover:bg-gray-50 dark:border-gray-700 dark:bg-gray-800 dark:hover:bg-gray-700"
                >
                  <UserIcon
                    size={16}
                    className="text-gray-700 dark:text-gray-200"
                  />
                </motion.button>
              )}
            </div>
          )}
        </div>
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
