import { AnimatePresence, motion } from "motion/react";
import { useEffect, useMemo, useState } from "react";
import { useLocation, useSearch } from "wouter";
import { useResponsiveColumns } from "../hooks/useResponsiveColumns";
import { glassPillClassName } from "../styles/uiClasses";
import { cn } from "../utils/cn";

import type { DisplayAnimeItem, SiteMeta, UnifiedMetadata } from "../types";
import AnimeCard from "./AnimeCard";

const WEEKDAY_DATA = [
  { id: "0", label: "日", fullLabel: "日曜日" },
  { id: "1", label: "月", fullLabel: "月曜日" },
  { id: "2", label: "火", fullLabel: "火曜日" },
  { id: "3", label: "水", fullLabel: "水曜日" },
  { id: "4", label: "木", fullLabel: "木曜日" },
  { id: "5", label: "金", fullLabel: "金曜日" },
  { id: "6", label: "土", fullLabel: "土曜日" },
  { id: "7", label: "他", fullLabel: "その他" },
];

interface TabbedGridProps {
  items: DisplayAnimeItem[];
  siteMeta?: SiteMeta;
  selectedSite?: string;
  onOpenModal: (title: string, info: UnifiedMetadata | null) => void;
}

const containerVariants = {
  hidden: (direction: number) => ({
    x: direction > 0 ? "100%" : "-100%",
  }),
  show: {
    x: 0,
    transition: {
      x: { type: "tween" as const, ease: "easeInOut" as const, duration: 0.5 },
    },
  },
  pageExit: (direction: number) => ({
    x: direction > 0 ? "-100%" : "100%",
    transition: {
      x: { type: "tween" as const, ease: "easeInOut" as const, duration: 0.5 },
    },
  }),
};

const COLUMNS_BREAKPOINTS = {
  1280: 4, // xl
  768: 3, // md
  0: 2, // default
};

export default function TabbedGrid({
  items,
  siteMeta,
  selectedSite,
  onOpenModal,
}: TabbedGridProps) {
  const [location, setLocation] = useLocation();
  const search = useSearch();
  const currentDay = new Date().getDay().toString();

  const activeTab = useMemo(() => {
    const params = new URLSearchParams(search);
    const day = params.get("day");
    return day && WEEKDAY_DATA.some((d) => d.id === day) ? day : currentDay;
  }, [search, currentDay]);

  const [direction, setDirection] = useState(0);

  // Sync initial day to URL if missing, or ensure standard consistency
  useEffect(() => {
    const params = new URLSearchParams(search);
    if (!params.get("day")) {
      const newParams = new URLSearchParams(search);
      newParams.set("day", currentDay);
      // Replace to avoid pushing a history entry for the initial default
      window.history.replaceState(
        null,
        "",
        `${location}?${newParams.toString()}`,
      );
    }
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  const handleTabChange = (newTab: string) => {
    if (newTab === activeTab) return;
    const prevIndex = parseInt(activeTab);
    const nextIndex = parseInt(newTab);
    setDirection(nextIndex > prevIndex ? 1 : -1);

    const params = new URLSearchParams(search);
    params.set("day", newTab);
    setLocation(`${location}?${params.toString()}`);
  };

  const dayIndex = parseInt(activeTab);

  const columns = useResponsiveColumns(COLUMNS_BREAKPOINTS, 2);

  const groupedItems = useMemo(() => {
    const groups: { item: DisplayAnimeItem; time: number }[][] = Array.from(
      { length: 8 },
      () => [],
    );
    items.forEach((item) => {
      let itemDayIndex = 7;
      let time = 0;
      if (item.begin) {
        const date = new Date(item.begin);
        const t = date.getTime();
        if (!isNaN(t)) {
          itemDayIndex = date.getDay();
          time = t;
        }
      }
      groups[itemDayIndex].push({ item, time });
    });

    return groups.map((group) =>
      group.sort((a, b) => a.time - b.time).map((g) => g.item),
    );
  }, [items]);

  const dayItems = groupedItems[dayIndex];

  const columnItems = useMemo(() => {
    const result: (typeof dayItems)[] = Array.from(
      { length: columns },
      () => [],
    );
    dayItems.forEach((item, index) => {
      result[index % columns].push(item);
    });
    return result;
  }, [dayItems, columns]);

  return (
    <div className="flex flex-col gap-6">
      <div className=" flex justify-center px-6 pb-2">
        <div
          className={`${glassPillClassName} no-scrollbar flex max-w-full gap-1 overflow-x-auto p-1`}
        >
          {WEEKDAY_DATA.map((tab) => {
            const isActive = activeTab === tab.id;
            return (
              <button
                key={tab.id}
                onClick={() => handleTabChange(tab.id)}
                className={cn(
                  "relative z-10 flex flex-shrink-0 cursor-pointer items-center gap-2 rounded-full px-4 py-2 text-sm font-semibold outline-none transition-colors",
                  isActive
                    ? "text-white"
                    : "text-slate-500 hover:text-slate-900 dark:text-slate-400 dark:hover:text-slate-200",
                )}
              >
                {isActive && (
                  <motion.div
                    layoutId="air-tab-pill"
                    className="absolute inset-0 -z-10 rounded-full bg-blue-500 shadow-lg shadow-blue-500/25 dark:bg-blue-600 dark:shadow-blue-600/25"
                    transition={{ type: "spring", stiffness: 300, damping: 30 }}
                  />
                )}
                <span className="relative z-10">{tab.label}</span>
                {isActive && (
                  <motion.span
                    initial={{ scale: 0 }}
                    animate={{ scale: 1 }}
                    className="text-xs opacity-60"
                  >
                    · {tab.fullLabel}
                  </motion.span>
                )}
              </button>
            );
          })}
        </div>
      </div>

      <div
        className="relative -mx-2 overflow-hidden"
        style={{ display: "grid" }}
      >
        <AnimatePresence mode="sync" initial={false} custom={direction}>
          <motion.div
            key={activeTab}
            custom={direction}
            variants={containerVariants}
            initial="hidden"
            animate="show"
            exit="pageExit"
            className="px-2 pb-12"
            style={{ gridArea: "1 / 1" }}
          >
            {dayItems.length > 0 ? (
              <div className="flex gap-3 sm:gap-4 lg:gap-6">
                {columnItems.map((itemsInColumn, colIndex) => (
                  <div
                    key={colIndex}
                    className="flex flex-1 flex-col gap-3 sm:gap-4 lg:gap-6"
                  >
                    {itemsInColumn.map((item) => (
                      <div key={item.title} className="w-full">
                        <AnimeCard
                          item={item}
                          siteMeta={siteMeta}
                          selectedSite={selectedSite}
                          onOpenModal={onOpenModal}
                        />
                      </div>
                    ))}
                  </div>
                ))}
              </div>
            ) : (
              <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                className="py-20 text-center text-gray-500 dark:text-gray-400"
              >
                この日の放送はありません
              </motion.div>
            )}
          </motion.div>
        </AnimatePresence>
      </div>
    </div>
  );
}
