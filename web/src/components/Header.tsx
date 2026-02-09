import * as Select from "@radix-ui/react-select";
import { clsx, type ClassValue } from "clsx";
import { AnimatePresence, motion } from "framer-motion";
import { ChevronDown, Search, X } from "lucide-react";
import { useState } from "react";
import { twMerge } from "tailwind-merge";
import type { Config } from "../types";

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

interface HeaderProps {
  config: Config | null;
  selectedYear: string;
  setSelectedYear: (year: string) => void;
  selectedSeason: string;
  setSelectedSeason: (season: string) => void;
  selectedSite: string;
  setSelectedSite: (site: string) => void;
  searchQuery: string;
  setSearchQuery: (query: string) => void;
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
  const [isSearchFocused, setIsSearchFocused] = useState(false);

  return (
    <header className="group/header sticky top-2 z-50 mx-2 rounded-4xl border border-gray-200/50 bg-white/80 p-2 shadow-md backdrop-blur-md transition-all md:mx-4 dark:border-gray-700/50 dark:bg-gray-800/80">
      <div className="mx-auto flex max-w-7xl items-center gap-3">
        <AnimatePresence mode="popLayout" initial={false}>
          {!isSearchFocused && (
            <motion.div
              key="filters"
              initial={{ opacity: 0, x: -20 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: -20 }}
              transition={{ duration: 0.2 }}
              className="scrollbar-hide flex flex-1 items-center gap-2 overflow-x-auto md:flex"
            >
              {/* Year Select */}
              <CustomSelect
                value={selectedYear}
                onValueChange={setSelectedYear}
                options={
                  config?.years.map((y) => ({
                    value: y.toString(),
                    label: y.toString(),
                  })) || []
                }
                placeholder="年"
              />

              {/* Season Select */}
              <CustomSelect
                value={selectedSeason}
                onValueChange={setSelectedSeason}
                options={[
                  { value: "all", label: "全て" },
                  { value: "Winter", label: "冬" },
                  { value: "Spring", label: "春" },
                  { value: "Summer", label: "夏" },
                  { value: "Autumn", label: "秋" },
                ]}
                placeholder="シーズン"
              />

              {/* Site Select */}
              <CustomSelect
                value={selectedSite}
                onValueChange={setSelectedSite}
                options={[
                  { value: "all", label: "全て" },
                  ...Object.entries(config?.site_meta || {}).map(
                    ([key, meta]) => ({
                      value: key,
                      label: meta?.title || key,
                    }),
                  ),
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
            "group relative shrink-0 overflow-hidden rounded-full border-2 border-transparent bg-gray-100/50 hover:border-blue-500/50 dark:bg-gray-700/50",
            isSearchFocused
              ? "w-full border-blue-500 ring-2 ring-blue-500/20"
              : "w-10 md:w-64",
          )}
        >
          <div
            className={cn(
              "pointer-events-none absolute inset-0 flex items-center justify-center text-gray-400 transition-all duration-300 md:inset-y-0 md:right-auto md:left-3 md:w-auto md:justify-start",
              isSearchFocused &&
                "inset-y-0 right-auto left-3 w-auto justify-start text-blue-500",
            )}
          >
            <Search size={16} />
          </div>
          <input
            type="text"
            placeholder="検索..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            onFocus={() => setIsSearchFocused(true)}
            onBlur={() => setIsSearchFocused(false)}
            className="w-full border-none bg-transparent py-1.5 pr-8 pl-9 text-sm text-gray-900 placeholder-transparent outline-none focus:placeholder-gray-500 md:placeholder-gray-500 dark:text-gray-100 dark:focus:placeholder-gray-400 md:dark:placeholder-gray-400"
          />
          <AnimatePresence>
            {searchQuery && (
              <motion.button
                initial={{ opacity: 0, scale: 0.8 }}
                animate={{ opacity: 1, scale: 1 }}
                exit={{ opacity: 0, scale: 0.8 }}
                onClick={() => setSearchQuery("")}
                className={cn(
                  "absolute inset-y-0 right-2 hidden items-center text-gray-400 hover:text-gray-600 md:flex dark:hover:text-gray-200",
                  isSearchFocused && "flex",
                )}
              >
                <X size={14} />
              </motion.button>
            )}
          </AnimatePresence>
        </motion.div>
      </div>
    </header>
  );
}

interface CustomSelectProps {
  value: string;
  onValueChange: (value: string) => void;
  options: { value: string; label: string }[];
  placeholder: string;
}

function CustomSelect({
  value,
  onValueChange,
  options,
  placeholder,
}: CustomSelectProps) {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <Select.Root
      value={value}
      onValueChange={onValueChange}
      open={isOpen}
      onOpenChange={setIsOpen}
    >
      <Select.Trigger asChild>
        <motion.button
          className={cn(
            "inline-flex min-w-[80px] cursor-pointer items-center justify-between rounded-full border-2 bg-gray-100 px-3 py-1.5 text-xs font-medium whitespace-nowrap text-gray-900 transition-all duration-200 sm:min-w-[100px] sm:text-sm dark:bg-gray-700 dark:text-gray-100",
            "ring-0 outline-none focus:ring-0 focus:outline-none focus-visible:ring-0 focus-visible:outline-none",
            "active:brightness-95",
            isOpen
              ? "border-blue-500"
              : "border-transparent hover:border-blue-500",
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
          className="select-content z-[60] overflow-hidden rounded-2xl border border-gray-200/50 bg-white/95 shadow-2xl shadow-black/10 backdrop-blur-xl dark:border-gray-700/50 dark:bg-gray-800/95 dark:shadow-black/30"
        >
          <Select.ScrollUpButton className="flex h-8 cursor-default items-center justify-center bg-white text-gray-700 dark:bg-gray-800 dark:text-gray-300">
            <ChevronDown className="rotate-180" size={16} />
          </Select.ScrollUpButton>
          <Select.Viewport className="max-h-[300px] overflow-y-auto p-1.5">
            {options.map((opt) => (
              <Select.Item
                key={opt.value}
                value={opt.value}
                className={cn(
                  "relative flex cursor-pointer items-center rounded-2xl px-8 py-2.5 text-sm text-gray-700 transition-all duration-150 outline-none select-none dark:text-gray-300",
                  "hover:bg-blue-50 dark:hover:bg-blue-900/30",
                  "data-[highlighted]:bg-blue-50 data-[highlighted]:text-blue-700 dark:data-[highlighted]:bg-blue-900/30 dark:data-[highlighted]:text-blue-200",
                  "data-[state=checked]:font-bold data-[state=checked]:text-blue-600 dark:data-[state=checked]:text-blue-400",
                )}
              >
                <Select.ItemText>{opt.label}</Select.ItemText>
                <Select.ItemIndicator className="absolute left-2.5 inline-flex items-center justify-center">
                  <div className="h-1.5 w-1.5 rounded-full bg-blue-500" />
                </Select.ItemIndicator>
              </Select.Item>
            ))}
          </Select.Viewport>
          <Select.ScrollDownButton className="flex h-8 cursor-default items-center justify-center bg-white text-gray-700 dark:bg-gray-800 dark:text-gray-300">
            <ChevronDown size={16} />
          </Select.ScrollDownButton>
        </Select.Content>
      </Select.Portal>
    </Select.Root>
  );
}
