import * as Tabs from '@radix-ui/react-tabs'
import { clsx, type ClassValue } from 'clsx'
import { AnimatePresence, motion } from 'framer-motion'
import { useEffect, useMemo, useState } from 'react'
import { twMerge } from 'tailwind-merge'
import type { AnimeItem, SiteMeta } from '../App'
import AnimeCard from './AnimeCard'

function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs))
}

const WEEKDAYS = ['日', '月', '火', '水', '木', '金', '土', '他']

interface TabbedGridProps {
    items: AnimeItem[]
    siteMeta?: SiteMeta
    selectedSite?: string
    onOpenModal: (title: string, info: any) => void
}

function useColumnCount() {
    const [columns, setColumns] = useState(1)

    useEffect(() => {
        const updateColumns = () => {
            const width = window.innerWidth
            if (width >= 1280) setColumns(4)
            else if (width >= 1024) setColumns(3)
            else if (width >= 640) setColumns(2)
            else setColumns(1)
        }
        updateColumns()
        window.addEventListener('resize', updateColumns)
        return () => window.removeEventListener('resize', updateColumns)
    }, [])

    return columns
}

export default function TabbedGrid({ items, siteMeta, selectedSite, onOpenModal }: TabbedGridProps) {
    const currentDay = new Date().getDay().toString()
    const [activeTab, setActiveTab] = useState(currentDay)
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
        <Tabs.Root value={activeTab} onValueChange={setActiveTab} className="flex flex-col gap-6">
            <Tabs.List className="flex gap-2 overflow-x-auto pb-4 scrollbar-hide no-scrollbar ring-offset-background">
                {WEEKDAYS.map((label, index) => (
                    <Tabs.Trigger
                        key={index}
                        value={index.toString()}
                        className={cn(
                            "px-5 py-2.5 rounded-full text-sm font-bold whitespace-nowrap transition-all outline-none",
                            "bg-white dark:bg-gray-800 text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700",
                            "data-[state=active]:bg-blue-500 data-[state=active]:text-white data-[state=active]:shadow-lg data-[state=active]:shadow-blue-500/30"
                        )}
                    >
                        {label}
                    </Tabs.Trigger>
                ))}
            </Tabs.List>

            <div className="relative min-h-[400px]">
                <AnimatePresence mode="wait">
                    <motion.div
                        key={activeTab}
                        initial={{ opacity: 0, x: 10 }}
                        animate={{ opacity: 1, x: 0 }}
                        exit={{ opacity: 0, x: -10 }}
                        transition={{ duration: 0.2, ease: "easeInOut" }}
                        className="outline-none"
                    >
                        {dayItems.length > 0 ? (
                            <div className="flex gap-6 items-start">
                                {columns.map((column, colIdx) => (
                                    <div key={colIdx} className="flex-1 flex flex-col gap-6">
                                        <AnimatePresence mode="popLayout" initial={false}>
                                            {column.map((item) => (
                                                <motion.div
                                                    key={item.title}
                                                    layout
                                                    initial={{ opacity: 0, scale: 0.9 }}
                                                    animate={{ opacity: 1, scale: 1 }}
                                                    exit={{ opacity: 0, scale: 0.9 }}
                                                    transition={{
                                                        opacity: { duration: 0.2 },
                                                        layout: { type: "spring", stiffness: 300, damping: 30 }
                                                    }}
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
                            <div className="text-center py-20 text-gray-500 dark:text-gray-400">
                                この日の放送はありません
                            </div>
                        )}
                    </motion.div>
                </AnimatePresence>
            </div>
        </Tabs.Root>
    )
}
