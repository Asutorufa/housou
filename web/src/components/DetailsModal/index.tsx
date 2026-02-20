import * as Dialog from "@radix-ui/react-dialog";
import DOMPurify from "dompurify";
import { Bookmark, X } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useAuth } from "../../contexts/AuthContext";
import { useAnimeStatus } from "../../hooks/useAnimeStatus";
import { useSmartMetadata } from "../../hooks/useSmartMetadata";
import type { DisplayAnimeItem, SiteMeta, UnifiedMetadata } from "../../types";
import { USER_STATUS_LABELS } from "../../types";
import { sortSites } from "../../utils/siteUtils";
import CustomSelect from "../CustomSelect";
import Skeleton from "../Skeleton";
import AnimeCover from "./AnimeCover";
import CastSection from "./CastSection";
import EpisodeList from "./EpisodeList";
import ExternalLinks from "./ExternalLinks";
import InfoBadges from "./InfoBadges";
import MultilingualTitles from "./MultilingualTitles";
import StaffSection from "./StaffSection";
import StudioSection from "./StudioSection";
import VideoSection from "./VideoSection";

interface DetailsModalProps {
  isOpen: boolean;
  onClose: () => void;
  anime: { title: string; info: UnifiedMetadata | null } | null;
  items: DisplayAnimeItem[];
  siteMeta?: SiteMeta;
  onUpdate?: () => void;
}

export default function DetailsModal(props: DetailsModalProps) {
  const { isOpen, onClose, anime } = props;
  const { title } = anime || { title: "" };

  return (
    <Dialog.Root open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <AnimatePresence>
        {isOpen && (
          <Dialog.Portal forceMount>
            <Dialog.Overlay asChild forceMount>
              <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                className="fixed inset-0 z-[100] bg-black/60 backdrop-blur-sm"
              />
            </Dialog.Overlay>
            <Dialog.Content
              asChild
              forceMount
              onInteractOutside={(e) => {
                // If interacting with an element outside the React root (like an extension popup),
                // prevent the modal from closing.
                const target = e.target as HTMLElement;
                const root = document.getElementById("root");
                if (root && !root.contains(target)) {
                  e.preventDefault();
                }
              }}
            >
              {/*
                We use title as a key to force remount when switching anime.
                This ensures all local state (like status optimistic updates) is reset cleanly.
              */}
              <DetailsModalContent key={title} {...props} />
            </Dialog.Content>
          </Dialog.Portal>
        )}
      </AnimatePresence>
    </Dialog.Root>
  );
}

