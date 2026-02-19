import { motion } from "motion/react";
import { useEffect, useMemo, useRef, useState } from "react";

import { useSmartMetadata } from "../hooks/useSmartMetadata";
import type { DisplayAnimeItem, SiteMeta, UnifiedMetadata } from "../types";
import { USER_STATUS_LABELS } from "../types";
import { cn } from "../utils/cn";
import { sortSites } from "../utils/siteUtils";
import { isValidUrl } from "../utils/urlUtils";
import SiteLink from "./SiteLink";
import Skeleton from "./Skeleton";

interface AnimeCardProps {
  item: DisplayAnimeItem;
  siteMeta?: SiteMeta;
  selectedSite?: string;
  onOpenModal: (title: string, info: UnifiedMetadata | null) => void;
}

export default function AnimeCard({
  item,
  siteMeta,
  selectedSite,
  onOpenModal,
}: AnimeCardProps) {
  /* REMOVED: internal state and loadMetadata */
  const [isEnabled, setIsEnabled] = useState(false);
  const { metadata, loading } = useSmartMetadata(item, null, isEnabled);
  const cardRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting && !isEnabled) {
          setIsEnabled(true);
          observer.disconnect();
        }
      },
      { rootMargin: "200px" },
    );

    if (cardRef.current) {
      observer.observe(cardRef.current);
    }

    return () => observer.disconnect();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [item.title]);

  const sitesToShow = useMemo(() => {
    let sites = item.sites || [];
    if (selectedSite && selectedSite !== "all") {
      sites = sites.filter((s) => s.site === selectedSite);
    }

    // Only show 'onair' sites on cards
    sites = sites.filter((s) => siteMeta?.[s.site]?.type === "onair");

    return sortSites(sites, siteMeta);
  }, [item.sites, selectedSite, siteMeta]);

  const coverUrl =
    metadata?.coverImage?.extraLarge || metadata?.coverImage?.large;

  return (
    <motion.div
      ref={cardRef}
      layoutId={`card-${item.title}`}
      initial={{
        y: 0,
        boxShadow:
          "0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)",
      }}
      whileHover={{
        y: -6,
        boxShadow:
          "0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1)",
        transition: { duration: 0.3, ease: "easeOut" },
      }}
      className="flex h-full flex-col overflow-hidden rounded-2xl bg-white ring-1 ring-black/5 cursor-pointer dark:bg-gray-800 dark:ring-white/5"
      onClick={() => onOpenModal(item.title, metadata)}
    >
      {/* Cover Image */}
      <motion.div
        layoutId={`image-${item.title}`}
        className={cn(
          "group relative aspect-[3/4] overflow-hidden bg-gray-200 dark:bg-gray-700",
          !coverUrl && loading && "animate-pulse",
        )}
      >
        {coverUrl ? (
          <img
            src={coverUrl}
            alt={item.title}
            className="h-full w-full object-cover transition-transform duration-500 group-hover:scale-105"
            loading="lazy"
          />
        ) : (
          !loading && (
            <div className="flex h-full w-full items-center justify-center text-sm text-gray-400 italic">
              No image
            </div>
          )
        )}

        {/* Status Badge */}
        {item.userStatus && item.userStatus > 0 && (
          <div className="absolute top-2 right-2 z-20">
            <span
              className={cn(
                "inline-flex items-center rounded-md px-2 py-1 text-[10px] font-bold text-white shadow-sm backdrop-blur-sm ring-1 ring-black/10",
                {
                  "bg-blue-500/90": item.userStatus === 1, // Watching
                  "bg-green-500/90": item.userStatus === 2, // Completed
                  "bg-yellow-500/90": item.userStatus === 3, // On Hold
                  "bg-red-500/90": item.userStatus === 4, // Dropped
                  "bg-purple-500/90": item.userStatus === 5, // Plan to Watch
                },
              )}
            >
              {USER_STATUS_LABELS[item.userStatus!]}
            </span>
          </div>
        )}

        {/* Hover Overlay */}
        <div className="absolute inset-0 bg-black/0 transition-colors group-hover:bg-black/20" />
      </motion.div>

      {/* Content */}
      <motion.div
        layoutId={`content-${item.title}`}
        className="flex flex-1 flex-col gap-2 p-3 md:gap-3 md:p-4"
      >
        <div className="flex flex-col gap-1">
          {loading && !metadata ? (
            <Skeleton className="h-5 w-3/4" />
          ) : (
            <motion.h3
              layoutId={`title-${item.title}`}
              className="line-clamp-2 text-sm leading-tight font-bold text-gray-900 md:text-base dark:text-gray-100"
            >
              {item.title}
            </motion.h3>
          )}
        </div>

        {/* Tags */}
        <div className="flex flex-wrap items-center gap-1.5">
          <span className="rounded-full bg-gray-100 px-2 py-0.5 text-[10px] font-bold text-gray-700 uppercase ring-1 ring-black/5 dark:bg-gray-700/50 dark:text-gray-300 dark:ring-white/10">
            {{
              tv: "TV",
              movie: "映画",
              ova: "OVA",
              ona: "ONA",
              special: "特別篇",
            }[item.type] || item.type}
          </span>
          {loading && !metadata ? (
            <>
              <Skeleton className="h-4 w-12 rounded-full" />
              <Skeleton className="h-4 w-8 rounded-full" />
              <Skeleton className="h-4 w-16 rounded-full" />
            </>
          ) : (
            <>
              {!!metadata?.averageScore && metadata.averageScore > 0 && (
                <span className="rounded-full bg-yellow-100 px-2 py-0.5 text-[10px] font-bold text-yellow-700 ring-1 ring-yellow-500/10 dark:bg-yellow-900/30 dark:text-yellow-300">
                  ⭐ {metadata.averageScore}%
                </span>
              )}
              {metadata?.episodes && (
                <span className="rounded-full bg-purple-100 px-2 py-0.5 text-[10px] font-bold text-purple-700 ring-1 ring-purple-500/10 dark:bg-purple-900/30 dark:text-purple-300">
                  {metadata.episodes}話
                </span>
              )}
              {metadata?.genres?.slice(0, 2).map((genre: string) => (
                <span
                  key={genre}
                  className="rounded-full bg-teal-100 px-2 py-0.5 text-[10px] font-bold text-teal-700 ring-1 ring-teal-500/10 dark:bg-teal-900/30 dark:text-teal-300"
                >
                  {genre}
                </span>
              ))}
              {item.begin && (
                <span className="rounded-full bg-blue-100 px-2 py-0.5 text-[10px] font-bold text-blue-700 ring-1 ring-blue-500/10 dark:bg-blue-900/30 dark:text-blue-300">
                  {new Date(item.begin).toISOString().split("T")[0]}
                </span>
              )}
            </>
          )}
        </div>

        {/* Links */}
        {sitesToShow.length > 0 && (
          <div className="mt-auto flex flex-wrap gap-1.5 border-t border-gray-100 pt-3 dark:border-gray-700/50">
            {sitesToShow.map((site, idx) => {
              const meta = siteMeta?.[site.site];
              const url =
                site.url || meta?.urlTemplate?.replace("{{id}}", site.id || "");
              if (!url || !isValidUrl(url)) return null;

              return (
                <SiteLink
                  key={`${site.site}-${idx}`}
                  url={url}
                  label={meta?.title || site.site}
                  className="rounded-md px-2 py-1 text-[11px] font-semibold text-blue-600 hover:bg-blue-50 hover:text-blue-700 dark:text-blue-400 dark:hover:bg-blue-900/40 dark:hover:text-blue-300"
                />
              );
            })}
          </div>
        )}
      </motion.div>
    </motion.div>
  );
}
