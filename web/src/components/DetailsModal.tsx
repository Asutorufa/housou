import * as Dialog from '@radix-ui/react-dialog'
import { AnimatePresence, motion } from 'framer-motion'
import { ExternalLink, PlayCircle, Star, X } from 'lucide-react'
import { useEffect, useState } from 'react'
import type { AnimeItem, SiteMeta } from '../App'
import { sortSites } from '../utils/siteUtils'

interface DetailsModalProps {
    isOpen: boolean
    onClose: () => void
    anime: { title: string; info: any } | null
    items: AnimeItem[]
    siteMeta?: SiteMeta
}

export default function DetailsModal({ isOpen, onClose, anime, items, siteMeta }: DetailsModalProps) {
    const [displayAnime, setDisplayAnime] = useState(anime)

    useEffect(() => {
        if (anime) {
            setDisplayAnime(anime)
        }
    }, [anime])

    const { title, info } = displayAnime || { title: '', info: null }

    // Find the original item to get site links
    const originalItem = items.find(i => i.title === title)
    const sites = sortSites(originalItem?.sites || [])

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
                                className="fixed inset-0 bg-black/60 backdrop-blur-sm z-[100]"
                            />
                        </Dialog.Overlay>
                        <Dialog.Content asChild>
                            <div
                                className="fixed inset-0 z-[110] flex items-center justify-center p-4 sm:p-6 overflow-y-auto cursor-pointer"
                                onClick={onClose}
                            >
                                <motion.div
                                    layoutId={`card-${title}`}
                                    initial={{ opacity: 0 }}
                                    animate={{ opacity: 1 }}
                                    exit={{ opacity: 0 }}
                                    transition={{ duration: 0.35, ease: [0.4, 0, 0.2, 1] }}
                                    className="w-full max-w-4xl max-h-[85vh] bg-white dark:bg-gray-800 rounded-3xl shadow-2xl overflow-hidden relative outline-none cursor-default flex flex-col"
                                    onClick={(e) => e.stopPropagation()}
                                >
                                    <Dialog.Close asChild>
                                        <button className="absolute top-4 right-4 p-2 rounded-full bg-black/5 hover:bg-black/10 dark:bg-white/5 dark:hover:bg-white/10 text-gray-500 dark:text-gray-400 transition-colors z-10">
                                            <X size={20} />
                                        </button>
                                    </Dialog.Close>

                                    <div className="flex flex-col md:flex-row flex-1 overflow-hidden">
                                        {/* Image Section */}
                                        <motion.div
                                            layoutId={`image-${title}`}
                                            className="w-full md:w-2/5 aspect-[3/4] md:aspect-auto relative overflow-hidden bg-gray-100 dark:bg-gray-900 flex items-center justify-center"
                                        >
                                            {info?.coverImage?.extraLarge || info?.coverImage?.large ? (
                                                <>
                                                    {/* Blurred background for better aesthetics with different aspect ratios */}
                                                    <img
                                                        src={info.coverImage.extraLarge || info.coverImage.large}
                                                        className="absolute inset-0 w-full h-full object-cover blur-2xl opacity-20 dark:opacity-40 saturate-150"
                                                        aria-hidden="true"
                                                    />
                                                    <img
                                                        src={info.coverImage.extraLarge || info.coverImage.large}
                                                        alt={title}
                                                        className="relative z-10 max-w-full max-h-full object-contain drop-shadow-xl"
                                                    />
                                                </>
                                            ) : (
                                                <div className="w-full h-full flex items-center justify-center text-gray-400 italic">
                                                    No image available
                                                </div>
                                            )}
                                            <div className="absolute inset-x-0 bottom-0 h-32 bg-gradient-to-t from-white dark:from-gray-800 to-transparent md:hidden z-20" />
                                        </motion.div>

                                        {/* Content Section */}
                                        <motion.div
                                            layoutId={`content-${title}`}
                                            className="flex-1 p-6 md:p-8 space-y-6 overflow-y-auto custom-scrollbar"
                                        >
                                            <div>
                                                <Dialog.Title asChild>
                                                    <motion.h1
                                                        layoutId={`title-${title}`}
                                                        className="text-2xl md:text-3xl font-black text-gray-900 dark:text-white leading-tight mb-3"
                                                    >
                                                        {title}
                                                    </motion.h1>
                                                </Dialog.Title>
                                                <div className="flex flex-wrap gap-2">
                                                    {info?.averageScore && (
                                                        <div className="flex items-center gap-1.5 px-3 py-1 rounded-full bg-yellow-50 dark:bg-yellow-900/20 text-yellow-700 dark:text-yellow-400 border border-yellow-200/50 dark:border-yellow-700/30 text-sm font-bold">
                                                            <Star size={14} className="fill-current" />
                                                            {info.averageScore}%
                                                        </div>
                                                    )}
                                                    {info?.episodes && (
                                                        <div className="flex items-center gap-1.5 px-3 py-1 rounded-full bg-purple-50 dark:bg-purple-900/20 text-purple-700 dark:text-purple-400 border border-purple-200/50 dark:border-purple-700/30 text-sm font-bold">
                                                            <PlayCircle size={14} />
                                                            {info.episodes}话
                                                        </div>
                                                    )}
                                                    {info?.genres?.slice(0, 3).map((g: string) => (
                                                        <span key={g} className="px-3 py-1 rounded-full bg-gray-100 dark:bg-gray-700/50 text-gray-600 dark:text-gray-300 text-sm font-medium">
                                                            {g}
                                                        </span>
                                                    ))}
                                                </div>
                                            </div>

                                            {/* Multilingual Titles */}
                                            <div className="space-y-2 p-4 rounded-2xl bg-gray-50 dark:bg-gray-900/50 text-sm border border-gray-100 dark:border-gray-700/50">
                                                {info?.title?.native && (
                                                    <div className="flex gap-3">
                                                        <span className="text-gray-400 font-bold w-14 shrink-0">日本语</span>
                                                        <span className="text-gray-700 dark:text-gray-200">{info.title.native}</span>
                                                    </div>
                                                )}
                                                {/* Local data translations */}
                                                {originalItem?.titleTranslate && Object.entries(originalItem.titleTranslate).map(([lang, titles]) => (
                                                    <div key={lang} className="flex gap-3">
                                                        <span className="text-gray-400 font-bold w-14 shrink-0 uppercase">{lang}</span>
                                                        <span className="text-gray-700 dark:text-gray-200">{titles.join(' / ')}</span>
                                                    </div>
                                                ))}
                                                {info?.title?.romaji && (
                                                    <div className="flex gap-3">
                                                        <span className="text-gray-400 font-bold w-14 shrink-0">Romaji</span>
                                                        <span className="text-gray-700 dark:text-gray-200">{info.title.romaji}</span>
                                                    </div>
                                                )}
                                                {info?.title?.english && (
                                                    <div className="flex gap-3">
                                                        <span className="text-gray-400 font-bold w-14 shrink-0">English</span>
                                                        <span className="text-gray-700 dark:text-gray-200">{info.title.english}</span>
                                                    </div>
                                                )}
                                            </div>

                                            {/* Links Section */}
                                            {sites.length > 0 && (
                                                <div>
                                                    <h4 className="text-sm font-black uppercase tracking-wider text-gray-400 dark:text-gray-500 mb-2">Streaming</h4>
                                                    <div className="flex flex-wrap gap-2">
                                                        {sites.map((site, idx) => {
                                                            const meta = siteMeta?.[site.site]
                                                            const url = site.url || (meta?.urlTemplate?.replace('{{id}}', site.id || ''))
                                                            if (!url) return null

                                                            return (
                                                                <a
                                                                    key={`${site.site}-${idx}`}
                                                                    href={url}
                                                                    target="_blank"
                                                                    rel="noopener noreferrer"
                                                                    className="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400 text-sm font-bold border border-blue-100 dark:border-blue-800/50 hover:bg-blue-100 dark:hover:bg-blue-900/30 transition-colors"
                                                                >
                                                                    {meta?.title || site.site}
                                                                    <ExternalLink size={12} />
                                                                </a>
                                                            )
                                                        })}
                                                    </div>
                                                </div>
                                            )}

                                            {/* Description */}
                                            {info?.description && (
                                                <div>
                                                    <h4 className="text-sm font-black uppercase tracking-wider text-gray-400 dark:text-gray-500 mb-2">Introduction</h4>
                                                    <div
                                                        className="text-sm md:text-base text-gray-600 dark:text-gray-300 leading-relaxed prose prose-sm dark:prose-invert"
                                                        dangerouslySetInnerHTML={{ __html: info.description }}
                                                    />
                                                </div>
                                            )}

                                            {/* Episodes List */}
                                            {info?.episodesList?.length > 0 && (
                                                <div>
                                                    <h4 className="text-sm font-black uppercase tracking-wider text-gray-400 dark:text-gray-500 mb-3">Episodes</h4>
                                                    <div className="grid grid-cols-1 gap-2 max-h-80 overflow-y-auto pr-2 custom-scrollbar">
                                                        {info.episodesList.map((ep: any) => (
                                                            <div key={ep.number} className="flex flex-col gap-1.5 p-2.5 rounded-xl bg-gray-50 dark:bg-gray-900/40 border border-gray-100 dark:border-gray-700/50 text-sm group/ep hover:bg-white dark:hover:bg-gray-800 transition-colors">
                                                                <div className="flex gap-3 items-center">
                                                                    <span className="font-black text-blue-600 dark:text-blue-400 w-6 shrink-0 text-center">{ep.number}</span>
                                                                    <span className="text-gray-700 dark:text-gray-200 font-medium truncate group-hover/ep:text-blue-600 dark:group-hover/ep:text-blue-400 transition-colors">{ep.title || `Episode ${ep.number}`}</span>
                                                                    {ep.airDate && (
                                                                        <span className="text-[10px] text-gray-400 dark:text-gray-500 ml-auto self-center shrink-0">{ep.airDate}</span>
                                                                    )}
                                                                </div>
                                                                {ep.overview && (
                                                                    <p className="text-xs text-gray-500 dark:text-gray-400 pl-9 line-clamp-2 group-hover/ep:line-clamp-none transition-all">{ep.overview}</p>
                                                                )}
                                                            </div>
                                                        ))}
                                                    </div>
                                                </div>
                                            )}

                                            {/* Studio & Cast */}
                                            <div className="grid grid-cols-1 sm:grid-cols-2 gap-6">
                                                {info?.studios?.length > 0 && (
                                                    <div>
                                                        <h4 className="text-sm font-black uppercase tracking-wider text-gray-400 dark:text-gray-500 mb-2">Studio</h4>
                                                        <div className="text-sm text-gray-700 dark:text-gray-200 font-medium">
                                                            {info.studios.join(', ')}
                                                        </div>
                                                    </div>
                                                )}
                                                {info?.characters?.length > 0 && (
                                                    <div className="sm:col-span-2">
                                                        <h4 className="text-sm font-black uppercase tracking-wider text-gray-400 dark:text-gray-500 mb-3">Cast</h4>
                                                        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
                                                            {info.characters.slice(0, 6).map((char: any, idx: number) => (
                                                                <div key={idx} className="flex flex-col p-3 rounded-2xl bg-gray-50 dark:bg-gray-900/40 border border-gray-100 dark:border-gray-700/50 hover:border-blue-200 dark:hover:border-blue-900/50 transition-colors group">
                                                                    <div className="font-bold text-gray-800 dark:text-gray-100 group-hover:text-blue-600 dark:group-hover:text-blue-400 transition-colors truncate">
                                                                        {char.name}
                                                                    </div>
                                                                    {char.voiceActor && (
                                                                        <div className="flex items-center gap-1.5 mt-1">
                                                                            <span className="text-[9px] font-black px-1 py-0.5 rounded bg-gray-200 dark:bg-gray-800 text-gray-500 dark:text-gray-400 uppercase tracking-tighter">CV</span>
                                                                            <span className="text-xs text-gray-500 dark:text-gray-400 truncate">
                                                                                {char.voiceActor}
                                                                            </span>
                                                                        </div>
                                                                    )}
                                                                </div>
                                                            ))}
                                                        </div>
                                                    </div>
                                                )}
                                                {info?.staff?.length > 0 && (
                                                    <div className="sm:col-span-2">
                                                        <h4 className="text-sm font-black uppercase tracking-wider text-gray-400 dark:text-gray-500 mb-3">Staff</h4>
                                                        {(() => {
                                                            const grouped = info.staff.slice(0, 12).reduce((acc: any, member: any) => {
                                                                const dept = member.department || 'Other';
                                                                if (!acc[dept]) acc[dept] = [];
                                                                acc[dept].push(member);
                                                                return acc;
                                                            }, {});
                                                            return Object.entries(grouped).map(([dept, members]: [string, any]) => (
                                                                <div key={dept} className="mb-3">
                                                                    <div className="text-[10px] font-bold text-gray-400 dark:text-gray-500 uppercase tracking-wider mb-1.5">{dept}</div>
                                                                    <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-2">
                                                                        {members.map((member: any, idx: number) => (
                                                                            <div key={idx} className="flex flex-col p-2 rounded-lg bg-gray-50 dark:bg-gray-900/40 border border-gray-100 dark:border-gray-700/50 text-sm">
                                                                                <span className="text-[10px] font-semibold text-blue-600 dark:text-blue-400 uppercase tracking-tight truncate">{member.role}</span>
                                                                                <span className="text-gray-700 dark:text-gray-200 font-medium truncate mt-0.5">{member.name}</span>
                                                                            </div>
                                                                        ))}
                                                                    </div>
                                                                </div>
                                                            ));
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
    )
}