function DetailsModalContent({
  onClose,
  anime,
  items,
  siteMeta,
  onUpdate,
  ...radixProps
}: Omit<DetailsModalProps, "isOpen"> & Record<string, unknown>) {
  const { loggedIn } = useAuth();

  // Find the original item to get site links and user status
  const title = anime?.title || "";
  const originalItem = items.find((i) => i.title === title);

  // Use smart hook for metadata
  // We pass anime.info as initial data if it exists.
  // Enable the hook if we found the item OR if we have the title from URL
  const { metadata: info, loading } = useSmartMetadata(
    originalItem || ({ title } as DisplayAnimeItem), // Use a dummy item with just title if originalItem isn't loaded yet
    anime?.info || null,
    !!originalItem || !!title,
  );

  const { currentStatus, updateStatus } = useAnimeStatus({
    title,
    initialStatus: originalItem?.userStatus,
    initialScore: originalItem?.userScore,
    beginAt: originalItem?.begin,
    onUpdate,
  });

  let displaySites = originalItem?.sites || [];

  // Inject metadata source if missing from original sites
  if (
    info?.id &&
    info?.sourceSite &&
    !displaySites.some((s) => s.site === info.sourceSite)
  ) {
    displaySites = [...displaySites, { site: info.sourceSite, id: info.id }];
  }

  const sites = sortSites(displaySites, siteMeta);

  return (
    <div
      {...radixProps}
      className="fixed inset-0 z-[110] flex cursor-pointer items-center justify-center overflow-y-auto p-4 sm:p-6"
      onClick={onClose}
    >
      <motion.div
        layoutId={`card-${title}`}
        initial={false}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        transition={{ duration: 0.35, ease: [0.4, 0, 0.2, 1] }}
        className="relative flex max-h-[85vh] w-full max-w-4xl cursor-auto flex-col overflow-hidden rounded-3xl bg-white shadow-2xl outline-none dark:bg-gray-800"
        onClick={(e) => e.stopPropagation()}
      >
        <Dialog.Close asChild>
          <button className="absolute top-4 right-4 z-50 rounded-full bg-black/10 p-3 text-gray-800 backdrop-blur-sm transition-colors hover:bg-black/20 focus:ring-2 focus:ring-white/20 focus:outline-none dark:bg-white/10 dark:text-gray-200 dark:hover:bg-white/20">
            <X size={24} />
          </button>
        </Dialog.Close>

        <div className="flex flex-1 flex-col overflow-hidden md:flex-row">
          {/* Image Section - Show Skeleton if loading */}
          {loading && !info ? (
            <Skeleton
              className="h-64 md:h-full md:w-1/3"
              variant="rectangular"
            />
          ) : (
            <AnimeCover info={info} title={title} />
          )}

          {/* Content Section */}
          <motion.div
            layoutId={`content-${title}`}
            className="custom-scrollbar flex-1 space-y-6 overflow-y-auto p-6 md:p-8"
          >
            <div>
              <Dialog.Title asChild>
                <motion.h1
                  layoutId={`title-${title}`}
                  className="mb-3 text-2xl leading-tight font-black text-gray-900 md:text-3xl dark:text-white"
                >
                  {title}
                </motion.h1>
              </Dialog.Title>

              {/* Status Selector (Only if logged in) */}
              {loggedIn && (
                <div className="mb-4">
                  <CustomSelect
                    value={currentStatus.toString()}
                    onValueChange={updateStatus}
                    options={Object.entries(USER_STATUS_LABELS).map(
                      ([value, label]) => ({
                        value,
                        label: (value === "0"
                          ? "リストに追加"
                          : label) as string,
                      }),
                    )}
                    placeholder="状態を選択"
                    icon={<Bookmark size={16} />}
                    triggerClassName="bg-blue-50 border border-blue-200 text-blue-700 hover:bg-blue-100 dark:bg-blue-900/20 dark:border-blue-900/50 dark:text-blue-300 dark:hover:bg-blue-900/30 font-bold py-2"
                    contentClassName="z-[200]"
                  />
                </div>
              )}

              {loading && !info ? (
                <div className="flex flex-wrap gap-2">
                  <Skeleton className="h-6 w-16 rounded-md" />
                  <Skeleton className="h-6 w-20 rounded-md" />
                  <Skeleton className="h-6 w-14 rounded-md" />
                </div>
              ) : (
                <InfoBadges info={info} />
              )}
            </div>

            {loading && !info ? (
              <div className="space-y-8">
                {/* Titles & Links Placeholder */}
                <div className="space-y-4">
                  <Skeleton className="h-32 w-full rounded-2xl" />
                  <div className="flex gap-2">
                    <Skeleton className="h-8 w-24 rounded-full" />
                    <Skeleton className="h-8 w-24 rounded-full" />
                    <Skeleton className="h-8 w-24 rounded-full" />
                  </div>
                </div>

                {/* Description Placeholder */}
                <div className="space-y-2">
                  <Skeleton className="h-4 w-20 rounded" />
                  <div className="space-y-1">
                    <Skeleton className="h-4 w-full rounded" />
                    <Skeleton className="h-4 w-full rounded" />
                    <Skeleton className="h-4 w-3/4 rounded" />
                  </div>
                </div>

                {/* Cast/Staff Grid Placeholder */}
                <div className="grid grid-cols-1 gap-6 sm:grid-cols-2">
                  <div className="space-y-3">
                    <Skeleton className="h-4 w-24 rounded" />
                    <div className="grid grid-cols-2 gap-3">
                      {[1, 2, 3, 4].map((i) => (
                        <Skeleton key={i} className="h-16 w-full rounded-xl" />
                      ))}
                    </div>
                  </div>
                  <div className="space-y-3">
                    <Skeleton className="h-4 w-24 rounded" />
                    <div className="grid grid-cols-2 gap-3">
                      {[1, 2, 3, 4].map((i) => (
                        <Skeleton key={i} className="h-16 w-full rounded-xl" />
                      ))}
                    </div>
                  </div>
                </div>
              </div>
            ) : (
              <>
                {/* Multilingual Titles */}
                <MultilingualTitles info={info} originalItem={originalItem} />

                {/* Links Section */}
                <ExternalLinks
                  originalItem={originalItem}
                  siteMeta={siteMeta}
                  sites={sites}
                />

                {/* Description */}
                {info?.description && (
                  <div>
                    <h4 className="mb-2 text-sm font-black tracking-wider text-gray-400 uppercase dark:text-gray-500">
                      あらすじ
                    </h4>
                    <div className="prose prose-sm dark:prose-invert text-sm leading-relaxed text-gray-600 md:text-base dark:text-gray-300">
                      {(() => {
                        const cleanConfig = {
                          ALLOWED_TAGS: [
                            "b",
                            "i",
                            "em",
                            "strong",
                            "a",
                            "br",
                            "p",
                            "ul",
                            "ol",
                            "li",
                          ],
                          ALLOWED_ATTR: ["href", "target", "rel"],
                        };

                        const rawDescription = info.description || "";
                        const sanitized = DOMPurify.sanitize(
                          rawDescription,
                          cleanConfig,
                        );

                        // Collapse multiple newlines/brs into max 2
                        const collapsed = sanitized
                          .replace(/<br\s*\/?>/gi, "\n") // Convert br to newline
                          .replace(/\n{2,}/g, "\n") // Max 2 newlines
                          .trim();

                        return (
                          <div
                            className="whitespace-pre-wrap"
                            dangerouslySetInnerHTML={{ __html: collapsed }}
                          />
                        );
                      })()}
                    </div>
                  </div>
                )}

                {/* Videos */}
                <VideoSection videos={info?.videos} />

                {/* Episodes List */}
                <EpisodeList episodes={info?.episodesList} />

                {/* Studio & Cast */}
                <div className="grid grid-cols-1 gap-6 sm:grid-cols-2">
                  <StudioSection studios={info?.studios} />
                  <CastSection characters={info?.characters} />
                  <StaffSection staff={info?.staff} />
                </div>
              </>
            )}
          </motion.div>
        </div>
      </motion.div>
    </div>
  );
}
