import { Check, KeyRound, Loader2, Pencil, X } from "lucide-react";
import { motion } from "motion/react";
import { useState } from "react";
import useSWR, { useSWRConfig } from "swr";
import { type PasskeySummary, useAuth } from "../../contexts/AuthContext";
import { fetcher } from "../../utils/fetcher";

export default function PasskeyTab() {
  const { registerPasskey, deletePasskey, renamePasskey } = useAuth();
  const { mutate } = useSWRConfig();
  const { data: passkeys = [], isLoading: isListLoading } = useSWR<
    PasskeySummary[]
  >("/api/auth/passkey", fetcher);

  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editName, setEditName] = useState("");

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
      await mutate("/api/auth/passkey");
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
      await mutate("/api/auth/passkey");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to rename passkey");
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm("このパスキーを削除してもよろしいですか？")) return;
    try {
      await deletePasskey(id);
      await mutate("/api/auth/passkey");
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
