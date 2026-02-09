import * as Dialog from "@radix-ui/react-dialog";
import { AnimatePresence, motion } from "framer-motion";
import { Clock, ExternalLink, PlayCircle, Star, X } from "lucide-react";
import { useEffect, useState } from "react";
import type {
  AnimeItem,
  SiteMeta,
  UnifiedMetadata,
  UniversalEpisode,
  UniversalStaff,
} from "../types";
import { sortSites } from "../utils/siteUtils";

interface DetailsModalProps {
  isOpen: boolean;
  onClose: () => void;
  anime: { title: string; info: UnifiedMetadata | null } | null;
  items: AnimeItem[];
  siteMeta?: SiteMeta;
}

export default function DetailsModal({
  isOpen,
  onClose,
  anime,
  items,
  siteMeta,
}: DetailsModalProps) {
  const [displayAnime, setDisplayAnime] = useState(anime);

  useEffect(() => {
    if (anime) {
      setDisplayAnime(anime);
    }
  }, [anime]);

  const { title, info } = anime || displayAnime || { title: "", info: null };

  // Find the original item to get site links
  const originalItem = items.find((i) => i.title === title);
  const sites = sortSites(originalItem?.sites || [], siteMeta);

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
              <div
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
                    <motion.div
                      layoutId={`image-${title}`}
                      className="relative flex aspect-[3/4] w-full items-center justify-center overflow-hidden bg-gray-100 md:aspect-auto md:w-2/5 dark:bg-gray-900"
                    >
                      {info?.coverImage?.extraLarge ||
                      info?.coverImage?.large ? (
                        <>
                          {/* Blurred background for better aesthetics with different aspect ratios */}
                          <img
                            src={
                              info.coverImage.extraLarge ||
                              info.coverImage.large
                            }
                            className="absolute inset-0 h-full w-full object-cover opacity-20 blur-2xl saturate-150 dark:opacity-40"
                            aria-hidden="true"
                          />
                          <img
                            src={
                              info.coverImage.extraLarge ||
                              info.coverImage.large
                            }
                            alt={title}
                            className="relative z-10 max-h-full max-w-full object-contain drop-shadow-xl"
                          />
                        </>
                      ) : (
                        <div className="flex h-full w-full items-center justify-center text-gray-400 italic">
                          No image available
                        </div>
                      )}
                      <div className="absolute inset-x-0 bottom-0 z-20 h-32 bg-gradient-to-t from-white to-transparent md:hidden dark:from-gray-800" />
                    </motion.div>

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
                        <div className="flex flex-wrap gap-2">
                          {!!info?.averageScore && info.averageScore > 0 && (
                            <div className="flex items-center gap-1.5 rounded-full border border-yellow-200/50 bg-yellow-50 px-3 py-1 text-sm font-bold text-yellow-700 dark:border-yellow-700/30 dark:bg-yellow-900/20 dark:text-yellow-400">
                              <Star size={14} className="fill-current" />
                              {info.averageScore}%
                            </div>
                          )}
                          {info?.episodes && (
                            <div className="flex items-center gap-1.5 rounded-full border border-purple-200/50 bg-purple-50 px-3 py-1 text-sm font-bold text-purple-700 dark:border-purple-700/30 dark:bg-purple-900/20 dark:text-purple-400">
                              <PlayCircle size={14} />
                              {info.episodes}話
                            </div>
                          )}
                          {info?.totalSeasons && (
                            <div className="flex items-center gap-1 rounded-full bg-purple-100 px-3 py-1 text-sm font-bold text-purple-700 dark:bg-purple-900/30 dark:text-purple-300">
                              シーズン{info.currentSeason || 1} / 全
                              {info.totalSeasons}シーズン
                            </div>
                          )}
                          {!!info?.runtime && info.runtime > 0 && (
                            <div className="flex items-center gap-1.5 rounded-full border border-gray-200/50 bg-gray-100 px-3 py-1 text-sm font-bold text-gray-700 dark:border-gray-700/30 dark:bg-gray-700/50 dark:text-gray-300">
                              <Clock size={14} className="stroke-current" />
                              {info.runtime}分
                            </div>
                          )}
                          {info?.contentRating && (
                            <div className="flex items-center gap-1.5 rounded-full border border-gray-200/50 bg-gray-100 px-3 py-1 text-sm font-bold text-gray-700 dark:border-gray-700/30 dark:bg-gray-700/50 dark:text-gray-300">
                              {info.contentRating}
                            </div>
                          )}
                          {info?.genres?.slice(0, 3).map((g: string) => (
                            <span
                              key={g}
                              className="rounded-full bg-gray-100 px-3 py-1 text-sm font-medium text-gray-600 dark:bg-gray-700/50 dark:text-gray-300"
                            >
                              {g}
                            </span>
                          ))}
                        </div>
                      </div>

                      {/* Multilingual Titles */}
                      <div className="space-y-2 rounded-2xl border border-gray-100 bg-gray-50 p-4 text-sm dark:border-gray-700/50 dark:bg-gray-900/50">
                        {info?.title?.native && (
                          <div className="flex gap-3">
                            <span className="w-14 shrink-0 font-bold text-gray-400">
                              日本語
                            </span>
                            <span className="text-gray-700 dark:text-gray-200">
                              {info.title.native}
                            </span>
                          </div>
                        )}
                        {info?.title?.native !== info?.title?.romaji &&
                          info?.title?.romaji && (
                            <div className="flex gap-3">
                              <span className="w-14 shrink-0 font-bold text-gray-400">
                                ローマ字
                              </span>
                              <span className="text-gray-700 dark:text-gray-200">
                                {info.title.romaji}
                              </span>
                            </div>
                          )}
                        {/* Local data translations */}
                        {originalItem?.titleTranslate &&
                          Object.entries(originalItem.titleTranslate).map(
                            ([lang, titles]) => {
                              if (!titles?.length) return null;
                              return (
                                <div key={lang} className="flex gap-3">
                                  <span className="w-14 shrink-0 font-bold text-gray-400 uppercase">
                                    {{
                                      "zh-Hans": "簡体字",
                                      "zh-Hant": "繁体字",
                                      en: "英語",
                                      ja: "日本語",
                                    }[lang] || lang}
                                  </span>
                                  <span className="text-gray-700 dark:text-gray-200">
                                    {titles.join(" / ")}
                                  </span>
                                </div>
                              );
                            },
                          )}

                        {info?.title?.english && (
                          <div className="flex gap-3">
                            <span className="w-14 shrink-0 font-bold text-gray-400">
                              英語
                            </span>
                            <span className="text-gray-700 dark:text-gray-200">
                              {info.title.english}
                            </span>
                          </div>
                        )}
                      </div>

                      {/* Links Section */}
                      {originalItem?.officialSite && (
                        <div className="mb-4">
                          <h4 className="mb-2 text-sm font-black tracking-wider text-gray-400 uppercase dark:text-gray-500">
                            公式サイト
                          </h4>
                          <a
                            href={originalItem.officialSite}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="inline-flex items-center gap-1.5 rounded-xl border border-purple-100 bg-purple-50 px-3 py-1.5 text-sm font-bold text-purple-600 transition-colors hover:bg-purple-100 dark:border-purple-800/50 dark:bg-purple-900/20 dark:text-purple-400 dark:hover:bg-purple-900/30"
                          >
                            公式サイト
                            <ExternalLink size={12} />
                          </a>
                        </div>
                      )}
                      {Object.entries({
                        onair: {
                          label: "配信",
                          sites: sites.filter(
                            (s) => siteMeta?.[s.site]?.type === "onair",
                          ),
                        },
                        info: {
                          label: "情報",
                          sites: sites.filter(
                            (s) => siteMeta?.[s.site]?.type === "info",
                          ),
                        },
                        resource: {
                          label: "リソース",
                          sites: sites.filter(
                            (s) => siteMeta?.[s.site]?.type === "resource",
                          ),
                        },
                        other: {
                          label: "その他",
                          sites: sites.filter(
                            (s) =>
                              !siteMeta?.[s.site]?.type ||
                              !["onair", "info", "resource"].includes(
                                siteMeta[s.site]!.type,
                              ),
                          ),
                        },
                      }).map(([key, group]) => {
                        if (group.sites.length === 0) return null;

                        return (
                          <div key={key}>
                            <h4 className="mb-2 text-sm font-black tracking-wider text-gray-400 uppercase dark:text-gray-500">
                              {group.label}
                            </h4>
                            <div className="mb-4 flex flex-wrap gap-2">
                              {group.sites.map((site, idx) => {
                                const meta = siteMeta?.[site.site];
                                const url =
                                  site.url ||
                                  meta?.urlTemplate?.replace(
                                    "{{id}}",
                                    site.id || "",
                                  );
                                if (!url) return null;

                                return (
                                  <a
                                    key={`${site.site}-${idx}`}
                                    href={url}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    className="flex items-center gap-1.5 rounded-xl border border-blue-100 bg-blue-50 px-3 py-1.5 text-sm font-bold text-blue-600 transition-colors hover:bg-blue-100 dark:border-blue-800/50 dark:bg-blue-900/20 dark:text-blue-400 dark:hover:bg-blue-900/30"
                                  >
                                    {meta?.title || site.site}
                                    <ExternalLink size={12} />
                                  </a>
                                );
                              })}
                            </div>
                          </div>
                        );
                      })}

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

                      {/* Episodes List */}
                      {!!info?.episodesList && info.episodesList.length > 0 && (
                        <div>
                          <h4 className="mb-3 text-sm font-black tracking-wider text-gray-400 uppercase dark:text-gray-500">
                            エピソード
                          </h4>
                          <div className="custom-scrollbar grid max-h-80 grid-cols-1 gap-2 overflow-y-auto pr-2">
                            {info.episodesList.map((ep: UniversalEpisode) => (
                              <div
                                key={ep.number}
                                className="group/ep flex flex-col gap-1.5 rounded-xl border border-gray-100 bg-gray-50 p-2.5 text-sm transition-colors hover:bg-white dark:border-gray-700/50 dark:bg-gray-900/40 dark:hover:bg-gray-800"
                              >
                                <div className="flex items-center gap-3">
                                  <span className="w-6 shrink-0 text-center font-black text-blue-600 dark:text-blue-400">
                                    {ep.number}
                                  </span>
                                  <span className="truncate font-medium text-gray-700 transition-colors group-hover/ep:text-blue-600 dark:text-gray-200 dark:group-hover/ep:text-blue-400">
                                    {ep.title || `Episode ${ep.number}`}
                                  </span>
                                  <div className="ml-auto flex shrink-0 items-center gap-2">
                                    {ep.runtime && (
                                      <span className="self-center rounded bg-gray-100 px-1.5 py-0.5 text-[10px] font-bold text-gray-400 dark:bg-gray-800 dark:text-gray-500">
                                        {ep.runtime}分
                                      </span>
                                    )}
                                    {ep.airDate && (
                                      <span className="self-center text-[10px] text-gray-400 dark:text-gray-500">
                                        {ep.airDate}
                                      </span>
                                    )}
                                  </div>
                                </div>
                                {ep.overview && (
                                  <p className="line-clamp-2 pl-9 text-xs text-gray-500 transition-all group-hover/ep:line-clamp-none dark:text-gray-400">
                                    {ep.overview}
                                  </p>
                                )}
                              </div>
                            ))}
                          </div>
                        </div>
                      )}

                      {/* Studio & Cast */}
                      <div className="grid grid-cols-1 gap-6 sm:grid-cols-2">
                        {!!info?.studios && info.studios.length > 0 && (
                          <div className="sm:col-span-2">
                            <h4 className="mb-2 text-sm font-black tracking-wider text-gray-400 uppercase dark:text-gray-500">
                              スタジオ
                            </h4>
                            <div className="flex flex-wrap gap-2">
                              {info.studios.map(
                                (studio: string, idx: number) => (
                                  <span
                                    key={idx}
                                    className="rounded-xl border border-purple-200/50 bg-gradient-to-r from-purple-50 to-pink-50 px-3 py-1.5 text-sm font-medium text-purple-700 dark:border-purple-700/30 dark:from-purple-900/20 dark:to-pink-900/20 dark:text-purple-300"
                                  >
                                    {studio}
                                  </span>
                                ),
                              )}
                            </div>
                          </div>
                        )}
                        {!!info?.characters && info.characters.length > 0 && (
                          <div className="sm:col-span-2">
                            <h4 className="mb-3 text-sm font-black tracking-wider text-gray-400 uppercase dark:text-gray-500">
                              キャスト
                            </h4>
                            <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
                              {info.characters
                                .slice(0, 6)
                                .map((char, idx: number) => (
                                  <div
                                    key={idx}
                                    className="group flex flex-col rounded-2xl border border-gray-100 bg-gray-50 p-3 transition-colors hover:border-blue-200 dark:border-gray-700/50 dark:bg-gray-900/40 dark:hover:border-blue-900/50"
                                  >
                                    <div className="truncate font-bold text-gray-800 transition-colors group-hover:text-blue-600 dark:text-gray-100 dark:group-hover:text-blue-400">
                                      {char.name}
                                    </div>
                                    {char.voiceActor && (
                                      <div className="mt-1 flex items-center gap-1.5">
                                        <span className="rounded bg-gray-200 px-1 py-0.5 text-[9px] font-black tracking-tighter text-gray-500 uppercase dark:bg-gray-800 dark:text-gray-400">
                                          CV
                                        </span>
                                        <span className="truncate text-xs text-gray-500 dark:text-gray-400">
                                          {char.voiceActor}
                                        </span>
                                      </div>
                                    )}
                                  </div>
                                ))}
                            </div>
                          </div>
                        )}
                        {!!info?.staff && info.staff.length > 0 && (
                          <div className="sm:col-span-2">
                            <h4 className="mb-3 text-sm font-black tracking-wider text-gray-400 uppercase dark:text-gray-500">
                              スタッフ
                            </h4>
                            {(() => {
                              const grouped = info.staff
                                .slice(0, 12)
                                .reduce(
                                  (
                                    acc: Record<string, UniversalStaff[]>,
                                    member: UniversalStaff,
                                  ) => {
                                    const deptMap: Record<string, string> = {
                                      Directing: "監督・演出",
                                      Writing: "脚本",
                                      Sound: "音響",
                                      Camera: "撮影",
                                      Art: "美術",
                                      Production: "制作",
                                      "Visual Effects": "視覚効果",
                                      Editing: "編集",
                                      Lighting: "照明",
                                      "Costume & Make-Up": "衣装・メイク",
                                      Creator: "原案・原作",
                                      Crew: "スタッフ",
                                    };
                                    const deptEnglish =
                                      member.department || "Other";
                                    const dept =
                                      deptMap[deptEnglish] || deptEnglish;
                                    if (!acc[dept]) acc[dept] = [];
                                    acc[dept].push(member);
                                    return acc;
                                  },
                                  {},
                                );
                              return Object.entries(grouped).map(
                                ([dept, members]: [
                                  string,
                                  UniversalStaff[],
                                ]) => (
                                  <div key={dept} className="mb-3">
                                    <div className="mb-1.5 text-[10px] font-bold tracking-wider text-gray-400 uppercase dark:text-gray-500">
                                      {dept}
                                    </div>
                                    <div className="grid grid-cols-2 gap-2 sm:grid-cols-3 lg:grid-cols-4">
                                      {members.map(
                                        (
                                          member: UniversalStaff,
                                          idx: number,
                                        ) => (
                                          <div
                                            key={idx}
                                            className="flex flex-col rounded-lg border border-gray-100 bg-gray-50 p-2 text-sm dark:border-gray-700/50 dark:bg-gray-900/40"
                                          >
                                            <span className="truncate text-[10px] font-semibold tracking-tight text-blue-600 uppercase dark:text-blue-400">
                                              {member.role}
                                            </span>
                                            <span className="mt-0.5 truncate font-medium text-gray-700 dark:text-gray-200">
                                              {member.name}
                                            </span>
                                          </div>
                                        ),
                                      )}
                                    </div>
                                  </div>
                                ),
                              );
                            })()}
                          </div>
                        )}
                      </div>
                    </motion.div>
                  </div>
                </motion.div>
              </div>
            </Dialog.Content>
          </Dialog.Portal>
        )}
      </AnimatePresence>
    </Dialog.Root>
  );
}
