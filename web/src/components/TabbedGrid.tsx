import * as Tabs from '@radix-ui/react-tabs'
import { AnimatePresence, motion } from 'framer-motion'
import { useEffect, useMemo, useState } from 'react'
import type { AnimeItem, SiteMeta, UnifiedMetadata } from '../types'
import AnimeCard from './AnimeCard'


interface TabbedGridProps {
    items: AnimeItem[]
    siteMeta?: SiteMeta
    selectedSite?: string
    onOpenModal: (title: string, info: UnifiedMetadata | null) => void
    activeTab: string
    onTabChange: (newTab: string) => void
    direction: number
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
        x: direction > 0 ? '110%' : '-110%',
    }),
    show: {
        x: 0,
        transition: {
            x: { type: "tween" as const, duration: 0.5, ease: [0.4, 0, 0.2, 1] as any },
        }
    },
    exit: (direction: number) => ({
        x: direction > 0 ? '-110%' : '110%',
        transition: {
            x: { type: "tween" as const, duration: 0.5, ease: [0.4, 0, 0.2, 1] as any },
        }
    })
}

export default function TabbedGrid({
    items,
    siteMeta,
    selectedSite,
    onOpenModal,
    activeTab,
    onTabChange,
    direction
}: TabbedGridProps) {
    const columnCount = useColumnCount()

    const groupedItems = useMemo(() => {
        const groups: AnimeItem[][] = Array.from({ length: 8 }, () => [])
        items.forEach(item => {
            let dayIndex = 7
            if (item.begin) {
                const date = new Date(item.begin)
                if (!isNaN(date.getTime())) {
                    dayIndex = date.getDay()
                }
            }
            groups[dayIndex].push(item)
        })

        groups.forEach(group => {
            group.sort((a, b) => {
                const timeA = a.begin ? new Date(a.begin).getTime() : 0
                const timeB = b.begin ? new Date(b.begin).getTime() : 0
                return timeA - timeB
            })
        })

        return groups
    }, [items])

    const dayIndex = parseInt(activeTab)
    const dayItems = groupedItems[dayIndex]

    // Distribute items into columns for horizontal masonry feel
    const columns: AnimeItem[][] = Array.from({ length: columnCount }, () => [])
    dayItems.forEach((item, idx) => {
        columns[idx % columnCount].push(item)
    })
    return (
        <Tabs.Root value={activeTab} onValueChange={onTabChange} className="flex flex-col gap-6 md:gap-8">
            <div className="relative min-h-[50vh] overflow-hidden" style={{ display: 'grid' }}>
                <AnimatePresence mode="popLayout" initial={false} custom={direction}>
                    <motion.div
                        key={activeTab}
                        custom={direction}
                        variants={containerVariants}
                        initial="hidden"
                        animate="show"
                        exit="exit"
                        className="px-1 col-start-1 row-start-1 w-full will-change-transform"
                    >
                        {dayItems.length > 0 ? (
                            <div className="flex gap-4 md:gap-6 items-start">
                                {columns.map((column, colIdx) => (
                                    <div key={colIdx} className="flex-1 flex flex-col gap-4 md:gap-6">
                                        {column.map((item) => (
                                            <AnimeCard
                                                key={item.title}
                                                item={item}
                                                siteMeta={siteMeta}
                                                selectedSite={selectedSite}
                                                onOpenModal={onOpenModal}
                                            />
                                        ))}
                                    </div>
                                ))}
                            </div>
                        ) : (
                            <motion.div
                                initial={{ opacity: 0 }}
                                animate={{ opacity: 1 }}
                                className="flex flex-col items-center justify-center py-32 text-gray-400 dark:text-gray-600"
                            >
                                <div className="text-lg font-medium">No Broadcasts</div>
                                <div className="text-sm">この日の放送はありません</div>
                            </motion.div>
                        )}
                    </motion.div>
                </AnimatePresence>
            </div>
        </Tabs.Root >
    )
}
