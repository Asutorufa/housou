import * as Select from '@radix-ui/react-select'
import { clsx, type ClassValue } from 'clsx'
import { AnimatePresence, motion } from 'framer-motion'
import { ChevronDown, Search, X } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { twMerge } from 'tailwind-merge'
import type { Config } from '../types'
import WeekdaySelector from './WeekdaySelector'

function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs))
}

interface HeaderProps {
    config: Config | null
    selectedYear: string
    setSelectedYear: (year: string) => void
    selectedSeason: string
    setSelectedSeason: (season: string) => void
    selectedSite: string
    setSelectedSite: (site: string) => void
    searchQuery: string
    setSearchQuery: (query: string) => void
    activeTab: string
    onTabChange: (newTab: string) => void
}


export default function Header({
    config,
    selectedYear,
    setSelectedYear,
    selectedSeason,
    setSelectedSeason,
    selectedSite,
    setSelectedSite,
    searchQuery,
    setSearchQuery,
    activeTab,
    onTabChange
}: HeaderProps) {
    const [isSearchFocused, setIsSearchFocused] = useState(false)
    const [activeDropdown, setActiveDropdown] = useState<'year' | 'season' | 'site' | null>(null)
    const [isWeekdayExpanded, setIsWeekdayExpanded] = useState(false)
    const headerRef = useRef<HTMLDivElement>(null)

    // Close weekday expansion if search starts
    useEffect(() => {
        if (isSearchFocused) setIsWeekdayExpanded(false)
    }, [isSearchFocused])

    // Close on outside click
    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (isWeekdayExpanded && headerRef.current && !headerRef.current.contains(event.target as Node)) {
                setIsWeekdayExpanded(false)
            }
        }
        document.addEventListener('mousedown', handleClickOutside)
        return () => document.removeEventListener('mousedown', handleClickOutside)
    }, [isWeekdayExpanded])

    return (
        <header className="sticky top-0 z-[60] pt-2 px-1 pb-2 md:pt-2 md:pb-4 w-full flex justify-center pointer-events-none">
            <div
                ref={headerRef}
                className="w-full max-w-7xl grid grid-cols-[auto_1fr_auto] gap-2 md:gap-4 items-center pointer-events-auto px-1 min-h-[40px]"
            >
                {/* Left: Filters */}
                <div className="flex items-center h-full">
                    <AnimatePresence mode="popLayout" initial={false}>
                        {(!isSearchFocused && !isWeekdayExpanded) && (
                            <motion.div
                                key="filters"
                                initial={{ opacity: 0, x: -20 }}
                                animate={{ opacity: 1, x: 0 }}
                                exit={{ opacity: 0, x: -20 }}
                                transition={{ type: "spring", bounce: 0, duration: 0.3 }}
                                className="flex items-center h-10 bg-white/70 dark:bg-gray-800/70 backdrop-blur-xl rounded-full p-1 shadow-sm border border-white/20 dark:border-white/10"
                            >
                                {/* Year Select */}
                                <CustomSelect
                                    value={selectedYear}
                                    onValueChange={setSelectedYear}
                                    options={config?.years.map(y => ({ value: y.toString(), label: y.toString() })) || []}
                                    placeholder="年"
                                    open={activeDropdown === 'year'}
                                    onOpenChange={(open) => setActiveDropdown(open ? 'year' : null)}
                                />

                                <div className="w-px h-4 bg-gray-200 dark:bg-gray-700 mx-1" />

                                {/* Season Select */}
                                <CustomSelect
                                    value={selectedSeason}
                                    onValueChange={setSelectedSeason}
                                    options={[
                                        { value: 'all', label: '全て' },
                                        { value: 'Winter', label: '冬' },
                                        { value: 'Spring', label: '春' },
                                        { value: 'Summer', label: '夏' },
                                        { value: 'Autumn', label: '秋' },
                                    ]}
                                    placeholder="シーズン"
                                    open={activeDropdown === 'season'}
                                    onOpenChange={(open) => setActiveDropdown(open ? 'season' : null)}
                                />

                                <div className="w-px h-4 bg-gray-200 dark:bg-gray-700 mx-1" />

                                {/* Site Select */}
                                <CustomSelect
                                    value={selectedSite}
                                    onValueChange={setSelectedSite}
                                    options={[
                                        { value: 'all', label: '全て' },
                                        ...Object.entries(config?.site_meta || {}).map(([key, meta]) => ({
                                            value: key,
                                            label: meta?.title || key
                                        }))
                                    ]}
                                    placeholder="サイト"
                                    open={activeDropdown === 'site'}
                                    onOpenChange={(open) => setActiveDropdown(open ? 'site' : null)}
                                />
                            </motion.div>
                        )}
                    </AnimatePresence>
                </div>

                {/* Center: Weekday Navigation */}
                <div className={cn(
                    "flex justify-center items-center h-full",
                    isWeekdayExpanded ? "col-span-full" : (isSearchFocused ? "hidden" : "")
                )}>
                    <AnimatePresence>
                        {(!isSearchFocused) && (
                            <motion.div
                                initial={{ opacity: 0, scale: 0.95 }}
                                animate={{ opacity: 1, scale: 1 }}
                                exit={{ opacity: 0, scale: 0.95 }}
                                transition={{ type: "spring", bounce: 0, duration: 0.4 }}
                                layout
                                className="w-full flex justify-center"
                            >
                                <WeekdaySelector
                                    activeTab={activeTab}
                                    onTabChange={onTabChange}
                                    isExpanded={isWeekdayExpanded}
                                    onToggleExpand={setIsWeekdayExpanded}
                                />
                            </motion.div>
                        )}
                    </AnimatePresence>
                </div>

                {/* Right: Search Input */}
                <div className={cn(
                    "h-full flex items-center",
                    isSearchFocused ? "col-span-full" : "col-start-3",
                    isWeekdayExpanded ? "hidden" : "justify-end"
                )}>
                    <motion.div
                        layout
                        transition={{ type: "spring", bounce: 0, duration: 0.4 }}
                        onClick={() => {
                            if (!isSearchFocused) {
                                setIsSearchFocused(true)
                                setTimeout(() => document.querySelector<HTMLInputElement>('input[placeholder="検索..."]')?.focus(), 100)
                            }
                        }}
                        className={cn(
                            "relative shrink-0 group rounded-full backdrop-blur-xl flex items-center h-10 ml-auto",
                            isSearchFocused
                                ? "w-full bg-white dark:bg-gray-800 shadow-2xl ring-1 ring-black/5 dark:ring-white/10"
                                : "w-10 lg:w-auto bg-white/70 dark:bg-gray-800/70 shadow-sm border border-white/20 dark:border-white/10 hover:bg-white/90 dark:hover:bg-gray-800/90"
                        )}
                    >
                        <div className={cn(
                            "absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none transition-colors duration-200 z-10",
                            isSearchFocused ? "text-blue-500" : "text-gray-400"
                        )}>
                            <Search size={16} />
                        </div>

                        <motion.div
                            layout
                            className={cn(
                                "h-full flex items-center overflow-hidden",
                                isSearchFocused ? "w-full pl-9 pr-12" : "w-0 lg:w-32 xl:w-48 pl-9 lg:pr-4 opacity-0 lg:opacity-100"
                            )}
                        >
                            <input
                                type="text"
                                placeholder="検索..."
                                value={searchQuery}
                                onChange={(e) => setSearchQuery(e.target.value)}
                                onFocus={() => setIsSearchFocused(true)}
                                onBlur={() => setIsSearchFocused(false)}
                                className={cn(
                                    "w-full bg-transparent border-none outline-none text-sm text-gray-900 dark:text-gray-100 placeholder-gray-500 dark:placeholder-gray-400 min-w-[100px]",
                                    !isSearchFocused && !searchQuery && "pointer-events-none md:pointer-events-auto"
                                )}
                            />
                        </motion.div>

                        <AnimatePresence>
                            {searchQuery && (
                                <motion.button
                                    initial={{ opacity: 0, scale: 0.8 }}
                                    animate={{ opacity: 1, scale: 1 }}
                                    exit={{ opacity: 0, scale: 0.8 }}
                                    onClick={(e) => {
                                        e.stopPropagation()
                                        setSearchQuery('')
                                    }}
                                    className={cn(
                                        "absolute inset-y-0 right-2 flex items-center text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 z-10",
                                        !isSearchFocused && "hidden"
                                    )}
                                >
                                    <X size={14} />
                                </motion.button>
                            )}
                        </AnimatePresence>
                    </motion.div>
                </div>
            </div>
        </header>
    )
}

