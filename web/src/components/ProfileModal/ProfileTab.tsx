import { useEffect, useState } from "react";
import { formInputClassName } from "../../styles/uiClasses";
import type { User } from "../../types";

export default function ProfileTab({
  user,
  updateProfile,
}: {
  user: User;
  updateProfile: (data: {
    username: string;
    email: string;
    avatar_url: string;
  }) => Promise<unknown>;
}) {
  const [username, setUsername] = useState(user.username);
  const [email, setEmail] = useState(user.email);
  const [avatarUrl, setAvatarUrl] = useState(user.avatar_url || "");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  useEffect(() => {
    if (success) {
      const timer = setTimeout(() => setSuccess(false), 3000);
      return () => clearTimeout(timer);
    }
  }, [success]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setSuccess(false);
    setLoading(true);

    try {
      await updateProfile({ username, email, avatar_url: avatarUrl });
      setSuccess(true);
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
          className={formInputClassName}
          aria-label="メールアドレス"
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
          className={formInputClassName}
          aria-label="ユーザー名"
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
          className={formInputClassName}
          aria-label="アバター URL"
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
