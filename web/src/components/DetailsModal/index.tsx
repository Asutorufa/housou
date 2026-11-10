import * as Dialog from "@radix-ui/react-dialog";
import DOMPurify from "../../utils/domPurifyConfig";
import { Bookmark, X } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { startTransition, useEffect, useMemo, useState } from "react";
import { useAuth } from "../../contexts/AuthContext";
import { useAnimeStatus } from "../../hooks/useAnimeStatus";
import { useSmartMetadata } from "../../hooks/useSmartMetadata";
import { MORPH_SETTLE_MS, MORPH_TRANSITION } from "../../styles/motion";
import { focusRingClassName } from "../../styles/uiClasses";
import type { DisplayAnimeItem, SiteMeta, UnifiedMetadata } from "../../types";
import { USER_STATUS_LABELS } from "../../types";
import { sortSites } from "../../utils/siteUtils";
import CustomSelect from "../CustomSelect";
import Skeleton from "../Skeleton";
import AnimeCover from "./AnimeCover";
import CastSection from "./CastSection";
import CommentSection from "./CommentSection";
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
  /** Opened by clicking a card, so the panel can grow out of it. */
  fromCard?: boolean;
  items: DisplayAnimeItem[];
  siteMeta?: SiteMeta;
  authEnabled: boolean;
  onUpdate?: () => void;
}

const DESCRIPTION_SANITIZE_CONFIG = {
  ALLOWED_TAGS: ["b", "i", "em", "strong", "a", "br", "p", "ul", "ol", "li"],
  ALLOWED_ATTR: ["href", "target", "rel"],
};

function sanitizeDescription(rawDescription: string) {
  // Sanitize and get a DOM fragment to avoid global hooks
  const fragment = DOMPurify.sanitize(rawDescription, {
    ...DESCRIPTION_SANITIZE_CONFIG,
    RETURN_DOM_FRAGMENT: true,
  });

  // Replace <br> with newlines in the fragment
  fragment.querySelectorAll("br").forEach((br) => {
    br.replaceWith("\n");
  });

  // Serialize back to string
  const container = document.createElement("div");
  container.appendChild(fragment);

  // Collapse multiple newlines into max 2
  return container.innerHTML.replace(/\n{2,}/g, "\n").trim();
}

