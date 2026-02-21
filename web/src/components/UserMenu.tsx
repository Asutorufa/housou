import { LogOut, Settings, User as UserIcon } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import type { ReactNode } from "react";
import { useAuth } from "../contexts/AuthContext";
import { focusRingClassName, iconButtonClassName } from "../styles/uiClasses";
import { cn } from "../utils/cn";

interface UserMenuProps {
  isOpen: boolean;
  onOpenChange: (open: boolean) => void;
  onOpenProfile: () => void;
}

interface UserMenuActionButtonProps {
  label: string;
  icon: ReactNode;
  onClick: () => void;
  tone?: "default" | "danger";
}

function UserMenuActionButton({
  label,
  icon,
  onClick,
  tone = "default",
}: UserMenuActionButtonProps) {
  return (
    <button
      onClick={onClick}
      className={cn(
        `flex w-full cursor-pointer items-center gap-2 rounded-lg px-3 py-2 text-sm transition-colors ${focusRingClassName}`,
        tone === "danger"
          ? "text-red-600 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-900/20"
          : "text-gray-700 hover:bg-blue-50 hover:text-blue-600 dark:text-gray-200 dark:hover:bg-blue-900/30 dark:hover:text-blue-400",
      )}
    >
      {icon}
      {label}
    </button>
  );
}

export default function UserMenu({
  isOpen,
  onOpenChange,
  onOpenProfile,
}: UserMenuProps) {
  const { user, logout } = useAuth();

  return (
    <div className="relative">
      <motion.button
        onClick={() => onOpenChange(!isOpen)}
        className={cn(
          `${iconButtonClassName} overflow-hidden border border-gray-200 bg-white shadow-sm hover:bg-gray-50 dark:border-gray-700 dark:bg-gray-800 dark:hover:bg-gray-700`,
          isOpen && "ring-2 ring-blue-500/20",
        )}
      >
        {user?.avatar_url ? (
          <img
            src={user.avatar_url}
            alt={user.username}
            className="h-full w-full object-cover"
          />
        ) : (
          <span className="text-sm font-semibold text-gray-700 dark:text-gray-200">
            {user?.username?.[0]?.toUpperCase() || <UserIcon size={16} />}
          </span>
        )}
      </motion.button>

      <AnimatePresence>
        {isOpen && (
          <>
            <div
              className="fixed inset-0 z-40 bg-transparent"
              onClick={() => onOpenChange(false)}
            />
            <motion.div
              initial={{ opacity: 0, y: 10, scale: 0.95 }}
              animate={{ opacity: 1, y: 0, scale: 1 }}
              exit={{ opacity: 0, y: 10, scale: 0.95 }}
              transition={{ duration: 0.1 }}
              className="absolute right-0 top-full z-50 mt-2 w-48 rounded-xl border border-gray-200/50 bg-white/95 p-1 shadow-xl backdrop-blur-xl dark:border-gray-700/50 dark:bg-gray-800/95"
            >
              <div className="px-3 py-2 border-b border-gray-100 dark:border-gray-700/50 mb-1">
                <p className="text-xs font-medium text-gray-500 dark:text-gray-400">
                  ログイン中:
                </p>
                <p className="truncate text-sm font-semibold text-gray-900 dark:text-gray-100">
                  {user?.username}
                </p>
              </div>

              <UserMenuActionButton
                label="プロフィール"
                icon={<Settings size={16} />}
                onClick={() => {
                  onOpenProfile();
                  onOpenChange(false);
                }}
              />

              <UserMenuActionButton
                label="ログアウト"
                icon={<LogOut size={16} />}
                tone="danger"
                onClick={() => {
                  logout();
                  onOpenChange(false);
                }}
              />
            </motion.div>
          </>
        )}
      </AnimatePresence>
    </div>
  );
}
