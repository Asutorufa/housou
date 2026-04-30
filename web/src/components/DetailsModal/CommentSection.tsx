import { useState } from "react";
import useSWRInfinite from "swr/infinite";
import { useAuth } from "../../contexts/AuthContext";
import { focusRingClassName } from "../../styles/uiClasses";
import {
  USER_STATUS_LABELS,
  type PaginatedComments,
  type UserStatus,
} from "../../types";
import { fetcher } from "../../utils/fetcher";
import {
  MessageSquare,
  Send,
  Trash2,
  User as UserIcon,
  Check,
  Edit2,
  Eraser,
  Star,
  X,
} from "lucide-react";

interface CommentSectionProps {
  title: string;
  viewerStatus?: UserStatus;
}

const PAGE_SIZE = 10;
const COMMENT_REFRESH_INTERVAL = 30_000;

const commentsFetcher = (url: string) =>
  fetcher(url, {
    cache: "no-store",
    headers: { "Cache-Control": "no-cache" },
  });

function formatCommentTime(timestamp: number) {
  return new Date(timestamp).toLocaleString();
}

function formatCommentScore(score: number) {
  return `${score}/100`;
}

function getCommentScoreClassName(score: number) {
  if (score >= 85) {
    return "bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-300";
  }

  if (score >= 70) {
    return "bg-sky-100 text-sky-700 dark:bg-sky-900/30 dark:text-sky-300";
  }

  if (score >= 50) {
    return "bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-300";
  }

  return "bg-rose-100 text-rose-700 dark:bg-rose-900/30 dark:text-rose-300";
}

function getCommentStatusClassName(status: UserStatus) {
  switch (status) {
    case 1:
      return "bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-300";
    case 2:
      return "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-300";
    case 3:
      return "bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-300";
    case 4:
      return "bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-300";
    case 5:
      return "bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-300";
    default:
      return "bg-gray-100 text-gray-500 dark:bg-gray-800 dark:text-gray-400";
  }
}

