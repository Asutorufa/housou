import { clsx, type ClassValue } from 'clsx'
import { motion } from 'framer-motion'
import { useEffect, useMemo, useRef, useState } from 'react'
import { twMerge } from 'tailwind-merge'
import type { AnimeItem, SiteMeta } from '../App'
import { sortSites } from '../utils/siteUtils'

function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs))
}

interface AnimeCardProps {
    item: AnimeItem
    siteMeta?: SiteMeta
    selectedSite?: string
    onOpenModal: (title: string, info: any) => void
}

export default function AnimeCard({ item, siteMeta, selectedSite, onOpenModal }: AnimeCardProps) {
    const [metadata, setMetadata] = useState<any>(null)
    const [loading, setLoading] = useState(false)
    const cardRef = useRef<HTMLDivElement>(null)
    const loadedRef = useRef(false)

    useEffect(() => {
        const observer = new IntersectionObserver(
            (entries) => {
                if (entries[0].isIntersecting && !loadedRef.current) {
                    loadedRef.current = true
                    fetchMetadata()
                    observer.disconnect()
                }
            },
            { rootMargin: '200px' }
        )

        if (cardRef.current) {
            observer.observe(cardRef.current)
        }

        return () => observer.disconnect()
    }, [item.title])

    async function fetchMetadata() {
        setLoading(true)
        try {
            const tmdbSite = item.sites?.find(s => s.site === 'tmdb')
            let url = `/api/metadata?title=${encodeURIComponent(item.title)}`

            if (tmdbSite?.id) {
                url += `&tmdb_id=${encodeURIComponent(tmdbSite.id)}`
            }

            if (item.begin) {
                url += `&begin=${encodeURIComponent(item.begin)}`
            }

            const response = await fetch(url)
            if (!response.ok) throw new Error('Metadata fetch failed')
            const data = await response.json()
            setMetadata(data || null)
        } catch (err) {
            console.error('Metadata error:', err)
        } finally {
            setLoading(false)
        }
    }

    const sitesToShow = useMemo(() => {
        let sites = item.sites || []
        if (selectedSite && selectedSite !== 'all') {
            sites = sites.filter(s => s.site === selectedSite)
        }

        return sortSites(sites)
    }, [item.sites, selectedSite])

    const coverUrl = metadata?.coverImage?.extraLarge || metadata?.coverImage?.large

    return (
        <motion.div
            ref={cardRef}
            layoutId={`card-${item.title}`}
            whileHover={{ y: -4, transition: { duration: 0.2, ease: 'easeOut' } }}
            className="bg-white dark:bg-gray-800 rounded-2xl shadow-md overflow-hidden hover:shadow-xl transition-shadow flex flex-col h-full ring-1 ring-black/5 dark:ring-white/5"
        >
            {/* Cover Image */}
            <motion.div
                layoutId={`image-${item.title}`}
                className={cn(
                    "aspect-[3/4] relative overflow-hidden bg-gray-200 dark:bg-gray-700 cursor-pointer group",
                    !coverUrl && loading && "animate-pulse"
                )}
                onClick={() => onOpenModal(item.title, metadata)}
            >
                {coverUrl ? (
                    <img
                        src={coverUrl}
                        alt={item.title}
                        className="w-full h-full object-cover transition-transform duration-500 group-hover:scale-105"
                        loading="lazy"
                    />
                ) : !loading && (
                    <div className="w-full h-full flex items-center justify-center text-gray-400 text-sm italic">
                        No image
                    </div>
                )}

                {/* Hover Overlay */}
                <div className="absolute inset-0 bg-black/0 group-hover:bg-black/20 transition-colors" />
            </motion.div>

            {/* Content */}
            <motion.div
                layoutId={`content-${item.title}`}
                className="p-4 flex flex-col gap-3 flex-1"
            >
                <motion.h3
                    layoutId={`title-${item.title}`}
                    className="text-base font-bold text-gray-900 dark:text-gray-100 line-clamp-2 leading-tight"
                >
                    {item.title}
                </motion.h3>

                {/* Tags */}
                <div className="flex flex-wrap gap-1.5 items-center">
                    <span className="px-2 py-0.5 rounded-full text-[10px] font-bold bg-gray-100 dark:bg-gray-700/50 text-gray-700 dark:text-gray-300 ring-1 ring-black/5 dark:ring-white/10 uppercase">
                        {item.type}
                    </span>
                    {metadata?.averageScore && (
                        <span className="px-2 py-0.5 rounded-full text-[10px] font-bold bg-yellow-100 dark:bg-yellow-900/30 text-yellow-700 dark:text-yellow-300 ring-1 ring-yellow-500/10">
                            ⭐ {metadata.averageScore}%
                        </span>
                    )}
                    {metadata?.episodes && (
                        <span className="px-2 py-0.5 rounded-full text-[10px] font-bold bg-purple-100 dark:bg-purple-900/30 text-purple-700 dark:text-purple-300 ring-1 ring-purple-500/10">
                            {metadata.episodes}话
                        </span>
                    )}
                    {metadata?.genres?.slice(0, 2).map((genre: string) => (
                        <span key={genre} className="px-2 py-0.5 rounded-full text-[10px] font-bold bg-teal-100 dark:bg-teal-900/30 text-teal-700 dark:text-teal-300 ring-1 ring-teal-500/10">
                            {genre}
                        </span>
                    ))}
                    {item.begin && (
                        <span className="px-2 py-0.5 rounded-full text-[10px] font-bold bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 ring-1 ring-blue-500/10">
                            {new Date(item.begin).toISOString().split('T')[0]}
                        </span>
                    )}
                </div>

                {/* Links */}
                {sitesToShow.length > 0 && (
                    <div className="flex flex-wrap gap-1.5 pt-3 mt-auto border-t border-gray-100 dark:border-gray-700/50">
                        {sitesToShow.map((site, idx) => {
                            const meta = siteMeta?.[site.site]
                            const url = site.url || (meta?.urlTemplate?.replace('{{id}}', site.id || ''))
                            if (!url) return null

                            return (
                                <a
                                    key={`${site.site}-${idx}`}
                                    href={url}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    className="text-[11px] font-semibold text-blue-600 dark:text-blue-400 hover:text-blue-700 dark:hover:text-blue-300 hover:bg-blue-50 dark:hover:bg-blue-900/40 px-2 py-1 rounded-md transition-all active:scale-95"
                                >
                                    {meta?.title || site.site}
                                </a>
                            )
                        })}
                    </div>
                )}
            </motion.div>
        </motion.div>
    )
}
