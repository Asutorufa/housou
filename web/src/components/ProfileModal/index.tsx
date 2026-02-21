import * as Dialog from "@radix-ui/react-dialog";
import * as Tabs from "@radix-ui/react-tabs";
import { KeyRound, Link, Lock, User, X } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useCallback, useEffect, useRef, useState } from "react";
import { useAuth } from "../../contexts/AuthContext";
import ConnectedAccountsTab from "./ConnectedAccountsTab";
import PasskeyTab from "./PasskeyTab";
import ProfileTab from "./ProfileTab";
import SecurityTab from "./SecurityTab";

interface ProfileModalProps {
  isOpen: boolean;
  onClose: () => void;
  githubEnabled?: boolean;
  telegramBotName?: string;
}

const tabTriggerClass =
  "relative flex items-center justify-center gap-1.5 rounded-md py-2 text-sm font-medium text-gray-600 transition-colors data-[state=active]:text-gray-900 dark:text-gray-400 dark:data-[state=active]:text-gray-100";

const TAB_ORDER = ["profile", "security", "connected", "passkey"] as const;

export default function ProfileModal({
  isOpen,
  onClose,
  githubEnabled,
  telegramBotName,
}: ProfileModalProps) {
  const { user, updateProfile } = useAuth();
  const [activeTab, setActiveTab] = useState("profile");
  const [direction, setDirection] = useState(0);
  const contentRef = useRef<HTMLDivElement>(null);
  const [contentHeight, setContentHeight] = useState<number | "auto">("auto");

  useEffect(() => {
    const el = contentRef.current;
    if (!el) return;
    const observer = new ResizeObserver(() => {
      setContentHeight(el.offsetHeight);
    });
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  const handleTabChange = useCallback(
    (value: string) => {
      const oldIdx = TAB_ORDER.indexOf(activeTab as (typeof TAB_ORDER)[number]);
      const newIdx = TAB_ORDER.indexOf(value as (typeof TAB_ORDER)[number]);
      setDirection(newIdx > oldIdx ? 1 : -1);
      setActiveTab(value);
    },
    [activeTab],
  );

  return (
    <Dialog.Root open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <AnimatePresence>
        {isOpen && (
          <Dialog.Portal forceMount>
            <Dialog.Overlay asChild>
              <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                className="fixed inset-0 z-50 bg-black/50 backdrop-blur-sm"
              />
            </Dialog.Overlay>
            <Dialog.Content asChild>
              <motion.div
                initial={{ opacity: 0, scale: 0.95, x: "-50%", y: "-48%" }}
                animate={{ opacity: 1, scale: 1, x: "-50%", y: "-50%" }}
                exit={{ opacity: 0, scale: 0.95, x: "-50%", y: "-48%" }}
                transition={{ type: "spring", damping: 25, stiffness: 300 }}
                className="fixed left-[50%] top-[50%] z-50 flex w-full max-w-md sm:max-w-xl flex-col rounded-2xl border border-gray-200 bg-white shadow-xl dark:border-gray-800 dark:bg-gray-900 focus:outline-none max-h-[85vh]"
              >
                {/* Header */}
                <div className="flex items-center justify-between px-6 pt-5 pb-0">
                  <Dialog.Title className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                    設定
                  </Dialog.Title>
                  <Dialog.Close className="rounded-full p-1.5 text-gray-500 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-800">
                    <X size={18} />
                  </Dialog.Close>
                </div>

                {/* Tabs */}
                <Tabs.Root value={activeTab} onValueChange={handleTabChange}>
                  <div className="mx-6 mt-4 mb-0 rounded-lg bg-gray-100 p-1 dark:bg-gray-800">
                    <Tabs.List className="relative grid grid-cols-4">
                      <motion.div
                        className="pointer-events-none absolute inset-y-0 w-1/4 rounded-md bg-white shadow-sm dark:bg-gray-700"
                        initial={false}
                        animate={{
                          x: `${TAB_ORDER.indexOf(activeTab as (typeof TAB_ORDER)[number]) * 100}%`,
                        }}
                        transition={{
                          type: "spring",
                          bounce: 0.15,
                          duration: 0.4,
                        }}
                      />
                      <Tabs.Trigger value="profile" className={tabTriggerClass}>
                        <span className="relative z-10 flex items-center gap-1.5">
                          <User size={14} />
                          <span className="hidden sm:inline">プロフィール</span>
                          <span className="sm:hidden">基本</span>
                        </span>
                      </Tabs.Trigger>
                      <Tabs.Trigger
                        value="security"
                        className={tabTriggerClass}
                      >
                        <span className="relative z-10 flex items-center gap-1.5">
                          <Lock size={14} />
                          <span className="hidden sm:inline">セキュリティ</span>
                          <span className="sm:hidden">鍵</span>
                        </span>
                      </Tabs.Trigger>
                      <Tabs.Trigger
                        value="connected"
                        className={tabTriggerClass}
                      >
                        <span className="relative z-10 flex items-center gap-1.5">
                          <Link size={14} />
                          <span className="hidden sm:inline">連携</span>
                          <span className="sm:hidden">連携</span>
                        </span>
                      </Tabs.Trigger>
                      <Tabs.Trigger value="passkey" className={tabTriggerClass}>
                        <span className="relative z-10 flex items-center gap-1.5">
                          <KeyRound size={14} />
                          <span className="hidden sm:inline">パスキー</span>
                          <span className="sm:hidden">生体</span>
                        </span>
                      </Tabs.Trigger>
                    </Tabs.List>
                  </div>

                  {/* Animated content area */}
                  <motion.div
                    className="overflow-hidden"
                    animate={{ height: contentHeight }}
                    transition={{
                      type: "spring",
                      bounce: 0,
                      duration: 0.35,
                    }}
                  >
                    <div ref={contentRef} className="px-6 py-5">
                      <AnimatePresence
                        mode="wait"
                        initial={false}
                        custom={direction}
                      >
                        <motion.div
                          key={activeTab}
                          custom={direction}
                          variants={{
                            enter: (d: number) => ({
                              x: `${d * 15}%`,
                              opacity: 0,
                            }),
                            center: { x: 0, opacity: 1 },
                            exit: (d: number) => ({
                              x: `${d * -15}%`,
                              opacity: 0,
                            }),
                          }}
                          initial="enter"
                          animate="center"
                          exit="exit"
                          transition={{
                            duration: 0.2,
                            ease: [0.25, 0.1, 0.25, 1],
                          }}
                        >
                          {activeTab === "profile" && (
                            <ProfileTab
                              user={user}
                              updateProfile={updateProfile}
                            />
                          )}
                          {activeTab === "security" && <SecurityTab />}
                          {activeTab === "connected" && (
                            <ConnectedAccountsTab
                              githubEnabled={githubEnabled}
                              telegramBotName={telegramBotName}
                            />
                          )}
                          {activeTab === "passkey" && <PasskeyTab />}
                        </motion.div>
                      </AnimatePresence>
                    </div>
                  </motion.div>
                </Tabs.Root>
              </motion.div>
            </Dialog.Content>
          </Dialog.Portal>
        )}
      </AnimatePresence>
    </Dialog.Root>
  );
}
