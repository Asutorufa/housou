import * as Dialog from "@radix-ui/react-dialog";
import DOMPurify from "../../utils/domPurifyConfig";
import { Bookmark, X } from "lucide-react";
import {
  animate,
  AnimatePresence,
  motion,
  usePresence,
  useReducedMotion,
} from "motion/react";
import {
  startTransition,
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { useAuth } from "../../contexts/AuthContext";
import { useAnimeStatus } from "../../hooks/useAnimeStatus";
import { useSmartMetadata } from "../../hooks/useSmartMetadata";
import {
  MORPH_CLOSE_DURATION,
  MORPH_EASE,
  MORPH_OPEN_DURATION,
  MORPH_SETTLE_MS,
} from "../../styles/motion";
import { focusRingClassName } from "../../styles/uiClasses";
import type { DisplayAnimeItem, SiteMeta, UnifiedMetadata } from "../../types";
import { USER_STATUS_LABELS } from "../../types";
import {
  applyCoverMorphAt,
  applyPanelMorphAt,
  captureCardMorphSnapshot,
  computeCoverMorphFrame,
  computePanelMorphFrame,
  isRectInViewport,
  type CardMorphSnapshot,
} from "../../utils/coverCrop";
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
  /** Live geometry captured from the clicked card before state updates hide it. */
  cardSnapshot?: CardMorphSnapshot | null;
  /** Called once the modal is painted over the card so the card can hide cleanly. */
  onMorphStart?: (title: string) => void;
  /** Called right before the closing modal hands back to the card. */
  onMorphEnd?: (title: string) => void;
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
  const {
    items,
    siteMeta,
    authEnabled,
    onUpdate,
    fromCard,
    cardSnapshot,
    onMorphStart,
    onMorphEnd,
  } = props;
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
                transition={{ duration: 0.28, ease: "easeOut" }}
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
                cardSnapshot={cardSnapshot}
                onMorphStart={onMorphStart}
                onMorphEnd={onMorphEnd}
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
  cardSnapshot,
  onMorphStart,
  onMorphEnd,
  items,
  siteMeta,
  authEnabled,
  onUpdate,
  ...radixProps
}: Omit<DetailsModalProps, "isOpen"> & Record<string, unknown>) {
  const { loggedIn } = useAuth();
  const [isPresent, safeToRemove] = usePresence();
  const reducedMotion = useReducedMotion();

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

  // Capture initial card geometry once on mount so re-renders never restart the open morph.
  const [initialSnapshot] = useState<CardMorphSnapshot | null>(() => {
    if (!fromCard) return null;
    return cardSnapshot ?? captureCardMorphSnapshot(title, true);
  });
  const canMorphOnOpen = !!initialSnapshot && !reducedMotion;

  // Rendering the whole body in the click that opens the modal delays the first frame of the
  // morph. With the panel locked to a fixed height, mounting the body after the morph never
  // changes the panel's size or triggers a secondary layout jump.
  const [bodyReady, setBodyReady] = useState(!canMorphOnOpen);
  useEffect(() => {
    if (bodyReady || !isPresent) return;
    const timer = setTimeout(
      () => startTransition(() => setBodyReady(true)),
      MORPH_SETTLE_MS,
    );
    return () => clearTimeout(timer);
  }, [bodyReady, isPresent]);

  const panelRef = useRef<HTMLDivElement>(null);
  const coverImgRef = useRef<HTMLImageElement>(null);
  const openCancelRef = useRef<(() => void) | null>(null);
  const morphProgressRef = useRef(1);
  const morphEndedRef = useRef(false);
  const exitStartedRef = useRef(false);
  const safeToRemoveRef = useRef(safeToRemove);
  const onMorphEndRef = useRef(onMorphEnd);
  useEffect(() => {
    safeToRemoveRef.current = safeToRemove;
    onMorphEndRef.current = onMorphEnd;
  });

  // Hide the source card once the modal has mounted, and always restore it on unmount.
  useLayoutEffect(() => {
    if (!title || !initialSnapshot) return;
    morphEndedRef.current = false;
    onMorphStart?.(title);
    return () => {
      if (!morphEndedRef.current) {
        morphEndedRef.current = true;
        onMorphEnd?.(title);
      }
    };
  }, [title, initialSnapshot, onMorphStart, onMorphEnd]);

  // Open morph: pure GPU transform + rounded clip-path from the card's exact rect to the modal.
  useLayoutEffect(() => {
    const panel = panelRef.current;
    if (!panel) return;

    const resetStyles = () => {
      panel.style.transform = "";
      panel.style.clipPath = "";
      panel.style.opacity = "";
      if (coverImgRef.current) {
        coverImgRef.current.style.transform = "";
        coverImgRef.current.style.clipPath = "";
      }
    };

    resetStyles();
    morphProgressRef.current = 1;

    if (reducedMotion) return;

    const modalRect = panel.getBoundingClientRect();
    const canRunCardMorph =
      !!initialSnapshot &&
      modalRect.width > 0 &&
      modalRect.height > 0 &&
      initialSnapshot.cardRect.width > 0 &&
      initialSnapshot.cardRect.height > 0;

    if (!canRunCardMorph) {
      // Deep link or jsdom fallback: subtle fade + scale.
      if (modalRect.width <= 0) return;
      let cancelled = false;
      const fallback = animate(
        panel,
        {
          opacity: [0, 1],
          transform: ["scale(0.96)", "scale(1)"],
        },
        { duration: 0.24, ease: MORPH_EASE },
      );
      const cancel = () => {
        cancelled = true;
        fallback.stop();
      };
      openCancelRef.current = cancel;
      void fallback.then(() => {
        if (cancelled) return;
        panel.style.opacity = "";
        panel.style.transform = "";
      });
      return () => {
        cancel();
        openCancelRef.current = null;
        resetStyles();
      };
    }

    const panelFrame = computePanelMorphFrame(
      initialSnapshot.cardRect,
      modalRect,
    );

    const coverImg = coverImgRef.current;
    const targetCoverRect =
      coverImg &&
      initialSnapshot.coverPosterRect &&
      initialSnapshot.coverWindowRect
        ? coverImg.getBoundingClientRect()
        : null;
    const coverFrame =
      coverImg &&
      targetCoverRect &&
      targetCoverRect.width > 0 &&
      targetCoverRect.height > 0 &&
      initialSnapshot.coverPosterRect &&
      initialSnapshot.coverWindowRect
        ? computeCoverMorphFrame(
            initialSnapshot.coverPosterRect,
            initialSnapshot.coverWindowRect,
            targetCoverRect,
            modalRect,
            panelFrame,
          )
        : null;

    // Paint frame 0 synchronously before first browser paint, then drive panel & cover
    // from one scalar progress so transform and clip-path never skew by a frame.
    morphProgressRef.current = 0;
    applyPanelMorphAt(panel, panelFrame, 0);
    if (coverImg && coverFrame) applyCoverMorphAt(coverImg, coverFrame, 0);

    let cancelled = false;
    const control = animate(0, 1, {
      duration: MORPH_OPEN_DURATION,
      ease: MORPH_EASE,
      onUpdate: (progress) => {
        morphProgressRef.current = progress;
        applyPanelMorphAt(panel, panelFrame, progress);
        if (coverImg && coverFrame) {
          applyCoverMorphAt(coverImg, coverFrame, progress);
        }
      },
    });
    const cancel = () => {
      cancelled = true;
      control.stop();
    };
    openCancelRef.current = cancel;

    void control.then(() => {
      if (cancelled) return;
      morphProgressRef.current = 1;
      // Clear clip-path once settled so box-shadow renders unclipped at rest.
      panel.style.transform = "";
      panel.style.clipPath = "";
      if (coverImg && coverFrame) {
        coverImg.style.transform = "";
        coverImg.style.clipPath = "";
      }
    });

    return () => {
      cancel();
      openCancelRef.current = null;
      resetStyles();
    };
  }, [initialSnapshot, reducedMotion]);

  // Close morph: reverse back to the card's live resting rect on top of the overlay, then hand off.
  useLayoutEffect(() => {
    if (isPresent || exitStartedRef.current) return;
    exitStartedRef.current = true;

    const panel = panelRef.current;
    const finishExit = () => {
      if (!morphEndedRef.current) {
        morphEndedRef.current = true;
        onMorphEndRef.current?.(title);
      }
      safeToRemoveRef.current?.();
    };

    if (!panel || reducedMotion) {
      finishExit();
      return;
    }

    // Stop any open animation still in flight and remember its current progress so closing
    // mid-open reverses smoothly from where the panel is instead of snapping to full size.
    const startProgress = Math.min(1, Math.max(0, morphProgressRef.current));
    openCancelRef.current?.();
    openCancelRef.current = null;

    panel.style.transform = "";
    panel.style.clipPath = "";
    if (coverImgRef.current) {
      coverImgRef.current.style.transform = "";
      coverImgRef.current.style.clipPath = "";
    }

    const modalRect = panel.getBoundingClientRect();
    if (modalRect.width <= 0 || modalRect.height <= 0) {
      finishExit();
      return;
    }

    const liveCard = initialSnapshot
      ? captureCardMorphSnapshot(title, false)
      : null;
    const canMorphBack = !!liveCard && isRectInViewport(liveCard.cardRect);

    let cancelled = false;

    if (!canMorphBack) {
      // Card scrolled out of view or opened from a deep link: clean fade + scale out.
      if (!morphEndedRef.current) {
        morphEndedRef.current = true;
        onMorphEndRef.current?.(title);
      }
      const fadeOut = animate(
        panel,
        {
          opacity: [1, 0],
          transform: ["scale(1)", "scale(0.96)"],
        },
        { duration: 0.2, ease: "easeOut" },
      );
      void fadeOut.then(() => {
        if (!cancelled) safeToRemoveRef.current?.();
      });
      return () => {
        cancelled = true;
        fadeOut.stop();
      };
    }

    if (startProgress <= 0.01) {
      finishExit();
      return;
    }

    const panelFrame = computePanelMorphFrame(liveCard.cardRect, modalRect);
    const coverImg = coverImgRef.current;
    const currentCoverRect =
      coverImg && liveCard.coverPosterRect && liveCard.coverWindowRect
        ? coverImg.getBoundingClientRect()
        : null;
    const coverFrame =
      coverImg &&
      currentCoverRect &&
      currentCoverRect.width > 0 &&
      currentCoverRect.height > 0 &&
      liveCard.coverPosterRect &&
      liveCard.coverWindowRect
        ? computeCoverMorphFrame(
            liveCard.coverPosterRect,
            liveCard.coverWindowRect,
            currentCoverRect,
            modalRect,
            panelFrame,
          )
        : null;

    // Immediately restore the interrupted progress before paint, then reverse smoothly to 0.
    applyPanelMorphAt(panel, panelFrame, startProgress);
    if (coverImg && coverFrame) {
      applyCoverMorphAt(coverImg, coverFrame, startProgress);
    }

    const dissolveThreshold = Math.min(0.03, startProgress);
    const control = animate(startProgress, 0, {
      duration: Math.max(0.12, MORPH_CLOSE_DURATION * startProgress),
      ease: MORPH_EASE,
      onUpdate: (t) => {
        morphProgressRef.current = t;
        applyPanelMorphAt(panel, panelFrame, t);
        if (coverImg && coverFrame) {
          applyCoverMorphAt(coverImg, coverFrame, t);
        }
        if (t <= dissolveThreshold) {
          if (!morphEndedRef.current) {
            morphEndedRef.current = true;
            onMorphEndRef.current?.(title);
          }
          panel.style.opacity = String(
            Math.max(0, t / Math.max(0.001, dissolveThreshold)),
          );
        }
      },
    });

    void control.then(() => {
      if (!cancelled) finishExit();
    });

    return () => {
      cancelled = true;
      control.stop();
    };
  }, [isPresent, reducedMotion, title, initialSnapshot]);

  // Stacked (narrow) layout: scrolling the body collapses the cover above it. The offset goes
  // straight into a CSS variable, so scrolling never re-renders the modal.
  const layoutRef = useRef<HTMLDivElement>(null);
  const handleBodyScroll = (event: React.UIEvent<HTMLDivElement>) => {
    layoutRef.current?.style.setProperty(
      "--cover-shrink",
      `${event.currentTarget.scrollTop}px`,
    );
  };

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
      <div
        ref={panelRef}
        className="relative flex h-[85dvh] w-full max-w-4xl cursor-auto flex-col overflow-hidden rounded-3xl bg-white shadow-2xl outline-none md:h-[min(85dvh,48rem)] dark:bg-gray-800"
        onClick={(e) => e.stopPropagation()}
      >
        <div
          ref={layoutRef}
          className="relative flex min-h-0 flex-1 flex-col md:flex-row"
        >
          <Dialog.Close asChild>
            <motion.button
              initial={{ opacity: fromCard ? 0 : 1 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0, transition: { duration: 0.1 } }}
              transition={{
                duration: 0.18,
                delay: fromCard ? 0.16 : 0,
                ease: "easeOut",
              }}
              aria-label="作品詳細を閉じる"
              className={`absolute top-4 right-4 z-50 rounded-full bg-black/15 p-3 text-gray-800 transition-colors hover:bg-black/25 dark:bg-white/15 dark:text-gray-200 dark:hover:bg-white/25 ${focusRingClassName}`}
            >
              <X size={24} />
            </motion.button>
          </Dialog.Close>

          <AnimeCover
            info={info}
            title={title}
            loading={loading}
            fromCard={fromCard}
            initialAspect={initialSnapshot?.coverAspect}
            imgRef={coverImgRef}
          />

          {/* Content Section */}
          <motion.div
            initial={{ opacity: 0, y: fromCard ? 10 : 0 }}
            animate={{
              opacity: 1,
              y: 0,
              transition: {
                duration: 0.24,
                delay: fromCard ? 0.1 : 0,
                ease: "easeOut",
              },
            }}
            exit={{ opacity: 0, transition: { duration: 0.11 } }}
            onScroll={handleBodyScroll}
            // The extra top padding equals what the cover has given up, so the text moves
            // exactly with the scroll while the cover collapses instead of at twice the speed.
            className="custom-scrollbar flex-1 space-y-6 overflow-y-auto p-6 pt-[calc(1.5rem+min(var(--cover-shrink,0px),calc(40dvh-6rem)))] md:p-8"
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
        </div>
      </div>
    </div>
  );
}
