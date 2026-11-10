import { motion } from "motion/react";
import { memo, useEffect, useMemo, useRef, useState } from "react";

import { useSmartMetadata } from "../hooks/useSmartMetadata";
import { MORPH_TRANSITION } from "../styles/motion";
import type { DisplayAnimeItem, SiteMeta, UnifiedMetadata } from "../types";
import { USER_STATUS_LABELS } from "../types";
import { cn } from "../utils/cn";
import {
  cardCoverBox,
  getCoverAspect,
  rememberCoverAspect,
} from "../utils/coverCrop";
import { getCardCoverUrl } from "../utils/coverImage";
import { observeLazy, unobserveLazy } from "../utils/lazyObserver";
import { sortSites } from "../utils/siteUtils";
import { isValidUrl } from "../utils/urlUtils";
import SiteLink from "./SiteLink";
import Skeleton from "./Skeleton";

interface AnimeCardProps {
  item: DisplayAnimeItem;
  siteMeta?: SiteMeta;
  selectedSite?: string;
  onOpenModal: (title: string, info: UnifiedMetadata | null) => void;
  /** The details modal is showing this cover; hide ours so it isn't seen twice. */
  coverHidden?: boolean;
}

function AnimeCard({
  item,
  siteMeta,
  selectedSite,
  onOpenModal,
  coverHidden,
}: AnimeCardProps) {
  const [isEnabled, setIsEnabled] = useState(false);
  const { metadata, loading, error, retry } = useSmartMetadata(
    item,
    null,
    isEnabled,
  );
  const cardRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (isEnabled || !cardRef.current) return;
    const element = cardRef.current;

    observeLazy(element, () => {
      setIsEnabled(true);
    });

    return () => {
      unobserveLazy(element);
    };
  }, [item.title, isEnabled]);

  const sitesToShow = useMemo(() => {
    let sites = item.sites || [];
    if (selectedSite && selectedSite !== "all") {
      sites = sites.filter((s) => s.site === selectedSite);
    }

    // Only show 'onair' sites on cards
    sites = sites.filter((s) => siteMeta?.[s.site]?.type === "onair");

    return sortSites(sites, siteMeta);
  }, [item.sites, selectedSite, siteMeta]);

  const coverUrl = getCardCoverUrl(metadata?.coverImage);
  const [loadedCover, setLoadedCover] = useState<string | null>(null);
  // Known once the image has loaded (here or on an earlier visit to this card).
  const coverAspect =
    loadedCover === coverUrl || getCoverAspect(coverUrl)
      ? getCoverAspect(coverUrl)
      : undefined;
  const handleCoverLoaded = (img: HTMLImageElement | null) => {
    if (!img?.complete || !coverUrl) return;
    rememberCoverAspect(coverUrl, img);
    setLoadedCover(coverUrl);
  };
  // Cards outside the lazy-load margin have not asked for metadata yet.
  const isPending = loading || (!isEnabled && !metadata);
  const beginDate = useMemo(() => {
    if (!item.begin) return null;
    const parsed = new Date(item.begin);
    if (Number.isNaN(parsed.getTime())) return null;
    return parsed.toISOString().split("T")[0];
  }, [item.begin]);

  return (
    // The hover lift is a CSS transform transition, so it lives on this plain wrapper: on the
    // Motion element below it would smooth (and so lag behind) every frame of the layout morph.
    <div
      ref={cardRef}
      className="anime-card relative isolate h-full cursor-pointer rounded-2xl transition-transform duration-300 ease-out hover:-translate-y-1.5"
      onClick={() => onOpenModal(item.title, metadata)}
    >
      <motion.div
        layoutId={`card-${item.title}`}
        // Motion only corrects border radius during layout animations via style.
        style={{ borderRadius: 16 }}
        transition={{ layout: MORPH_TRANSITION }}
        className="h-full bg-white shadow-md ring-1 ring-black/5 dark:bg-gray-800 dark:ring-white/5"
      >
        <div className="flex h-full flex-col overflow-hidden rounded-[inherit]">
          {/* Cover Image */}
          <div
            className={cn(
              "group relative aspect-[3/4] overflow-hidden bg-gray-200 dark:bg-gray-700",
              !coverUrl && isPending && "animate-pulse",
            )}
          >
            {coverUrl && !coverAspect ? (
              // Loads the poster to learn its shape before the real cover below is mounted.
              <img
                ref={handleCoverLoaded}
                onLoad={(event) => handleCoverLoaded(event.currentTarget)}
                src={coverUrl}
                alt=""
                aria-hidden="true"
                decoding="async"
                className="invisible absolute h-px w-px"
              />
            ) : coverUrl ? (
              // Shared with the details modal. The box has the poster's own shape (cropped by the
              // 3:4 window above), so the morph into the uncropped modal cover never stretches it.
              <motion.div
                layoutId={`image-${item.title}`}
                transition={{ layout: MORPH_TRANSITION }}
                className="absolute"
                style={cardCoverBox(coverAspect)}
              >
                <img
                  src={coverUrl}
                  alt={item.title}
                  className={cn(
                    "h-full w-full object-cover transition-transform duration-500 group-hover:scale-105",
                    coverHidden && "invisible",
                  )}
                  decoding="async"
                />
              </motion.div>
            ) : (
              !isPending && (
                <div className="flex h-full w-full items-center justify-center text-sm text-gray-400 italic">
                  {error ? (
                    <button
                      type="button"
                      onClick={(event) => {
                        event.stopPropagation();
                        retry();
                      }}
                      className="underline"
                    >
                      再試行
                    </button>
                  ) : (
                    "No image"
                  )}
                </div>
              )
            )}

            {/* Status Badge */}
            {item.userStatus !== undefined && item.userStatus > 0 && (
              <div className="absolute top-2 right-2 z-20">
                <span
                  className={cn(
                    "inline-flex items-center rounded-md px-2 py-1 text-[10px] font-bold text-white shadow-sm ring-1 ring-black/10",
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
            <div className="pointer-events-none absolute inset-0 bg-black/20 opacity-0 transition-opacity group-hover:opacity-100" />
          </div>

          {/* Content */}
          <div className="flex flex-1 flex-col gap-2 p-3 md:gap-3 md:p-4">
            <div className="flex flex-col gap-1">
              <h3 className="line-clamp-2 text-sm leading-tight font-bold text-gray-900 md:text-base dark:text-gray-100">
                <button
                  type="button"
                  aria-haspopup="dialog"
                  className="cursor-pointer text-left focus-visible:outline-2 focus-visible:outline-blue-500"
                  onClick={(event) => {
                    event.stopPropagation();
                    onOpenModal(item.title, metadata);
                  }}
                >
                  {item.title}
                </button>
              </h3>
            </div>

            {/* Tags */}
            <div className="flex min-h-11 flex-wrap content-start items-center gap-1.5">
              <span className="rounded-full bg-gray-100 px-2 py-0.5 text-[10px] font-bold text-gray-700 uppercase ring-1 ring-black/5 dark:bg-gray-700/50 dark:text-gray-300 dark:ring-white/10">
                {{
                  tv: "TV",
                  movie: "映画",
                  ova: "OVA",
                  ona: "ONA",
                  special: "特別篇",
                }[item.type] || item.type}
              </span>
              {isPending && !metadata ? (
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
                  {beginDate && (
                    <span className="rounded-full bg-blue-100 px-2 py-0.5 text-[10px] font-bold text-blue-700 ring-1 ring-blue-500/10 dark:bg-blue-900/30 dark:text-blue-300">
                      {beginDate}
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
                    site.url ||
                    meta?.urlTemplate?.replace("{{id}}", site.id || "");
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
          </div>
        </div>
      </motion.div>
    </div>
  );
}

export default memo(AnimeCard);
