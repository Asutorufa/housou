import * as Dialog from "@radix-ui/react-dialog";
import { Bookmark, X } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useAuth } from "../../contexts/AuthContext";
import { useAnimeStatus } from "../../hooks/useAnimeStatus";
import type { DisplayAnimeItem, SiteMeta, UnifiedMetadata } from "../../types";
import { USER_STATUS_LABELS } from "../../types";
import { sortSites } from "../../utils/siteUtils";
import CustomSelect from "../CustomSelect";
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
            <Dialog.Overlay asChild>
              <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                className="fixed inset-0 z-[100] bg-black/60 backdrop-blur-sm"
              />
            </Dialog.Overlay>
            <Dialog.Content asChild>
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
  const { title, info } = anime || { title: "", info: null };

  // Find the original item to get site links and user status
  const originalItem = items.find((i) => i.title === title);

  const { currentStatus, updateStatus } = useAnimeStatus({
    title,
    initialStatus: originalItem?.userStatus,
    initialScore: originalItem?.userScore,
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
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        transition={{ duration: 0.35, ease: [0.4, 0, 0.2, 1] }}
        className="relative flex max-h-[85vh] w-full max-w-4xl cursor-default flex-col overflow-hidden rounded-3xl bg-white shadow-2xl outline-none dark:bg-gray-800"
        onClick={(e) => e.stopPropagation()}
      >
        <Dialog.Close asChild>
          <button className="absolute top-4 right-4 z-50 rounded-full bg-black/10 p-3 text-gray-800 backdrop-blur-sm transition-colors hover:bg-black/20 focus:ring-2 focus:ring-white/20 focus:outline-none dark:bg-white/10 dark:text-gray-200 dark:hover:bg-white/20">
            <X size={24} />
          </button>
        </Dialog.Close>

        <div className="flex flex-1 flex-col overflow-hidden md:flex-row">
          {/* Image Section */}
          <AnimeCover info={info} title={title} />

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

              <InfoBadges info={info} />
            </div>

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
                  {info.description}
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
          </motion.div>
        </div>
      </motion.div>
    </div>
  );
}