export default function DetailsModal(props: DetailsModalProps) {
  const { isOpen, onClose, anime } = props;
  const { items, siteMeta, authEnabled, onUpdate, fromCard } = props;
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
                transition={{ duration: 0.3, ease: "easeOut" }}
                // No backdrop blur: it re-blurs the whole viewport on every frame of the morph,
                // which is what weak GPUs choke on.
                className="fixed inset-0 z-[100] bg-black/65"
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
              <DetailsModalContent
                key={title}
                onClose={onClose}
                anime={anime}
                fromCard={fromCard}
                items={items}
                siteMeta={siteMeta}
                authEnabled={authEnabled}
                onUpdate={onUpdate}
              />
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
  fromCard,
  items,
  siteMeta,
  authEnabled,
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
  const {
    metadata: info,
    loading,
    error: metadataError,
    retry,
  } = useSmartMetadata(
    originalItem || { title }, // Use a dummy item with just title if originalItem isn't loaded yet
    anime?.info || null,
    !!originalItem || !!title,
    "detail",
  );

  const {
    currentStatus,
    updateStatus,
    updating,
    error: statusError,
  } = useAnimeStatus({
    title,
    initialStatus: originalItem?.userStatus,
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

  // Rendering the whole body in the click that opens the modal delays the first frame of the
  // morph, and it is not visible until the panel has grown anyway. It is mounted once the morph
  // is over, as a transition so it never blocks a frame for long.
  const [bodyReady, setBodyReady] = useState(false);
  useEffect(() => {
    const timer = setTimeout(
      () => startTransition(() => setBodyReady(true)),
      MORPH_SETTLE_MS,
    );
    return () => clearTimeout(timer);
  }, []);

  const description = bodyReady ? info?.description : undefined;
  const descriptionHtml = useMemo(
    () => (description ? sanitizeDescription(description) : ""),
    [description],
  );

  return (
    <div
      {...radixProps}
      className="fixed inset-0 z-[110] flex cursor-pointer items-center justify-center overflow-y-auto p-4 sm:p-6"
      onClick={onClose}
    >
      <motion.div
        layoutId={`card-${title}`}
        // The hand-over to the card is timed explicitly below; Motion's automatic cross-fade
        // would keep the whole panel see-through for the entire morph.
        layoutCrossfade={false}
        // Without a card to grow from (deep link), fall back to a plain fade + scale.
        initial={fromCard ? false : { opacity: 0, scale: 0.96 }}
        animate={{ opacity: 1, scale: 1 }}
        // Fade only once the panel is (almost) back on the card, handing over to it.
        exit={{
          opacity: 0,
          transition: { duration: 0.14, delay: 0.24, ease: "linear" },
        }}
        transition={MORPH_TRANSITION}
        // Motion only corrects border radius during layout animations via style.
        style={{ borderRadius: 24 }}
        className="relative flex max-h-[85dvh] w-full max-w-4xl cursor-auto flex-col overflow-hidden shadow-2xl outline-none"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Surface fades in over the card so its text cross-fades instead of popping. */}
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ duration: 0.1, ease: "linear" }}
          className="absolute inset-0 bg-white dark:bg-gray-800"
        />
        {/*
          Only the panel box and the cover morph from the card. This wrapper opts into layout
          projection so everything inside keeps its real size (no stretched text) and is simply
          revealed as the panel grows. It must not clip: the cover leaves it on the way back to
          the card (the panel above does the clipping).
        */}
        <motion.div
          layout
          transition={MORPH_TRANSITION}
          className="relative flex min-h-0 flex-1 flex-col md:flex-row"
        >
          <Dialog.Close asChild>
            <motion.button
              exit={{ opacity: 0, transition: { duration: 0.12 } }}
              aria-label="作品詳細を閉じる"
              className={`absolute top-4 right-4 z-50 rounded-full bg-black/15 p-3 text-gray-800 transition-colors hover:bg-black/25 dark:bg-white/15 dark:text-gray-200 dark:hover:bg-white/25 ${focusRingClassName}`}
            >
              <X size={24} />
            </motion.button>
          </Dialog.Close>

          <AnimeCover info={info} title={title} loading={loading} />

          {/* Content Section */}
          <motion.div
            initial={{ opacity: 0 }}
            animate={{
              opacity: 1,
              transition: { duration: 0.25, delay: 0.12, ease: "easeOut" },
            }}
            exit={{ opacity: 0, transition: { duration: 0.12 } }}
            className="custom-scrollbar flex-1 space-y-6 overflow-y-auto p-6 md:p-8"
          >
            <div>
              <Dialog.Title asChild>
                <h1 className="mb-3 pr-12 text-2xl leading-tight font-black text-gray-900 md:text-3xl dark:text-white">
                  {title}
                </h1>
              </Dialog.Title>
              <Dialog.Description asChild>
                <p className="sr-only">
                  作品詳細モーダルです。あらすじ、外部リンク、キャスト、スタッフ、エピソード情報を確認できます。
                </p>
              </Dialog.Description>

              {/* Status Selector (Only if logged in) */}
              {loggedIn && (
                <div className="mb-4">
                  <CustomSelect
                    disabled={updating}
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

              {statusError && (
                <p role="alert" className="text-sm text-red-500">
                  {statusError}
                </p>
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

            {metadataError && (
              <div role="alert" className="text-sm text-red-500">
                {metadataError}{" "}
                <button onClick={retry} className="underline">
                  再試行
                </button>
              </div>
            )}
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
            ) : !bodyReady ? null : (
              <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                transition={{ duration: 0.2, ease: "easeOut" }}
                className="space-y-6"
              >
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
                    <div
                      className="rich-text text-sm leading-relaxed whitespace-pre-wrap text-gray-600 md:text-base dark:text-gray-300"
                      dangerouslySetInnerHTML={{ __html: descriptionHtml }}
                    />
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
                {/* Comments Section */}
                {authEnabled && (
                  <CommentSection title={title} viewerStatus={currentStatus} />
                )}
              </motion.div>
            )}
          </motion.div>
        </motion.div>
      </motion.div>
    </div>
  );
}
