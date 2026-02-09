import * as Tabs from '@radix-ui/react-tabs'
import { clsx, type ClassValue } from 'clsx'
import { AnimatePresence, motion } from 'framer-motion'
import { useEffect, useMemo, useState } from 'react'
import { twMerge } from 'tailwind-merge'
import type { AnimeItem, SiteMeta, UnifiedMetadata } from '../types'
import AnimeCard from './AnimeCard'

function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs))
}

const WEEKDAYS = ['日', '月', '火', '水', '木', '金', '土', '他']

interface TabbedGridProps {
    items: AnimeItem[]
    siteMeta?: SiteMeta
    selectedSite?: string
    onOpenModal: (title: string, info: UnifiedMetadata | null) => void
}

function useColumnCount() {
    const [columns, setColumns] = useState(1)

    useEffect(() => {
        const updateColumns = () => {
            const width = window.innerWidth
            if (width >= 1280) setColumns(4)
            else if (width >= 1024) setColumns(3)
            else setColumns(2)
        }
        updateColumns()
        window.addEventListener('resize', updateColumns)
        return () => window.removeEventListener('resize', updateColumns)
    }, [])

    return columns
}

const containerVariants = {
    hidden: (direction: number) => ({
        x: direction > 0 ? '100%' : '-100%',
    }),
    show: {
        x: 0,
        transition: {
            x: { type: "tween" as const, ease: "easeInOut" as const, duration: 0.5 },
        }
    },
    pageExit: (direction: number) => ({
        x: direction > 0 ? '-100%' : '100%',
        transition: {
            x: { type: "tween" as const, ease: "easeInOut" as const, duration: 0.5 },
        }
    })
}

export default function TabbedGrid({ items, siteMeta, selectedSite, onOpenModal }: TabbedGridProps) {
    const currentDay = new Date().getDay().toString()
    const [activeTab, setActiveTab] = useState(currentDay)
    const [direction, setDirection] = useState(0)
    const columnCount = useColumnCount()

    const handleTabChange = (newTab: string) => {
        const prevIndex = parseInt(activeTab)
        const nextIndex = parseInt(newTab)
        setDirection(nextIndex > prevIndex ? 1 : -1)
        setActiveTab(newTab)
    }

    const groupedItems = useMemo(() => {
        const groups: { item: AnimeItem; time: number }[][] = Array.from({ length: 8 }, () => [])
        items.forEach(item => {
            let dayIndex = 7
            let time = 0
            if (item.begin) {
                const date = new Date(item.begin)
                const t = date.getTime()
                if (!isNaN(t)) {
                    dayIndex = date.getDay()
                    time = t
                }
            }
            groups[dayIndex].push({ item, time })
        })

        return groups.map(group => {
            group.sort((a, b) => a.time - b.time)
            return group.map(g => g.item)
        })
    }, [items])

    const dayIndex = parseInt(activeTab)
    const dayItems = groupedItems[dayIndex]

    // Distribute items into columns for horizontal masonry feel
    const columns: AnimeItem[][] = Array.from({ length: columnCount }, () => [])
    dayItems.forEach((item, idx) => {
        columns[idx % columnCount].push(item)
    })

    return (
        <Tabs.Root value={activeTab} onValueChange={handleTabChange} className="flex flex-col gap-6">
            <Tabs.List className="flex gap-2 overflow-x-auto pb-6 scrollbar-hide no-scrollbar ring-offset-background">
                {WEEKDAYS.map((label, index) => (
                    <Tabs.Trigger
                        key={index}
                        value={index.toString()}
                        className={cn(
                            "relative px-5 py-2.5 rounded-full text-sm font-bold whitespace-nowrap transition-colors outline-none",
                            "bg-white dark:bg-gray-800 text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200",
                            "data-[state=active]:text-white z-10"
                        )}
                    >
                        {activeTab === index.toString() && (
                            <motion.div
                                layoutId="activeTab"
                                className="absolute inset-0 bg-blue-500 rounded-full shadow-lg shadow-blue-500/30 -z-10"
                                transition={{ type: "spring", bounce: 0.2, duration: 0.6 }}
                            />
                        )}
                        {label}
                    </Tabs.Trigger>
                ))}
            </Tabs.List>

            <div className="relative overflow-hidden -mx-2" style={{ display: 'grid' }}>
                <AnimatePresence mode="sync" initial={false} custom={direction}>
                    <motion.div
                        key={activeTab}
                        custom={direction}
                        variants={containerVariants}
                        initial="hidden"
                        animate="show"
                        exit="pageExit"
                        className="px-2 pb-12"
                        style={{ gridArea: '1 / 1' }}
                    >
                        {dayItems.length > 0 ? (
                            <div className="flex gap-1 sm:gap-2 md:gap-4 items-start">
                                {columns.map((column, colIdx) => (
                                    <div key={colIdx} className="flex-1 flex flex-col gap-1 sm:gap-2 md:gap-4">
                                        <AnimatePresence mode="popLayout" initial={false}>
                                            {column.map((item) => (
                                                <motion.div
                                                    key={item.title}
                                                    layout="position"
                                                    className="p-2"
                                                >
                                                    <AnimeCard
                                                        item={item}
                                                        siteMeta={siteMeta}
                                                        selectedSite={selectedSite}
                                                        onOpenModal={onOpenModal}
                                                    />
                                                </motion.div>
                                            ))}
                                        </AnimatePresence>
                                    </div>
                                ))}
                            </div>
                        ) : (
                            <motion.div
                                initial={{ opacity: 0 }}
                                animate={{ opacity: 1 }}
                                className="text-center py-20 text-gray-500 dark:text-gray-400"
                            >
                                この日の放送はありません
                            </motion.div>
                        )}
                    </motion.div>
                </AnimatePresence>
            </div>
        </Tabs.Root>
    )
}
