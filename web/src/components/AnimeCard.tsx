import { clsx, type ClassValue } from 'clsx'
import { motion } from 'framer-motion'
import { useEffect, useMemo, useRef, useState } from 'react'
import { twMerge } from 'tailwind-merge'
import type { AnimeItem, Site, SiteMeta, UnifiedMetadata } from '../types'

function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs))
}

interface AnimeCardProps {
    item: AnimeItem
    siteMeta?: SiteMeta
    selectedSite?: string
    onOpenModal: (title: string, info: UnifiedMetadata | null) => void
}

import { sortSites } from '../utils/siteUtils'

// ... existing imports ...

export default function AnimeCard({ item, siteMeta, selectedSite, onOpenModal }: AnimeCardProps) {
    // ... existing state ...
    const [metadata, setMetadata] = useState<UnifiedMetadata | null>(null)
    const [loading, setLoading] = useState(false)
    const cardRef = useRef<HTMLDivElement>(null)
    const loadedRef = useRef(false)

    // ... existing useEffect ... (keep it same)

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
        // ... (keep same)
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

        // Only show 'onair' sites on cards
        sites = sites.filter(s => {
            const type = s.type || siteMeta?.[s.site]?.type
            return type === 'onair'
        })

        return sortSites(sites, siteMeta)
    }, [item.sites, selectedSite, siteMeta])

    const coverUrl = metadata?.coverImage?.extraLarge || metadata?.coverImage?.large

    return (
        <motion.div
            ref={cardRef}
            layoutId={`card-${item.title}`}
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, margin: "-50px" }}
            whileHover={{
                y: -5,
                transition: { duration: 0.2, ease: 'easeOut' }
            }}
            className="group relative flex flex-col h-full rounded-xl overflow-hidden cursor-pointer bg-transparent"
        >
            {/* Image Container */}
            <motion.div
                layoutId={`image-${item.title}`}
                className={cn(
                    "aspect-[3/4] relative overflow-hidden bg-gray-100 dark:bg-gray-800 rounded-xl shadow-sm transition-shadow duration-300 group-hover:shadow-lg dark:shadow-black/40 mb-3",
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
                    <div className="w-full h-full flex items-center justify-center text-gray-300 dark:text-gray-600">
                        <span className="text-4xl">?</span>
                    </div>
                )}

                {/* Gradient Overlay */}
                <div className="absolute inset-0 bg-gradient-to-t from-black/90 via-black/40 to-transparent opacity-60 group-hover:opacity-80 transition-opacity" />

                {/* Overlay for Type/Score (Top) */}
                <div className="absolute top-0 left-0 p-2 w-full flex justify-between items-start pointer-events-none">
                    {item.type && (
                        <span className="px-2 py-1 rounded-md bg-black/40 backdrop-blur-md border border-white/10 text-[10px] font-bold text-white uppercase tracking-wider shadow-sm">
                            {{
                                'tv': 'TV',
                                'movie': 'Movie',
                                'ova': 'OVA',
                                'ona': 'ONA',
                                'special': 'Special',
                            }[item.type] || item.type}
                        </span>
                    )}
                    {!item.type && <span />} {/* Spacer if needed, or just nothing */}

                    {!!metadata?.averageScore && metadata.averageScore > 0 && (
                        <motion.span
                            layoutId={`score-${item.title}`}
                            className="flex items-center gap-1 px-1.5 py-0.5 rounded-md bg-yellow-500/90 text-[10px] font-bold text-white shadow-sm"
                        >
                            ⭐ {metadata.averageScore}
                        </motion.span>
                    )}
                </div>

                {/* Bottom Overlay Content: Title and Meta */}
                <div className="absolute bottom-0 left-0 w-full p-3 flex flex-col justify-end gap-1 pointer-events-none">
                    <motion.h3
                        layoutId={`title-${item.title}`}
                        className="font-bold text-white text-sm md:text-base leading-tight line-clamp-2 drop-shadow-md"
                    >
                        {item.title}
                    </motion.h3>

                    <div className="flex flex-wrap gap-1 items-center text-white/80 text-[10px]">
                        {metadata?.episodes && (
                            <motion.span
                                layoutId={`episodes-${item.title}`}
                                className="font-bold text-white mr-1"
                            >
                                {metadata.episodes}話
                            </motion.span>
                        )}
                        {metadata?.genres?.slice(0, 2).map(genre => (
                            <motion.span
                                key={genre}
                                layoutId={`genre-${item.title}-${genre}`}
                                className="px-1.5 py-0.5 rounded-md bg-white/20 backdrop-blur-sm border border-white/10 shadow-sm"
                            >
                                {genre}
                            </motion.span>
                        ))}
                    </div>
                </div>
            </motion.div>

            {/* Content Below Image: Just Links */}
            <div className="flex flex-col gap-1.5 px-0.5">
                {/* Links are now the only thing here */}

                {/* External Links */}
                <div className="flex flex-wrap gap-1.5 min-h-[1.5rem] content-start">
                    {sitesToShow.map((site: Site, idx: number) => {
                        const meta = siteMeta?.[site.site]
                        const url = site.url || (meta?.urlTemplate?.replace('{{id}}', site.id || ''))
                        if (!url) return null

                        return (
                            <motion.a
                                key={`${site.site}-${idx}`}
                                layoutId={`site-${item.title}-${site.site}`}
                                href={url}
                                target="_blank"
                                rel="noopener noreferrer"
                                onClick={(e) => e.stopPropagation()}
                                className="text-[10px] font-semibold text-gray-500 dark:text-gray-400 bg-gray-100 dark:bg-gray-800 hover:bg-blue-100 dark:hover:bg-blue-900/30 hover:text-blue-600 dark:hover:text-blue-400 px-2 py-1 rounded-md transition-colors"
                            >
                                {meta?.title || site.site}
                            </motion.a>
                        )
                    })}
                </div>
            </div>
        </motion.div >
    )
}
