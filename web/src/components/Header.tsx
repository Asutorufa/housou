import * as Select from '@radix-ui/react-select'
import { clsx, type ClassValue } from 'clsx'
import { ChevronDown, Search, X } from 'lucide-react'
import { twMerge } from 'tailwind-merge'
import type { Config } from '../App'

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
    return (
        <header className="bg-white dark:bg-gray-800 shadow-sm sticky top-0 z-50 p-4 transition-colors">
            <div className="max-w-7xl mx-auto flex flex-col md:flex-row gap-4 items-center">
                <div className="flex flex-wrap gap-2 flex-1 w-full md:w-auto">
                    {/* Year Select */}
                    <CustomSelect
                        value={selectedYear}
                        onValueChange={setSelectedYear}
                        options={config?.years.map(y => ({ value: y.toString(), label: y.toString() })) || []}
                        placeholder="年份"
                    />

                    {/* Season Select */}
                    <CustomSelect
                        value={selectedSeason}
                        onValueChange={setSelectedSeason}
                        options={[
                            { value: 'all', label: 'すべての季節' },
                            { value: 'Winter', label: '冬' },
                            { value: 'Spring', label: '春' },
                            { value: 'Summer', label: '夏' },
                            { value: 'Autumn', label: '秋' },
                        ]}
                        placeholder="季节"
                    />

                    {/* Site Select */}
                    <CustomSelect
                        value={selectedSite}
                        onValueChange={setSelectedSite}
                        options={[
                            { value: 'all', label: 'すべてのサイト' },
                            ...Object.entries(config?.site_meta || {}).map(([key, meta]) => ({
                                value: key,
                                label: meta.title
                            }))
                        ]}
                        placeholder="站点"
                    />
                </div>

                {/* Search Input */}
                <div className="relative w-full md:max-w-xs group">
                    <div className="absolute inset-y-0 left-3 flex items-center pointer-events-none text-gray-400 group-focus-within:text-blue-500 transition-colors">
                        <Search size={18} />
                    </div>
                    <input
                        type="text"
                        placeholder="検索..."
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        className="w-full pl-10 pr-10 py-2 rounded-full bg-gray-100 dark:bg-gray-700 border-2 border-transparent hover:border-blue-500/50 focus:border-blue-500 focus:ring-2 focus:ring-blue-500/20 outline-none text-sm transition-all text-gray-900 dark:text-gray-100 placeholder-gray-500 dark:placeholder-gray-400"
                    />
                    {searchQuery && (
                        <button
                            onClick={() => setSearchQuery('')}
                            className="absolute inset-y-0 right-3 flex items-center text-gray-400 hover:text-gray-600 dark:hover:text-gray-200"
                        >
                            <X size={16} />
                        </button>
                    )}
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
}

function CustomSelect({ value, onValueChange, options, placeholder }: CustomSelectProps) {
    return (
        <Select.Root value={value} onValueChange={onValueChange}>
            <Select.Trigger
                className="inline-flex items-center justify-between px-4 py-2 rounded-full bg-gray-100 dark:bg-gray-700 border-2 border-transparent hover:border-blue-500 focus:border-blue-500 focus:ring-2 focus:ring-blue-500/20 outline-none text-sm font-medium transition-all cursor-pointer min-w-[120px] text-gray-900 dark:text-gray-100"
            >
                <Select.Value placeholder={placeholder} />
                <Select.Icon className="ml-2 text-gray-400">
                    <ChevronDown size={16} />
                </Select.Icon>
            </Select.Trigger>

            <Select.Portal>
                <Select.Content className="overflow-hidden bg-white dark:bg-gray-800 rounded-xl shadow-xl border border-gray-200 dark:border-gray-700 z-[60] animate-in fade-in zoom-in duration-200">
                    <Select.ScrollUpButton className="flex items-center justify-center h-8 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 cursor-default">
                        <ChevronDown className="rotate-180" size={16} />
                    </Select.ScrollUpButton>
                    <Select.Viewport className="p-1">
                        {options.map((opt) => (
                            <Select.Item
                                key={opt.value}
                                value={opt.value}
                                className={cn(
                                    "relative flex items-center px-8 py-2 text-sm text-gray-700 dark:text-gray-300 rounded-lg cursor-pointer outline-none select-none hover:bg-blue-50 dark:hover:bg-blue-900/30 transition-colors",
                                    "data-[highlighted]:bg-blue-50 data-[highlighted]:text-blue-700 dark:data-[highlighted]:bg-blue-900/30 dark:data-[highlighted]:text-blue-200",
                                    "data-[state=checked]:font-bold data-[state=checked]:text-blue-600 dark:data-[state=checked]:text-blue-400"
                                )}
                            >
                                <Select.ItemText>{opt.label}</Select.ItemText>
                                <Select.ItemIndicator className="absolute left-2 inline-flex items-center justify-center">
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