function getScoreTone(score: number | null) {
  if (score === null) {
    return {
      card: "border-gray-200/70 bg-gray-50/70 dark:border-gray-700/70 dark:bg-gray-900/25",
      icon: "bg-white text-gray-400 shadow-sm dark:bg-gray-800 dark:text-gray-500",
      title: "text-gray-900 dark:text-white",
      text: "text-gray-500 dark:text-gray-400",
      badge:
        "border-gray-200 bg-white text-gray-500 dark:border-gray-700 dark:bg-gray-800 dark:text-gray-400",
      progress: "bg-gray-300 dark:bg-gray-600",
      button:
        "border-gray-200 bg-white text-gray-700 hover:border-gray-300 hover:bg-gray-50 dark:border-gray-700 dark:bg-gray-800 dark:text-gray-200 dark:hover:border-gray-600 dark:hover:bg-gray-700",
      glow: "shadow-sm",
    };
  }

  if (score >= 85) {
    return {
      card: "border-emerald-200/80 bg-emerald-50/55 dark:border-emerald-900/50 dark:bg-emerald-950/20",
      icon: "bg-emerald-100 text-emerald-700 shadow-emerald-900/5 dark:bg-emerald-900/40 dark:text-emerald-300",
      title: "text-emerald-950 dark:text-emerald-50",
      text: "text-emerald-700 dark:text-emerald-300",
      badge:
        "border-emerald-200 bg-white/80 text-emerald-700 dark:border-emerald-800 dark:bg-emerald-950/40 dark:text-emerald-300",
      progress: "bg-emerald-500 dark:bg-emerald-400",
      button:
        "border-emerald-200 bg-emerald-100 text-emerald-800 hover:border-emerald-300 hover:bg-emerald-200/80 dark:border-emerald-800 dark:bg-emerald-900/35 dark:text-emerald-200 dark:hover:bg-emerald-900/55",
      glow: "shadow-sm shadow-emerald-900/5",
    };
  }

  if (score >= 70) {
    return {
      card: "border-sky-200/80 bg-sky-50/55 dark:border-sky-900/50 dark:bg-sky-950/20",
      icon: "bg-sky-100 text-sky-700 shadow-sky-900/5 dark:bg-sky-900/40 dark:text-sky-300",
      title: "text-sky-950 dark:text-sky-50",
      text: "text-sky-700 dark:text-sky-300",
      badge:
        "border-sky-200 bg-white/80 text-sky-700 dark:border-sky-800 dark:bg-sky-950/40 dark:text-sky-300",
      progress: "bg-sky-500 dark:bg-sky-400",
      button:
        "border-sky-200 bg-sky-100 text-sky-800 hover:border-sky-300 hover:bg-sky-200/80 dark:border-sky-800 dark:bg-sky-900/35 dark:text-sky-200 dark:hover:bg-sky-900/55",
      glow: "shadow-sm shadow-sky-900/5",
    };
  }

  if (score >= 50) {
    return {
      card: "border-amber-200/80 bg-amber-50/60 dark:border-amber-900/50 dark:bg-amber-950/20",
      icon: "bg-amber-100 text-amber-700 shadow-amber-900/5 dark:bg-amber-900/40 dark:text-amber-300",
      title: "text-amber-950 dark:text-amber-50",
      text: "text-amber-700 dark:text-amber-300",
      badge:
        "border-amber-200 bg-white/80 text-amber-700 dark:border-amber-800 dark:bg-amber-950/40 dark:text-amber-300",
      progress: "bg-amber-500 dark:bg-amber-400",
      button:
        "border-amber-200 bg-amber-100 text-amber-800 hover:border-amber-300 hover:bg-amber-200/80 dark:border-amber-800 dark:bg-amber-900/35 dark:text-amber-200 dark:hover:bg-amber-900/55",
      glow: "shadow-sm shadow-amber-900/5",
    };
  }

  return {
    card: "border-rose-200/80 bg-rose-50/55 dark:border-rose-900/50 dark:bg-rose-950/20",
    icon: "bg-rose-100 text-rose-700 shadow-rose-900/5 dark:bg-rose-900/40 dark:text-rose-300",
    title: "text-rose-950 dark:text-rose-50",
    text: "text-rose-700 dark:text-rose-300",
    badge:
      "border-rose-200 bg-white/80 text-rose-700 dark:border-rose-800 dark:bg-rose-950/40 dark:text-rose-300",
    progress: "bg-rose-500 dark:bg-rose-400",
    button:
      "border-rose-200 bg-rose-100 text-rose-800 hover:border-rose-300 hover:bg-rose-200/80 dark:border-rose-800 dark:bg-rose-900/35 dark:text-rose-200 dark:hover:bg-rose-900/55",
    glow: "shadow-sm shadow-rose-900/5",
  };
}

function getScoreSummary(score: number | null) {
  if (score === null) {
    return {
      title: "まだ評価していません",
      description: "右のスコアを押して評価できます。",
    };
  }

  if (score >= 90) {
    return { title: "素晴らしい", description: "かなり刺さった作品ですね。" };
  }

  if (score >= 80) {
    return { title: "とても良い", description: "満足度の高い一本です。" };
  }

  if (score >= 70) {
    return { title: "良い", description: "しっかり楽しめた評価です。" };
  }

  if (score >= 50) {
    return {
      title: "まずまず",
      description: "好みは分かれつつも悪くない感じ。",
    };
  }

  return {
    title: "合わなかったかも",
    description: "次はもっと刺さる作品に出会えますように。",
  };
}

