import * as Select from "@radix-ui/react-select";
import { clsx, type ClassValue } from "clsx";
import { ChevronDown } from "lucide-react";
import { motion } from "motion/react";
import { twMerge } from "tailwind-merge";

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export interface CustomSelectProps {
  value: string;
  onValueChange: (value: string) => void;
  options: { value: string; label: string }[];
  placeholder?: string;
  isOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
  icon?: React.ReactNode;
  triggerClassName?: string;
  contentClassName?: string;
  align?: "start" | "center" | "end";
}

export default function CustomSelect({
  value,
  onValueChange,
  options,
  placeholder,
  isOpen,
  onOpenChange,
  icon,
  triggerClassName,
  contentClassName,
  align = "start",
}: CustomSelectProps) {
  return (
    <Select.Root
      value={value}
      onValueChange={onValueChange}
      open={isOpen}
      onOpenChange={onOpenChange}
    >
      <Select.Trigger asChild>
        <motion.button
          className={cn(
            "inline-flex cursor-pointer items-center justify-between rounded-full px-3 py-1.5 text-xs font-medium whitespace-nowrap text-gray-700 transition-colors duration-200 sm:text-sm dark:text-gray-200",
            "ring-0 outline-none focus:ring-0 focus:outline-none focus-visible:ring-0 focus-visible:outline-none",
            "hover:bg-gray-100 dark:hover:bg-gray-700/50",
            isOpen && "text-blue-600 dark:text-blue-400",
            triggerClassName,
          )}
        >
          <div className="flex items-center gap-2">
            {icon && <span className="shrink-0">{icon}</span>}
            <Select.Value placeholder={placeholder} />
          </div>
          <Select.Icon className="ml-1 text-gray-400">
            <motion.div
              animate={{ rotate: isOpen ? 180 : 0 }}
              transition={{ duration: 0.2 }}
            >
              <ChevronDown size={14} />
            </motion.div>
          </Select.Icon>
        </motion.button>
      </Select.Trigger>

      <Select.Portal>
        <Select.Content
          sideOffset={8}
          position="popper"
          align={align}
          className={cn(
            "select-content z-[200] min-w-[120px] overflow-hidden rounded-2xl border border-gray-200/50 bg-white/95 shadow-xl shadow-black/10 backdrop-blur-xl dark:border-gray-700/50 dark:bg-gray-800/95 dark:shadow-black/30",
            contentClassName,
          )}
        >
          <Select.Viewport className="custom-scrollbar scroll-fade max-h-[300px] overflow-y-auto p-1">
            {options.map((opt) => (
              <Select.Item
                key={opt.value}
                value={opt.value}
                className={cn(
                  "relative flex cursor-pointer items-center rounded-xl px-2 py-2 pl-8 text-sm text-gray-700 transition-colors outline-none select-none dark:text-gray-200",
                  "focus:bg-blue-50 focus:text-blue-700 dark:focus:bg-blue-900/30 dark:focus:text-blue-200",
                  "data-[state=checked]:font-semibold data-[state=checked]:text-blue-600 dark:data-[state=checked]:text-blue-400",
                )}
              >
                <Select.ItemText>{opt.label}</Select.ItemText>
                <Select.ItemIndicator className="absolute left-2.5 inline-flex items-center justify-center text-blue-500">
                  <div className="h-1.5 w-1.5 rounded-full bg-current" />
                </Select.ItemIndicator>
              </Select.Item>
            ))}
          </Select.Viewport>
        </Select.Content>
      </Select.Portal>
    </Select.Root>
  );
}
