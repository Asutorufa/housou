import * as Dialog from "@radix-ui/react-dialog";
import * as Tabs from "@radix-ui/react-tabs";
import { Eye, EyeOff, KeyRound, Lock, User, X } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useCallback, useEffect, useRef, useState } from "react";
import { type PasskeySummary, useAuth } from "../contexts/AuthContext";

interface ProfileModalProps {
  isOpen: boolean;
  onClose: () => void;
}

const tabTriggerClass =
  "relative flex items-center justify-center gap-1.5 rounded-md py-2 text-sm font-medium text-gray-600 transition-colors data-[state=active]:text-gray-900 dark:text-gray-400 dark:data-[state=active]:text-gray-100";

const inputClass =
  "w-full rounded-lg border border-gray-300 bg-transparent px-3 py-2 text-sm placeholder:text-gray-400 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 dark:border-gray-700 dark:text-gray-100";

function ProfileTab({
  user,
  updateProfile,
}: {
  user: { username: string; email: string; avatar_url?: string } | undefined;
  updateProfile: (data: {
    username: string;
    email: string;
    avatar_url: string;
  }) => Promise<unknown>;
}) {
  const [username, setUsername] = useState(user?.username || "");
  const [email, setEmail] = useState(user?.email || "");
  const [avatarUrl, setAvatarUrl] = useState(user?.avatar_url || "");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  useEffect(() => {
    if (user) {
      setUsername(user.username);
      setEmail(user.email);
      setAvatarUrl(user.avatar_url || "");
    }
  }, [user]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setSuccess(false);
    setLoading(true);

    try {
      await updateProfile({ username, email, avatar_url: avatarUrl });
      setSuccess(true);
      setTimeout(() => setSuccess(false), 3000);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Update failed");
    } finally {
      setLoading(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div>
        <label className="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300">
          メールアドレス
        </label>
        <input
          type="email"
          required
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          className={inputClass}
        />
      </div>

      <div>
        <label className="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300">
          ユーザー名
        </label>
        <input
          type="text"
          required
          value={username}
          onChange={(e) => setUsername(e.target.value)}
          className={inputClass}
        />
      </div>

      <div>
        <label className="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300">
          アバター URL
        </label>
        <input
          type="url"
          value={avatarUrl}
          onChange={(e) => setAvatarUrl(e.target.value)}
          placeholder="https://example.com/avatar.png"
          className={inputClass}
        />
      </div>

      {error && (
        <div className="rounded-lg bg-red-50 p-3 text-sm text-red-500 dark:bg-red-900/20">
          {error}
        </div>
      )}

      {success && (
        <div className="rounded-lg bg-green-50 p-3 text-sm text-green-500 dark:bg-green-900/20">
          プロフィールを更新しました！
        </div>
      )}

      <button
        type="submit"
        disabled={loading}
        className="w-full rounded-lg bg-blue-600 px-4 py-2 text-sm font-semibold text-white transition-colors hover:bg-blue-700 disabled:opacity-50"
      >
        {loading ? "保存中..." : "保存"}
      </button>
    </form>
  );
}

function SecurityTab() {
  const { changePassword } = useAuth();
  const [oldPassword, setOldPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [confirmNewPassword, setConfirmNewPassword] = useState("");
  const [showOldPassword, setShowOldPassword] = useState(false);
  const [showNewPassword, setShowNewPassword] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setSuccess(false);

    if (newPassword.length < 8) {
      setError("新しいパスワードは8文字以上である必要があります");
      return;
    }

    if (newPassword !== confirmNewPassword) {
      setError("新しいパスワードが一致しません");
      return;
    }

    setLoading(true);

    try {
      await changePassword({
        old_password: oldPassword || undefined,
        new_password: newPassword,
      });
      setSuccess(true);
      setOldPassword("");
      setNewPassword("");
      setConfirmNewPassword("");
      setTimeout(() => setSuccess(false), 3000);
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "パスワードの更新に失敗しました",
      );
    } finally {
      setLoading(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div>
        <label className="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300">
          現在のパスワード
        </label>
        <div className="relative">
          <input
            type={showOldPassword ? "text" : "password"}
            value={oldPassword}
            onChange={(e) => setOldPassword(e.target.value)}
            className={`${inputClass} pr-10`}
          />
          <button
            type="button"
            onClick={() => setShowOldPassword(!showOldPassword)}
            className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200"
          >
            {showOldPassword ? <EyeOff size={16} /> : <Eye size={16} />}
          </button>
        </div>
      </div>

      <div>
        <label className="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300">
          新しいパスワード (8文字以上)
        </label>
        <div className="relative">
          <input
            type={showNewPassword ? "text" : "password"}
            required
            value={newPassword}
            onChange={(e) => setNewPassword(e.target.value)}
            className={`${inputClass} pr-10`}
          />
          <button
            type="button"
            onClick={() => setShowNewPassword(!showNewPassword)}
            className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200"
          >
            {showNewPassword ? <EyeOff size={16} /> : <Eye size={16} />}
          </button>
        </div>
      </div>

      <div>
        <label className="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300">
          新しいパスワード確認
        </label>
        <input
          type={showNewPassword ? "text" : "password"}
          required
          value={confirmNewPassword}
          onChange={(e) => setConfirmNewPassword(e.target.value)}
          className={inputClass}
        />
      </div>

      {error && (
        <div className="rounded-lg bg-red-50 p-3 text-sm text-red-500 dark:bg-red-900/20">
          {error}
        </div>
      )}

      {success && (
        <div className="rounded-lg bg-green-50 p-3 text-sm text-green-500 dark:bg-green-900/20">
          パスワードを更新しました！
        </div>
      )}

      <button
        type="submit"
        disabled={loading}
        className="w-full rounded-lg bg-gray-800 px-4 py-2 text-sm font-semibold text-white transition-colors hover:bg-gray-900 disabled:opacity-50 dark:bg-gray-700 dark:hover:bg-gray-600"
      >
        {loading ? "更新中..." : "パスワードを更新"}
      </button>
    </form>
  );
}

function PasskeyTab() {
  const { registerPasskey, listPasskeys, deletePasskey } = useAuth();
  const [passkeys, setPasskeys] = useState<PasskeySummary[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadPasskeys = useCallback(async () => {
    try {
      const list = await listPasskeys();
      setPasskeys(list);
    } catch (err) {
      console.error(err);
    }
  }, [listPasskeys]);

  useEffect(() => {
    loadPasskeys();
  }, [loadPasskeys]);

  const handleAdd = async () => {
    setLoading(true);
    setError(null);
    try {
      await registerPasskey();
      await loadPasskeys();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to add passkey");
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm("このパスキーを削除してもよろしいですか？")) return;
    try {
      await deletePasskey(id);
      await loadPasskeys();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to delete passkey");
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <p className="text-sm text-gray-500 dark:text-gray-400">
          パスキーを使用してパスワードなしでログインできます。
        </p>
        <button
          type="button"
          onClick={handleAdd}
          disabled={loading}
          className="shrink-0 rounded-lg bg-blue-600 px-3 py-1.5 text-xs font-semibold text-white transition-colors hover:bg-blue-700 disabled:opacity-50"
        >
          {loading ? "追加中..." : "追加"}
        </button>
      </div>

      {error && (
        <div className="rounded-lg bg-red-50 p-3 text-sm text-red-500 dark:bg-red-900/20">
          {error}
        </div>
      )}

      {passkeys.length === 0 ? (
        <div className="flex flex-col items-center gap-3 rounded-xl border border-dashed border-gray-200 py-8 dark:border-gray-700">
          <KeyRound size={32} className="text-gray-300 dark:text-gray-600" />
          <p className="text-sm text-gray-400 dark:text-gray-500">
            パスキーは登録されていません。
          </p>
        </div>
      ) : (
        <div className="space-y-2">
          {passkeys.map((pk) => (
            <div
              key={pk.id}
              className="flex items-center justify-between rounded-lg border border-gray-200 p-3 transition-colors hover:bg-gray-50 dark:border-gray-700 dark:hover:bg-gray-800/50"
            >
              <div className="flex items-center gap-3">
                <div className="flex h-8 w-8 items-center justify-center rounded-full bg-blue-50 dark:bg-blue-900/20">
                  <KeyRound
                    size={14}
                    className="text-blue-600 dark:text-blue-400"
                  />
                </div>
                <div>
                  <div className="text-sm font-medium text-gray-900 dark:text-gray-100">
                    {pk.name}
                  </div>
                  <div className="text-xs text-gray-500 dark:text-gray-400">
                    登録日: {new Date(pk.createdAt).toLocaleDateString()}
                  </div>
                </div>
              </div>
              <button
                type="button"
                onClick={() => handleDelete(pk.id)}
                className="rounded-full p-1.5 text-gray-400 transition-colors hover:bg-red-50 hover:text-red-500 dark:hover:bg-red-900/20 dark:hover:text-red-400"
              >
                <X size={14} />
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
const TAB_ORDER = ["profile", "security", "passkey"] as const;

export default function ProfileModal({ isOpen, onClose }: ProfileModalProps) {
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
                className="fixed left-[50%] top-[50%] z-50 flex w-full max-w-md flex-col rounded-2xl border border-gray-200 bg-white shadow-xl dark:border-gray-800 dark:bg-gray-900 focus:outline-none max-h-[85vh]"
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
                    <Tabs.List className="relative grid grid-cols-3">
                      <motion.div
                        className="pointer-events-none absolute inset-y-0 w-1/3 rounded-md bg-white shadow-sm dark:bg-gray-700"
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
                          <span>セキュリティ</span>
                        </span>
                      </Tabs.Trigger>
                      <Tabs.Trigger value="passkey" className={tabTriggerClass}>
                        <span className="relative z-10 flex items-center gap-1.5">
                          <KeyRound size={14} />
                          <span>パスキー</span>
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
