import { clsx, type ClassValue } from "clsx";
import { motion } from "motion/react";
import { useEffect, useMemo, useRef, useState } from "react";
import { twMerge } from "tailwind-merge";
import type { DisplayAnimeItem, SiteMeta, UnifiedMetadata } from "../types";
import { USER_STATUS_LABELS } from "../types";
import { isDev } from "../utils/envUtils";
import { sortSites } from "../utils/siteUtils";
import { isValidUrl } from "../utils/urlUtils";

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

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
  const [metadata, setMetadata] = useState<UnifiedMetadata | null>(null);
  const [loading, setLoading] = useState(false);
  const cardRef = useRef<HTMLDivElement>(null);
  const loadedRef = useRef(false);

  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting && !loadedRef.current) {
          loadedRef.current = true;
          fetchMetadata();
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

  async function fetchMetadata() {
    setLoading(true);
    try {
      const tmdbSite = item.sites?.find((s) => s.site === "tmdb");
      const malSite = item.sites?.find((s) => s.site === "mal");
      const anilistSite = item.sites?.find(
        (s) => s.site === "aniList" || s.site === "anilist",
      );

      const params = new URLSearchParams();
      params.append("title", item.title);

      if (tmdbSite?.id) {
        params.append("tmdb_id", tmdbSite.id);
      }

      if (malSite?.id) {
        params.append("mal_id", malSite.id);
      }

      if (anilistSite?.id) {
        params.append("anilist_id", anilistSite.id);
      }

      if (item.begin) {
        params.append("begin", item.begin);
      }

      const url = `/api/metadata?${params.toString()}`;
      const response = await fetch(url);
      if (!response.ok) throw new Error("Metadata fetch failed");
      const data = await response.json();
      setMetadata(data || null);
    } catch (err) {
      if (isDev()) {
        console.error("Metadata error:", err);
      }
    } finally {
      setLoading(false);
    }
  }

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
      className="flex h-full flex-col overflow-hidden rounded-2xl bg-white ring-1 ring-black/5 dark:bg-gray-800 dark:ring-white/5"
    >
      {/* Cover Image */}
      <motion.div
        layoutId={`image-${item.title}`}
        className={cn(
          "group relative aspect-[3/4] cursor-pointer overflow-hidden bg-gray-200 dark:bg-gray-700",
          !coverUrl && loading && "animate-pulse",
        )}
        onClick={() => onOpenModal(item.title, metadata)}
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
        <motion.h3
          layoutId={`title-${item.title}`}
          className="line-clamp-2 text-sm leading-tight font-bold text-gray-900 md:text-base dark:text-gray-100"
        >
          {item.title}
        </motion.h3>

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
                <a
                  key={`${site.site}-${idx}`}
                  href={url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="rounded-md px-2 py-1 text-[11px] font-semibold text-blue-600 transition-all hover:bg-blue-50 hover:text-blue-700 active:scale-95 dark:text-blue-400 dark:hover:bg-blue-900/40 dark:hover:text-blue-300"
                >
                  {meta?.title || site.site}
                </a>
              );
            })}
          </div>
        )}
      </motion.div>
    </motion.div>
  );
}
