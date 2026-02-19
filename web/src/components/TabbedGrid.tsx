import { AnimatePresence, motion } from "motion/react";
import { useMemo, useState } from "react";
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

export default function TabbedGrid({
  items,
  siteMeta,
  selectedSite,
  onOpenModal,
}: TabbedGridProps) {
  const currentDay = new Date().getDay().toString();
  const [activeTab, setActiveTab] = useState(currentDay);
  const [direction, setDirection] = useState(0);

  const handleTabChange = (newTab: string) => {
    const prevIndex = parseInt(activeTab);
    const nextIndex = parseInt(newTab);
    setDirection(nextIndex > prevIndex ? 1 : -1);
    setActiveTab(newTab);
  };

  const groupedItems = useMemo(() => {
    const groups: { item: DisplayAnimeItem; time: number }[][] = Array.from(
      { length: 8 },
      () => [],
    );
    items.forEach((item) => {
      let dayIndex = 7;
      let time = 0;
      if (item.begin) {
        const date = new Date(item.begin);
        const t = date.getTime();
        if (!isNaN(t)) {
          dayIndex = date.getDay();
          time = t;
        }
      }
      groups[dayIndex].push({ item, time });
    });

    return groups.map((group) =>
      group.sort((a, b) => a.time - b.time).map((g) => g.item),
    );
  }, [items]);

  const dayIndex = parseInt(activeTab);
  const dayItems = groupedItems[dayIndex];

  return (
    <div className="flex flex-col gap-6">
      <div className="sticky top-0 z-30 flex justify-center px-6 pb-2 pt-6">
        <div className="no-scrollbar flex max-w-full gap-1 overflow-x-auto rounded-full border border-white/40 bg-white/80 p-1.5 shadow-sm backdrop-blur-xl dark:border-white/10 dark:bg-gray-900/80">
          {WEEKDAY_DATA.map((tab) => {
            const isActive = activeTab === tab.id;
            return (
              <button
                key={tab.id}
                onClick={() => handleTabChange(tab.id)}
                className={cn(
                  "relative z-10 flex flex-shrink-0 items-center gap-2 rounded-full px-4 py-2 text-sm font-semibold outline-none transition-colors",
                  isActive
                    ? "text-white dark:text-slate-900"
                    : "text-slate-500 hover:text-slate-900 dark:text-slate-400 dark:hover:text-slate-200",
                )}
              >
                {isActive && (
                  <motion.div
                    layoutId="air-tab-pill"
                    className="absolute inset-0 -z-10 rounded-full bg-slate-900 shadow-lg dark:bg-slate-100"
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
              <div className="columns-2 gap-1 sm:gap-2 md:gap-4 lg:columns-3 xl:columns-4">
                <AnimatePresence mode="popLayout" initial={false}>
                  {dayItems.map((item) => (
                    <motion.div
                      key={item.title}
                      layout="position"
                      className="mb-1 break-inside-avoid p-2 sm:mb-2 md:mb-4"
                    >
                      <AnimeCard
                        item={item}
                        siteMeta={siteMeta}
                        selectedSite={selectedSite}
                        onOpenModal={onOpenModal}
                      />
                    </motion.div>
                  ))}
                </AnimatePresence>
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
