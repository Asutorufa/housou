/* eslint-disable react-hooks/set-state-in-effect */
import { useCallback, useEffect, useState } from "react";
import { useAuth } from "../../contexts/AuthContext";
import type { User } from "../../types";

interface ReviewItem {
  id: number;
  title: string;
  userId: number;
  score: number | null;
  comment: string;
  createdAt: number;
  updatedAt: number;
  username: string;
  avatarUrl?: string;
}

const PAGE_SIZE = 10;

export default function ReviewSection({ title }: { title: string }) {
  const { apiFetch, loggedIn, user } = useAuth() as {
    apiFetch: typeof fetch;
    loggedIn: boolean;
    user: User | null;
  };
  const [reviews, setReviews] = useState<ReviewItem[]>([]);
  const [page, setPage] = useState(1);
  const [comment, setComment] = useState("");
  const [score, setScore] = useState<string>("");
  const myReview = reviews.find((r) => r.userId === user?.id);

  const load = useCallback(
    async (p: number) => {
      if (!title) return;
      const res = await apiFetch(
        `/api/reviews?title=${encodeURIComponent(title)}&page=${p}&page_size=${PAGE_SIZE}`,
      );
      const data = (await res.json()) as ReviewItem[];
      setReviews(data);
    },
    [apiFetch, title],
  );

  useEffect(() => {
    void load(page);
  }, [load, page]);

  useEffect(() => {
    if (!myReview) return;
    setComment(myReview.comment);
    setScore(typeof myReview.score === "number" ? String(myReview.score) : "");
  }, [myReview]);

  const submit = async () => {
    if (!comment.trim()) return;
    await apiFetch("/api/user/review", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        title,
        comment: comment.trim(),
        score: score && !Number.isNaN(Number(score)) ? Number(score) : null,
      }),
    });
    setComment("");
    setScore("");
    await load(1);
    setPage(1);
  };

  const remove = async () => {
    if (!myReview) return;
    await apiFetch("/api/user/review", {
      method: "DELETE",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ title }),
    });
    setComment("");
    setScore("");
    await load(1);
    setPage(1);
  };

  return (
    <div>
      <h4 className="mb-2 text-sm font-black tracking-wider text-gray-400 uppercase dark:text-gray-500">
        コメント
      </h4>
      {loggedIn && (
        <div className="mb-4 rounded-xl border border-gray-200 p-3 dark:border-gray-700">
          <div className="mb-2 text-xs text-gray-500">
            {user?.username} さんのコメント
          </div>
          <textarea
            className="mb-2 w-full rounded-md border p-2 text-sm"
            rows={3}
            value={comment}
            onChange={(e) => setComment(e.target.value)}
            placeholder="コメントを入力"
          />
          <div className="flex items-center gap-2">
            <input
              value={score}
              onChange={(e) => setScore(e.target.value)}
              className="w-24 rounded-md border p-1 text-sm"
              placeholder="スコア（任意）"
            />
            <button
              className="rounded-md bg-blue-600 px-3 py-1 text-white"
              onClick={submit}
            >
              {myReview ? "更新" : "投稿"}
            </button>
            {myReview && (
              <button
                className="rounded-md bg-red-600 px-3 py-1 text-white"
                onClick={remove}
              >
                削除
              </button>
            )}
          </div>
        </div>
      )}

      <div className="space-y-3">
        {reviews.map((r) => (
          <div
            key={r.id}
            className="rounded-xl border border-gray-200 p-3 dark:border-gray-700"
          >
            <div className="mb-1 flex items-center gap-2">
              <img
                src={r.avatarUrl || "https://placehold.co/32x32"}
                className="h-8 w-8 rounded-full"
                alt={r.username}
              />
              <span className="text-sm font-semibold">{r.username}</span>
              {typeof r.score === "number" && (
                <span className="text-xs text-amber-600">
                  スコア: {r.score}
                </span>
              )}
            </div>
            <p className="text-sm text-gray-700 dark:text-gray-200">
              {r.comment}
            </p>
          </div>
        ))}
      </div>

      <div className="mt-3 flex gap-2">
        <button
          className="rounded border px-2 py-1 text-sm disabled:opacity-50"
          disabled={page <= 1}
          onClick={() => setPage((p) => Math.max(1, p - 1))}
        >
          前へ
        </button>
        <button
          className="rounded border px-2 py-1 text-sm disabled:opacity-50"
          disabled={reviews.length < PAGE_SIZE}
          onClick={() => setPage((p) => p + 1)}
        >
          次へ
        </button>
      </div>
    </div>
  );
}
