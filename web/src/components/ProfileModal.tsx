import * as Dialog from "@radix-ui/react-dialog";
import * as Tabs from "@radix-ui/react-tabs";
import {
  Check,
  Eye,
  EyeOff,
  KeyRound,
  Link,
  Loader2,
  Lock,
  Pencil,
  User,
  X,
} from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useCallback, useEffect, useRef, useState } from "react";
import { type PasskeySummary, useAuth } from "../contexts/AuthContext";
import TelegramLoginButton from "./TelegramLoginButton";

interface ProfileModalProps {
  isOpen: boolean;
  onClose: () => void;
  githubEnabled?: boolean;
  telegramBotName?: string;
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
  const { user, changePassword } = useAuth();
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
      {user?.has_password && (
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
      )}

      <div>
        <label className="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300">
          {user?.has_password
            ? "新しいパスワード (8文字以上)"
            : "パスワードを設定 (8文字以上)"}
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

function ConnectedAccountsTab({
  githubEnabled,
  telegramBotName,
}: {
  githubEnabled?: boolean;
  telegramBotName?: string;
}) {
  const { user, bindGithub, unbindGithub, bindTelegram, unbindTelegram } =
    useAuth();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleUnbind = async (
    serviceName: string,
    unbindFn: () => Promise<void>,
    hasAlternativeLogin: boolean,
  ) => {
    if (!hasAlternativeLogin) {
      setError(
        "連携を解除する前に、パスワードを設定するか、他のサービスと連携してください。",
      );
      return;
    }
    if (!confirm(`${serviceName}連携を解除してもよろしいですか？`)) return;

    setLoading(true);
    setError(null);
    try {
      await unbindFn();
    } catch (err) {
      setError(
        err instanceof Error ? err.message : `Failed to unbind ${serviceName}`,
      );
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-4">
      <p className="text-sm text-gray-500 dark:text-gray-400">
        外部サービスと連携してログインできます。
      </p>

      {error && (
        <div className="rounded-lg bg-red-50 p-3 text-sm text-red-500 dark:bg-red-900/20">
          {error}
        </div>
      )}

      {githubEnabled && (
        <div className="flex items-center justify-between rounded-lg border border-gray-200 p-4 dark:border-gray-700">
          <div className="flex items-center gap-3">
            <svg
              className="h-6 w-6 fill-current text-gray-900 dark:text-white"
              viewBox="0 0 24 24"
            >
              <path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12" />
            </svg>
            <div>
              <div className="font-medium text-gray-900 dark:text-gray-100">
                GitHub
              </div>
              <div className="text-xs text-gray-500 dark:text-gray-400">
                {user?.github_id ? "連携済み" : "未連携"}
              </div>
            </div>
          </div>

          {user?.github_id ? (
            <button
              onClick={() =>
                handleUnbind(
                  "GitHub",
                  unbindGithub,
                  !!(user?.has_password || user?.telegram_id),
                )
              }
              disabled={loading}
              className="rounded-lg border border-red-200 bg-red-50 px-3 py-1.5 text-sm font-medium text-red-600 hover:bg-red-100 disabled:opacity-50 dark:border-red-900/30 dark:bg-red-900/10 dark:text-red-400 dark:hover:bg-red-900/20"
            >
              解除
            </button>
          ) : (
            <button
              onClick={bindGithub}
              className="rounded-lg bg-gray-900 px-3 py-1.5 text-sm font-medium text-white hover:bg-gray-800 dark:bg-gray-700 dark:hover:bg-gray-600"
            >
              連携
            </button>
          )}
        </div>
      )}

      {telegramBotName && (
        <div className="flex items-center justify-between rounded-lg border border-gray-200 p-4 dark:border-gray-700">
          <div className="flex items-center gap-3">
            <svg
              className="h-6 w-6 fill-current text-[#229ED9] dark:text-[#2AABEE]"
              viewBox="0 0 24 24"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path d="M11.944 0A12 12 0 0 0 0 12a12 12 0 0 0 12 12 12 12 0 0 0 12-12A12 12 0 0 0 12 0a12 12 0 0 0-.056 0zm4.962 7.224c.1-.002.321.023.465.14a.506.506 0 0 1 .171.325c.016.093.036.306.02.472-.18 1.898-.962 6.502-1.36 8.627-.168.9-.499 1.201-.82 1.23-.696.065-1.225-.46-1.9-.902-1.056-.693-1.653-1.124-2.678-1.8-1.185-.78-.417-1.21.258-1.91.177-.184 3.247-2.977 3.307-3.23.007-.032.014-.15-.056-.212s-.174-.041-.249-.024c-.106.024-1.793 1.14-5.061 3.345-.48.33-.913.49-1.302.48-.428-.008-1.252-.241-1.865-.44-.752-.245-1.349-.374-1.297-.789.027-.216.325-.437.893-.663 3.498-1.524 5.83-2.529 6.998-3.014 3.332-1.386 4.025-1.627 4.476-1.635z" />
            </svg>
            <div>
              <div className="font-medium text-gray-900 dark:text-gray-100">
                Telegram
              </div>
              <div className="text-xs text-gray-500 dark:text-gray-400">
                {user?.telegram_id ? "連携済み" : "未連携"}
              </div>
            </div>
          </div>

          {user?.telegram_id ? (
            <button
              onClick={() =>
                handleUnbind(
                  "Telegram",
                  unbindTelegram,
                  !!(user?.has_password || user?.github_id),
                )
              }
              disabled={loading}
              className="rounded-lg border border-red-200 bg-red-50 px-3 py-1.5 text-sm font-medium text-red-600 hover:bg-red-100 disabled:opacity-50 dark:border-red-900/30 dark:bg-red-900/10 dark:text-red-400 dark:hover:bg-red-900/20"
            >
              解除
            </button>
          ) : (
            <div className="h-[28px] flex items-center">
              <TelegramLoginButton
                botName={telegramBotName}
                cornerRadius={4}
                buttonSize="medium"
                usePic={false}
                onAuth={async (data) => {
                  setLoading(true);
                  setError(null);
                  try {
                    await bindTelegram(data);
                  } catch (err) {
                    setError(
                      err instanceof Error
                        ? err.message
                        : "Telegram bind failed",
                    );
                  } finally {
                    setLoading(false);
                  }
                }}
              />
            </div>
          )}
        </div>
      )}
    </div>
  );
}

function PasskeyTab() {
  const { registerPasskey, listPasskeys, deletePasskey, renamePasskey } =
    useAuth();
  const [passkeys, setPasskeys] = useState<PasskeySummary[]>([]);
  const [loading, setLoading] = useState(false);
  const [isListLoading, setIsListLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editName, setEditName] = useState("");

  const loadPasskeys = useCallback(async () => {
    try {
      const list = await listPasskeys();
      setPasskeys(list);
    } catch (err) {
      console.error(err);
    } finally {
      setIsListLoading(false);
    }
  }, [listPasskeys]);

  useEffect(() => {
    loadPasskeys();
  }, [loadPasskeys]);

  const getDeviceName = () => {
    const ua = navigator.userAgent;
    let os = "Unknown OS";
    if (ua.includes("Mac")) os = "macOS";
    else if (ua.includes("Win")) os = "Windows";
    else if (ua.includes("iPhone")) os = "iPhone";
    else if (ua.includes("iPad")) os = "iPad";
    else if (ua.includes("Android")) os = "Android";
    else if (ua.includes("Linux")) os = "Linux";

    let browser = "Unknown Browser";
    if (ua.includes("Edg")) browser = "Edge";
    else if (ua.includes("Chrome")) browser = "Chrome";
    else if (ua.includes("Firefox")) browser = "Firefox";
    else if (ua.includes("Safari")) browser = "Safari";

    return `${os} (${browser})`;
  };

  const handleAdd = async () => {
    setLoading(true);
    setError(null);
    try {
      await registerPasskey(getDeviceName());
      await loadPasskeys();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to add passkey");
    } finally {
      setLoading(false);
    }
  };

  const startEdit = (pk: PasskeySummary) => {
    setEditingId(pk.id);
    setEditName(pk.name);
  };

  const cancelEdit = () => {
    setEditingId(null);
    setEditName("");
  };

  const saveEdit = async (id: string) => {
    if (
      !editName.trim() ||
      editName === passkeys.find((p) => p.id === id)?.name
    ) {
      cancelEdit();
      return;
    }

    try {
      await renamePasskey(id, editName);
      setEditingId(null);
      await loadPasskeys();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to rename passkey");
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

      {isListLoading ? (
        <div className="flex justify-center py-8">
          <Loader2 className="animate-spin text-gray-400" size={24} />
        </div>
      ) : passkeys.length === 0 ? (
        <div className="flex flex-col items-center gap-3 rounded-xl border border-dashed border-gray-200 py-8 dark:border-gray-700">
          <KeyRound size={32} className="text-gray-300 dark:text-gray-600" />
          <p className="text-sm text-gray-400 dark:text-gray-500">
            パスキーは登録されていません。
          </p>
        </div>
      ) : (
        <motion.div
          className="space-y-2"
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.2 }}
        >
          {passkeys.map((pk) => (
            <div
              key={pk.id}
              className="flex items-center justify-between rounded-lg border border-gray-200 p-3 transition-colors hover:bg-gray-50 dark:border-gray-700 dark:hover:bg-gray-800/50"
            >
              <div className="flex items-center gap-3 flex-1">
                <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-blue-50 dark:bg-blue-900/20">
                  <KeyRound
                    size={14}
                    className="text-blue-600 dark:text-blue-400"
                  />
                </div>
                {editingId === pk.id ? (
                  <input
                    type="text"
                    value={editName}
                    onChange={(e) => setEditName(e.target.value)}
                    className="w-full max-w-[200px] rounded-md border border-gray-300 bg-transparent px-2 py-1 text-sm focus:border-blue-500 focus:outline-none dark:border-gray-600 dark:text-gray-100"
                    autoFocus
                    onKeyDown={(e) => {
                      if (e.key === "Enter") saveEdit(pk.id);
                      if (e.key === "Escape") cancelEdit();
                    }}
                  />
                ) : (
                  <div>
                    <div className="text-sm font-medium text-gray-900 dark:text-gray-100">
                      {pk.name}
                    </div>
                    <div className="text-xs text-gray-500 dark:text-gray-400">
                      登録日: {new Date(pk.createdAt).toLocaleDateString()}
                    </div>
                  </div>
                )}
              </div>
              <div className="flex items-center gap-1">
                {editingId === pk.id ? (
                  <>
                    <button
                      type="button"
                      onClick={() => saveEdit(pk.id)}
                      className="rounded-full p-1.5 text-green-600 transition-colors hover:bg-green-50 dark:text-green-400 dark:hover:bg-green-900/20"
                      title="保存"
                    >
                      <Check size={14} />
                    </button>
                    <button
                      type="button"
                      onClick={cancelEdit}
                      className="rounded-full p-1.5 text-gray-400 transition-colors hover:bg-gray-100 dark:hover:bg-gray-800"
                      title="キャンセル"
                    >
                      <X size={14} />
                    </button>
                  </>
                ) : (
                  <>
                    <button
                      type="button"
                      onClick={() => startEdit(pk)}
                      className="rounded-full p-1.5 text-gray-400 transition-colors hover:bg-gray-100 hover:text-gray-700 dark:hover:bg-gray-800 dark:hover:text-gray-200"
                      title="名前を変更"
                    >
                      <Pencil size={14} />
                    </button>
                    <button
                      type="button"
                      onClick={() => handleDelete(pk.id)}
                      className="rounded-full p-1.5 text-gray-400 transition-colors hover:bg-red-50 hover:text-red-500 dark:hover:bg-red-900/20 dark:hover:text-red-400"
                      title="削除"
                    >
                      <X size={14} />
                    </button>
                  </>
                )}
              </div>
            </div>
          ))}
        </motion.div>
      )}
    </div>
  );
}
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
