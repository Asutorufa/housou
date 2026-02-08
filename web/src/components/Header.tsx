import * as Select from '@radix-ui/react-select'
import { clsx, type ClassValue } from 'clsx'
import { AnimatePresence, motion } from 'framer-motion'
import { ChevronDown, Search, X } from 'lucide-react'
import { useState } from 'react'
import { twMerge } from 'tailwind-merge'
import type { Config } from '../types'

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
}: HeaderProps) {
    const [isSearchFocused, setIsSearchFocused] = useState(false)

    return (
        <header className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-md border border-gray-200/50 dark:border-gray-700/50 shadow-md sticky top-2 z-50 p-2 mx-2 md:mx-4 rounded-4xl transition-all group/header">
            <div className="max-w-7xl mx-auto flex gap-3 items-center">
                <AnimatePresence mode="popLayout" initial={false}>
                    {!isSearchFocused && (
                        <motion.div
                            key="filters"
                            initial={{ opacity: 0, x: -20 }}
                            animate={{ opacity: 1, x: 0 }}
                            exit={{ opacity: 0, x: -20 }}
                            transition={{ duration: 0.2 }}
                            className="flex overflow-x-auto gap-2 scrollbar-hide flex-1 items-center md:flex"
                        >
                            {/* Year Select */}
                            <CustomSelect
                                value={selectedYear}
                                onValueChange={setSelectedYear}
                                options={config?.years.map(y => ({ value: y.toString(), label: y.toString() })) || []}
                                placeholder="年"
                            />

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
                            />

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
                            />
                        </motion.div>
                    )}
                </AnimatePresence>

                {/* Search Input */}
                <motion.div
                    layout
                    transition={{ type: "spring", stiffness: 300, damping: 30 }}
                    className={cn(
                        "relative shrink-0 group overflow-hidden rounded-full bg-gray-100/50 dark:bg-gray-700/50 border-2 border-transparent hover:border-blue-500/50",
                        isSearchFocused ? "w-full border-blue-500 ring-2 ring-blue-500/20" : "w-10 md:w-64"
                    )}
                >
                    <div className={cn(
                        "absolute inset-0 md:inset-y-0 md:left-3 md:right-auto md:w-auto flex items-center justify-center md:justify-start pointer-events-none text-gray-400 transition-all duration-300",
                        isSearchFocused && "text-blue-500 inset-y-0 left-3 right-auto w-auto justify-start"
                    )}>
                        <Search size={16} />
                    </div>
                    <input
                        type="text"
                        placeholder="検索..."
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        onFocus={() => setIsSearchFocused(true)}
                        onBlur={() => setIsSearchFocused(false)}
                        className="w-full pl-9 pr-8 py-1.5 bg-transparent border-none outline-none text-sm text-gray-900 dark:text-gray-100 placeholder-transparent focus:placeholder-gray-500 dark:focus:placeholder-gray-400 md:placeholder-gray-500 md:dark:placeholder-gray-400"
                    />
                    <AnimatePresence>
                        {searchQuery && (
                            <motion.button
                                initial={{ opacity: 0, scale: 0.8 }}
                                animate={{ opacity: 1, scale: 1 }}
                                exit={{ opacity: 0, scale: 0.8 }}
                                onClick={() => setSearchQuery('')}
                                className={cn(
                                    "absolute inset-y-0 right-2 hidden md:flex items-center text-gray-400 hover:text-gray-600 dark:hover:text-gray-200",
                                    isSearchFocused && "flex"
                                )}
                            >
                                <X size={14} />
                            </motion.button>
                        )}
                    </AnimatePresence>
                </motion.div>
            </div>
        </header>
    )
}

interface CustomSelectProps {
    value: string
    onValueChange: (value: string) => void
    options: { value: string; label: string }[]
    placeholder: string
}

function CustomSelect({ value, onValueChange, options, placeholder }: CustomSelectProps) {
    const [isOpen, setIsOpen] = useState(false)

    return (
        <Select.Root value={value} onValueChange={onValueChange} open={isOpen} onOpenChange={setIsOpen}>
            <Select.Trigger asChild>
                <motion.button
                    className={cn(
                        "inline-flex items-center justify-between px-3 py-1.5 rounded-full bg-gray-100 dark:bg-gray-700 border-2 text-xs sm:text-sm font-medium cursor-pointer min-w-[80px] sm:min-w-[100px] text-gray-900 dark:text-gray-100 whitespace-nowrap transition-all duration-200",
                        "outline-none ring-0 focus:outline-none focus:ring-0 focus-visible:outline-none focus-visible:ring-0",
                        "active:brightness-95",
                        isOpen
                            ? "border-blue-500"
                            : "border-transparent hover:border-blue-500"
                    )}
                >
                    <Select.Value placeholder={placeholder} />
                    <Select.Icon className="ml-2 text-gray-400">
                        <motion.div
                            animate={{ rotate: isOpen ? 180 : 0 }}
                            transition={{ duration: 0.2 }}
                        >
                            <ChevronDown size={16} />
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
