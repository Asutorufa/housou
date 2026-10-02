import { useState } from "react";
import { Check, Eraser, Star, X } from "lucide-react";
import { focusRingClassName } from "../../../styles/uiClasses";
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

export default function RatingEditor({
  score: currentScore,
  pending,
  onSave,
}: {
  score: number | null;
  pending: boolean;
  onSave: (score: number | null) => Promise<boolean>;
}) {
  const [ratingDraft, setRatingDraft] = useState<string | null>(null);
  const [ratingError, setRatingError] = useState<string | null>(null);
  const [isRatingEditing, setIsRatingEditing] = useState(false);
  const ratingInputValue = ratingDraft ?? currentScore?.toString() ?? "";
  const scoreTone = getScoreTone(currentScore);
  const scoreSummary = getScoreSummary(currentScore);
  const savedScoreLabel =
    currentScore !== null ? `${currentScore}/100` : "未評価";
  const scoreTitle = isRatingEditing ? "あなたの評価" : scoreSummary.title;
  const scoreDescription = isRatingEditing
    ? "1から100まで、あとで変更できます。"
    : scoreSummary.description;
  const startRatingEdit = () => {
    setRatingDraft(currentScore?.toString() ?? "");
    setRatingError(null);
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
    const score = Number(ratingInputValue);
    if (
      !ratingInputValue.trim() ||
      !Number.isInteger(score) ||
      score < 1 ||
      score > 100
    ) {
      setRatingError("1から100の整数を入力してください。");
      return;
    }
    if (await onSave(score)) cancelRatingEdit();
  };
  const handleClearRating = async () => {
    if (await onSave(null)) cancelRatingEdit();
  };
  return (
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
            <p className={`text-xs ${scoreTone.text}`}>{scoreDescription}</p>
          </div>
        </div>

        <button
          type="button"
          onClick={startRatingEdit}
          disabled={pending}
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
                pending ||
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
              disabled={pending || currentScore === null}
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
  );
}
