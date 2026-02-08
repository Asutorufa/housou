import { clsx, type ClassValue } from 'clsx'
import { animate, motion, useMotionValue } from 'framer-motion'
import { useEffect, useRef, useState } from 'react'
import { twMerge } from 'tailwind-merge'

function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs))
}

const WEEKDAYS = ['日', '月', '火', '水', '木', '金', '土', '他']
const DRAG_THRESHOLD = 30 // Threshold in pixels to trigger a tab change

interface WeekdaySelectorProps {
    activeTab: string
    onTabChange: (newTab: string) => void
    isExpanded?: boolean
    onToggleExpand?: (expanded: boolean) => void
}

export default function WeekdaySelector({
    activeTab,
    onTabChange,
    isExpanded = false,
    onToggleExpand
}: WeekdaySelectorProps) {
    const activeIndex = parseInt(activeTab)
    const [isMobile, setIsMobile] = useState(false)
    const [isDragging, setIsDragging] = useState(false)
    const x = useMotionValue(0)
    const containerRef = useRef<HTMLDivElement>(null)

    // Check for mobile on mount and resize
    useEffect(() => {
        const checkMobile = () => setIsMobile(window.innerWidth < 640)
        checkMobile()
        window.addEventListener('resize', checkMobile)
        return () => window.removeEventListener('resize', checkMobile)
    }, [])

    const handleDragStart = () => setIsDragging(true)

    const handleDragEnd = (_: any, info: any) => {
        setIsDragging(false)
        const offset = info.offset.x
        if (Math.abs(offset) > DRAG_THRESHOLD) {
            const direction = offset > 0 ? -1 : 1
            const nextIndex = (activeIndex + direction + WEEKDAYS.length) % WEEKDAYS.length
            onTabChange(nextIndex.toString())
        }
        // Snap back to 0
        animate(x, 0, { type: 'spring', stiffness: 300, damping: 30 })
    }

    const handleToggle = (e: React.MouseEvent) => {
        if (!isMobile || isDragging) return
        e.stopPropagation()
        onToggleExpand?.(!isExpanded)
    }

    const handleSelect = (index: number, e: React.MouseEvent) => {
        e.stopPropagation()
        onTabChange(index.toString())
        if (isMobile && isExpanded) {
            onToggleExpand?.(false)
        }
    }

    return (
        <div
            ref={containerRef}
            className={cn(
                "relative flex items-center justify-center h-10",
                isMobile ? (isExpanded ? "w-full" : "w-24") : "w-auto"
            )}
        >
            <motion.div
                layout
                transition={{ type: "spring", bounce: 0, duration: 0.4 }}
                drag={isMobile && !isExpanded ? "x" : false}
                dragConstraints={{ left: 0, right: 0 }}
                style={{ x }}
                onDragStart={handleDragStart}
                onDragEnd={handleDragEnd}
                onClick={handleToggle}
                whileDrag={{ scale: 0.98 }}
                className={cn(
                    "flex items-center bg-white/70 dark:bg-gray-800/70 backdrop-blur-xl rounded-full p-1 shadow-sm border border-white/20 dark:border-white/10",
                    isMobile && !isExpanded ? "cursor-grab active:cursor-grabbing px-6 py-1.5 min-w-[4rem] justify-center" : "px-0.5",
                    isMobile && isExpanded ? "w-full justify-between" : ""
                )}
            >
                {isMobile && !isExpanded ? (
                    <motion.div
                        key={activeIndex}
                        initial={{ opacity: 0, y: 5 }}
                        animate={{ opacity: 1, y: 0 }}
                        exit={{ opacity: 0, y: -5 }}
                        transition={{ type: "spring", bounce: 0, duration: 0.3 }}
                        className="text-xs font-black text-blue-600 dark:text-blue-400 select-none"
                    >
                        {WEEKDAYS[activeIndex]}
                    </motion.div>
                ) : (
                    WEEKDAYS.map((label, index) => (
                        <button
                            key={index}
                            onClick={(e) => handleSelect(index, e)}
                            className={cn(
                                "relative px-2 sm:px-4 py-1.5 rounded-full text-xs font-bold transition-all outline-none select-none",
                                activeTab === index.toString()
                                    ? "text-blue-600 dark:text-blue-400"
                                    : "text-gray-500 hover:text-gray-900 dark:text-gray-400 dark:hover:text-white"
                            )}
                        >
                            <span className="relative z-10">{label}</span>
                            {activeTab === index.toString() && (
                                <motion.div
                                    layoutId={isMobile ? "mobileTabIndicator" : "headerTab"}
                                    className="absolute inset-0 bg-white dark:bg-gray-800 rounded-full shadow-md border border-black/5 dark:border-white/5"
                                    transition={{ type: "spring", bounce: 0, duration: 0.4 }}
                                />
                            )}
                        </button>
                    ))
                )}
            </motion.div>

            {/* Visual indicator for drag (subtle arrows or dots if needed) */}
            {isMobile && (
                <div className="absolute -bottom-4 flex gap-1 opacity-20 group-hover:opacity-100 transition-opacity">
                    {WEEKDAYS.map((_, i) => (
                        <div
                            key={i}
                            className={cn(
                                "w-1 h-1 rounded-full transition-all",
                                activeIndex === i ? "bg-blue-600 w-2" : "bg-gray-400"
                            )}
                        />
                    ))}
                </div>
            )}
        </div>
    )
}