export default function CommentSection({
  title,
  viewerStatus = 0,
}: CommentSectionProps) {
  const { loggedIn, user } = useAuth();
  const [draftComment, setDraftComment] = useState<{
    title: string;
    value: string;
  }>({
    title,
    value: "",
  });
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isEditing, setIsEditing] = useState(false);
  const [ratingDraft, setRatingDraft] = useState<string | null>(null);
  const [ratingError, setRatingError] = useState<string | null>(null);
  const [isRatingEditing, setIsRatingEditing] = useState(false);
  const [optimisticScore, setOptimisticScore] = useState<{
    title: string;
    value: number | null;
  } | null>(null);

  const getKey = (
    pageIndex: number,
    previousPageData: PaginatedComments | null,
  ) => {
    if (previousPageData && !previousPageData.comments.length) return null;
    const viewerKey = user?.id ?? "guest";
    return `/api/comments?title=${encodeURIComponent(title)}&limit=${PAGE_SIZE}&offset=${pageIndex * PAGE_SIZE}&viewer=${viewerKey}`;
  };

  const { data, size, setSize, mutate, isValidating } =
    useSWRInfinite<PaginatedComments>(getKey, commentsFetcher, {
      refreshInterval: COMMENT_REFRESH_INTERVAL,
      revalidateOnFocus: true,
      revalidateFirstPage: true,
    });

  const comments = data ? data.flatMap((page) => page?.comments || []) : [];
  const total = data?.[0]?.total ?? 0;
  const hasMore = comments.length < total;

  // Check if current user already has a comment
  const userComment = comments.find((c) => c.userId === user?.id);
  const hasUserCommentContent = !!userComment?.content.trim();
  const visibleUserComment = hasUserCommentContent ? userComment : null;
  const currentScore =
    optimisticScore?.title === title
      ? optimisticScore.value
      : (userComment?.score ?? null);
  const ratingInputValue = ratingDraft ?? currentScore?.toString() ?? "";
  const scoreTone = getScoreTone(currentScore);
  const scoreSummary = getScoreSummary(currentScore);
  const savedScoreLabel =
    currentScore !== null ? `${currentScore}/100` : "未評価";
  const scoreTitle = isRatingEditing ? "あなたの評価" : scoreSummary.title;
  const scoreDescription = isRatingEditing
    ? "1から100まで、あとで変更できます。"
    : scoreSummary.description;
  const newComment = draftComment.title === title ? draftComment.value : "";

  const updateDraftComment = (value: string) => {
    setDraftComment({ title, value });
  };

  const resetDraftComment = () => {
    setDraftComment({ title, value: "" });
  };

  const startEditing = () => {
    setDraftComment({
      title,
      value: userComment?.content ?? "",
    });
    setIsEditing(true);
  };

  const stopEditing = () => {
    resetDraftComment();
    setIsEditing(false);
  };

  const startRatingEdit = () => {
    setRatingError(null);
    setRatingDraft(currentScore?.toString() ?? "");
    setIsRatingEditing(true);
  };

  const cancelRatingEdit = () => {
    setRatingError(null);
    setRatingDraft(null);
    setIsRatingEditing(false);
  };

  const handleRatingChange = (value: string) => {
    setRatingError(null);
    setRatingDraft(value);
  };

  const handleSaveRating = async () => {
    const trimmedValue = ratingInputValue.trim();
    const parsedValue = Number(trimmedValue);

    if (!trimmedValue) {
      setRatingError("1から100の整数を入力してください。");
      return;
    }

    if (
      !Number.isInteger(parsedValue) ||
      parsedValue < 1 ||
      parsedValue > 100
    ) {
      setRatingError("评分は1から100の整数で入力してください。");
      return;
    }

    const resp = await fetch("/api/comments", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ title, score: parsedValue }),
    });

    if (resp.ok) {
      setOptimisticScore({ title, value: parsedValue });
      setRatingDraft(parsedValue.toString());
      setRatingError(null);
      setIsRatingEditing(false);
      mutate();
    }
  };

  const handleClearRating = async () => {
    const resp = await fetch("/api/comments", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ title, score: null }),
    });

    if (resp.ok) {
      setOptimisticScore({ title, value: null });
      setRatingDraft("");
      setRatingError(null);
      setIsRatingEditing(false);
      mutate();
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newComment.trim() || isSubmitting) return;

    setIsSubmitting(true);
    try {
      const resp = await fetch("/api/comments", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ title, content: newComment }),
      });

      if (resp.ok) {
        stopEditing();
        mutate();
      }
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleDelete = async (id: number) => {
    if (!confirm("コメントを削除しますか？")) return;

    const resp = await fetch(`/api/comments/${id}`, { method: "DELETE" });
    if (resp.ok) {
      resetDraftComment();
      setIsEditing(false);
      mutate();
    }
  };

  const renderCommentMeta = (createdAt: number, updatedAt: number) => {
    const isEdited = updatedAt > createdAt;

    return (
      <div className="flex flex-wrap items-center gap-x-2 gap-y-1 text-[10px] text-gray-400">
        <span title={formatCommentTime(updatedAt)}>
          更新: {new Date(updatedAt).toLocaleDateString()}
        </span>
        {isEdited && (
          <span title={formatCommentTime(createdAt)}>
            投稿日: {new Date(createdAt).toLocaleDateString()}
          </span>
        )}
      </div>
    );
  };

  const renderCommentScore = (score?: number | null) => {
    if (score === undefined || score === null) return null;

    return (
      <span
        className={`rounded-md px-1.5 py-0.5 text-[10px] font-black ${getCommentScoreClassName(score)}`}
      >
        {formatCommentScore(score)}
      </span>
    );
  };

  const renderCommentStatus = (status: UserStatus) => {
    if (status === 0) return null;

    return (
      <span
        className={`rounded-md px-1.5 py-0.5 text-[10px] font-black ${getCommentStatusClassName(status)}`}
      >
        {USER_STATUS_LABELS[status]}
      </span>
    );
  };

  return (
    <div className="mt-8 space-y-6 border-t border-gray-100 pt-8 dark:border-gray-700">
      <div className="flex items-center gap-2">
        <MessageSquare className="text-blue-500" size={20} />
        <h3 className="text-lg font-black text-gray-900 dark:text-white">
          コメント ({total})
        </h3>
      </div>

      {loggedIn && (
        <section
          className={`overflow-hidden rounded-2xl border p-4 transition-all duration-300 ${scoreTone.card} ${scoreTone.glow}`}
        >
          <div className="flex flex-wrap items-start justify-between gap-3">
            <div className="flex items-center gap-3">
              <div
                className={`flex h-9 w-9 items-center justify-center rounded-2xl transition-colors ${scoreTone.icon}`}
              >
                <Star
                  size={17}
                  className={currentScore !== null ? "fill-current" : ""}
                />
              </div>
              <div>
                <h4 className={`text-sm font-black ${scoreTone.title}`}>
                  {scoreTitle}
                </h4>
                <p className={`text-xs ${scoreTone.text}`}>
                  {scoreDescription}
                </p>
              </div>
            </div>

            <button
              type="button"
              onClick={startRatingEdit}
              aria-expanded={isRatingEditing}
              className={`rounded-full border px-3 py-1 text-xs font-black transition-all hover:-translate-y-0.5 hover:shadow-sm ${scoreTone.badge} ${focusRingClassName}`}
            >
              {savedScoreLabel}
            </button>
          </div>

          <div className="mt-4 h-1.5 overflow-hidden rounded-full bg-white/80 ring-1 ring-black/5 dark:bg-gray-950/40 dark:ring-white/10">
            <div
              className={`h-full rounded-full transition-all duration-500 ease-out ${scoreTone.progress}`}
              style={{ width: `${currentScore ?? 0}%` }}
            />
          </div>

          {isRatingEditing && (
            <div className="mt-4 grid gap-2 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-center">
              <label className="relative block">
                <span className="sr-only">評価スコア</span>
                <input
                  type="number"
                  min={1}
                  max={100}
                  step={1}
                  inputMode="numeric"
                  value={ratingInputValue}
                  onChange={(e) => handleRatingChange(e.target.value)}
                  placeholder="1-100"
                  className={`w-full rounded-xl border border-white/80 bg-white/80 px-3 py-2 pr-14 text-sm font-bold text-gray-900 shadow-sm outline-none transition-all placeholder:text-gray-400 focus:border-blue-300 focus:bg-white focus:ring-2 focus:ring-blue-500/15 dark:border-gray-700/80 dark:bg-gray-900/70 dark:text-gray-100 dark:focus:border-blue-500/70 dark:focus:bg-gray-900 ${focusRingClassName}`}
                  autoFocus
                />
                <span className="pointer-events-none absolute top-1/2 right-3 -translate-y-1/2 text-xs font-black text-gray-400">
                  /100
                </span>
              </label>

              <div className="flex gap-2">
                <button
                  type="button"
                  onClick={handleSaveRating}
                  disabled={
                    !ratingInputValue.trim() ||
                    ratingInputValue === (currentScore?.toString() ?? "")
                  }
                  className={`inline-flex items-center justify-center gap-1.5 rounded-xl border px-4 py-2 text-sm font-black transition-all hover:-translate-y-0.5 disabled:pointer-events-none disabled:translate-y-0 disabled:opacity-45 ${scoreTone.button} ${focusRingClassName}`}
                >
                  <Check size={15} />
                  {currentScore !== null ? "更新" : "保存"}
                </button>
                <button
                  type="button"
                  onClick={handleClearRating}
                  disabled={currentScore === null}
                  className={`inline-flex items-center justify-center gap-1.5 rounded-xl border border-transparent px-3 py-2 text-sm font-bold text-gray-500 transition-all hover:bg-white/70 hover:text-gray-700 disabled:pointer-events-none disabled:opacity-40 dark:text-gray-400 dark:hover:bg-gray-800/70 dark:hover:text-gray-200 ${focusRingClassName}`}
                >
                  <Eraser size={15} />
                  クリア
                </button>
                <button
                  type="button"
                  onClick={cancelRatingEdit}
                  className={`inline-flex items-center justify-center rounded-xl p-2 text-gray-400 transition-colors hover:bg-white/70 hover:text-gray-700 dark:hover:bg-gray-800/70 dark:hover:text-gray-200 ${focusRingClassName}`}
                  aria-label="評価編集を閉じる"
                >
                  <X size={16} />
                </button>
              </div>

              {ratingError && (
                <p className="text-xs font-bold text-red-600 sm:col-span-2 dark:text-red-400">
                  {ratingError}
                </p>
              )}
            </div>
          )}
        </section>
      )}

      {/* Post/Edit Comment Form */}
      {loggedIn ? (
        !visibleUserComment || isEditing ? (
          <form onSubmit={handleSubmit} className="space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-xs font-bold text-gray-500 uppercase">
                {hasUserCommentContent ? "コメントを編集" : "新しいコメント"}
              </span>
              {hasUserCommentContent && (
                <button
                  type="button"
                  onClick={stopEditing}
                  className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200"
                >
                  <X size={14} />
                </button>
              )}
            </div>
            <textarea
              value={newComment}
              onChange={(e) => updateDraftComment(e.target.value)}
              placeholder="感想や評価を共有しましょう..."
              className="w-full rounded-2xl border border-gray-200 bg-gray-50 p-4 text-sm outline-none transition-all focus:border-blue-500 focus:ring-2 focus:ring-blue-500/20 dark:border-gray-700 dark:bg-gray-900/50 dark:text-white dark:focus:border-blue-400"
              rows={3}
            />
            <div className="flex justify-end">
              <button
                type="submit"
                disabled={
                  isSubmitting ||
                  !newComment.trim() ||
                  (hasUserCommentContent && newComment === userComment?.content)
                }
                className={`flex items-center gap-2 rounded-xl bg-blue-600 px-6 py-2 text-sm font-bold text-white transition-all hover:bg-blue-700 disabled:opacity-50 ${focusRingClassName}`}
              >
                <Send size={16} />
                {isSubmitting
                  ? "送信中..."
                  : hasUserCommentContent
                    ? "更新する"
                    : "投稿する"}
              </button>
            </div>
          </form>
        ) : (
          <div className="rounded-2xl border border-blue-100 bg-blue-50/30 p-4 dark:border-blue-900/30 dark:bg-blue-900/10">
            <div className="mb-2 flex items-center justify-between">
              <div className="flex items-center gap-2">
                <span className="text-xs font-bold text-blue-600 dark:text-blue-400 uppercase">
                  あなたのコメント
                </span>
                {renderCommentStatus(viewerStatus)}
                {renderCommentScore(visibleUserComment.score)}
              </div>
              <div className="flex gap-2">
                <button
                  onClick={startEditing}
                  className="text-gray-400 hover:text-blue-500"
                  title="編集"
                >
                  <Edit2 size={14} />
                </button>
                <button
                  onClick={() => handleDelete(visibleUserComment.id)}
                  className="text-gray-400 hover:text-red-500"
                  title="削除"
                >
                  <Trash2 size={14} />
                </button>
              </div>
            </div>
            {renderCommentMeta(
              visibleUserComment.createdAt,
              visibleUserComment.updatedAt,
            )}
            <p className="text-sm whitespace-pre-wrap text-gray-700 dark:text-gray-300">
              {visibleUserComment.content}
            </p>
          </div>
        )
      ) : (
        <div className="rounded-2xl bg-gray-50 p-6 text-center text-sm text-gray-500 dark:bg-gray-900/50">
          コメントを投稿するにはログインが必要です。
        </div>
      )}

      {/* Comment List */}
      <div className="space-y-4">
        {comments
          .filter((c) => c.userId !== user?.id)
          .map((comment) => (
            <div
              key={comment.id}
              className="group relative flex gap-4 rounded-2xl bg-gray-50/50 p-4 transition-colors hover:bg-gray-50 dark:bg-gray-900/30 dark:hover:bg-gray-900/50"
            >
              <div className="h-10 w-10 flex-shrink-0 overflow-hidden rounded-full bg-gray-200 dark:bg-gray-700">
                {comment.avatarUrl ? (
                  <img
                    src={comment.avatarUrl}
                    alt={comment.username}
                    className="h-full w-full object-cover"
                  />
                ) : (
                  <div className="flex h-full w-full items-center justify-center text-gray-400">
                    <UserIcon size={20} />
                  </div>
                )}
              </div>
              <div className="flex-1 space-y-1">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-bold text-gray-900 dark:text-white">
                      {comment.username}
                    </span>
                    {renderCommentStatus(comment.status)}
                    {renderCommentScore(comment.score)}
                  </div>
                </div>
                {renderCommentMeta(comment.createdAt, comment.updatedAt)}
                {comment.content.trim() ? (
                  <p className="whitespace-pre-wrap text-sm leading-relaxed text-gray-600 dark:text-gray-300">
                    {comment.content}
                  </p>
                ) : (
                  <p className="text-sm text-gray-400 italic dark:text-gray-500">
                    {comment.score !== null || comment.status !== 0
                      ? "記録のみ"
                      : "コメントなし"}
                  </p>
                )}
              </div>
            </div>
          ))}

        {hasMore && (
          <button
            onClick={() => setSize(size + 1)}
            disabled={isValidating}
            className="w-full rounded-xl border border-gray-200 py-3 text-sm font-bold text-gray-500 transition-colors hover:bg-gray-50 dark:border-gray-700 dark:text-gray-400 dark:hover:bg-gray-900/50"
          >
            {isValidating ? "読み込み中..." : "さらに読み込む"}
          </button>
        )}

        {!isValidating && comments.length === 0 && (
          <div className="py-8 text-center text-sm text-gray-400">
            まだコメントはありません。
          </div>
        )}
      </div>
    </div>
  );
}