interface CustomSelectProps {
    value: string
    onValueChange: (value: string) => void
    options: { value: string; label: string }[]
    placeholder: string
    open: boolean
    onOpenChange: (open: boolean) => void
}

function CustomSelect({ value, onValueChange, options, placeholder, open, onOpenChange }: CustomSelectProps) {
    return (
        <Select.Root value={value} onValueChange={onValueChange} open={open} onOpenChange={onOpenChange}>
            <Select.Trigger asChild>
                <motion.button
                    className={cn(
                        "inline-flex items-center justify-between px-2 py-1 rounded-md text-sm font-medium cursor-pointer min-w-[60px] text-gray-700 dark:text-gray-200 transition-colors",
                        "outline-none hover:bg-black/5 dark:hover:bg-white/5",
                        open && "bg-black/5 dark:bg-white/5 text-blue-600 dark:text-blue-400"
                    )}
                >
                    <Select.Value placeholder={placeholder} />
                    <Select.Icon className="ml-1 text-gray-400">
                        <motion.div
                            animate={{ rotate: open ? 180 : 0 }}
                            transition={{ duration: 0.2 }}
                        >
                            <ChevronDown size={14} />
                        </motion.div>
                    </Select.Icon>
                </motion.button>
            </Select.Trigger>

            <Select.Portal>
                <Select.Content
                    sideOffset={4}
                    position="popper"
                    align="center"
                    className="select-content overflow-hidden bg-white/95 dark:bg-gray-800/95 backdrop-blur-xl rounded-2xl shadow-2xl shadow-black/10 dark:shadow-black/30 border border-gray-200/50 dark:border-gray-700/50 z-[60]"
                >
                    <Select.ScrollUpButton className="flex items-center justify-center h-8 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 cursor-default">
                        <ChevronDown className="rotate-180" size={16} />
                    </Select.ScrollUpButton>
                    <Select.Viewport className="p-1.5 max-h-[300px] overflow-y-auto">
                        {options.map((opt) => (
                            <Select.Item
                                key={opt.value}
                                value={opt.value}
                                className={cn(
                                    "relative flex items-center px-8 py-2.5 text-sm text-gray-700 dark:text-gray-300 rounded-2xl cursor-pointer outline-none select-none transition-all duration-150",
                                    "hover:bg-blue-50 dark:hover:bg-blue-900/30",
                                    "data-[highlighted]:bg-blue-50 data-[highlighted]:text-blue-700 dark:data-[highlighted]:bg-blue-900/30 dark:data-[highlighted]:text-blue-200",
                                    "data-[state=checked]:font-bold data-[state=checked]:text-blue-600 dark:data-[state=checked]:text-blue-400"
                                )}
                            >
                                <Select.ItemText>{opt.label}</Select.ItemText>
                                <Select.ItemIndicator className="absolute left-2.5 inline-flex items-center justify-center">
                                    <div className="w-1.5 h-1.5 rounded-full bg-blue-500" />
                                </Select.ItemIndicator>
                            </Select.Item>
                        ))}
                    </Select.Viewport>
                    <Select.ScrollDownButton className="flex items-center justify-center h-8 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 cursor-default">
                        <ChevronDown size={16} />
                    </Select.ScrollDownButton>
                </Select.Content>
            </Select.Portal>
        </Select.Root>
    )
}
