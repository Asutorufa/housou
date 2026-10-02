import { useState, type FormEvent } from "react";
import { Send, X } from "lucide-react";
import { focusRingClassName } from "../../../styles/uiClasses";
export default function CommentEditor({
  savedContent = "",
  pending,
  onSave,
  onCancel,
}: {
  savedContent?: string;
  pending: boolean;
  onSave: (content: string) => Promise<boolean>;
  onCancel: () => void;
}) {
  const [newComment, setNewComment] = useState(savedContent);
  const hasComment = !!savedContent.trim();
  const updateDraftComment = setNewComment;
  const stopEditing = onCancel;
  const handleSubmit = async (event: FormEvent) => {
    event.preventDefault();
    if (!newComment.trim() || pending) return;
    if (await onSave(newComment)) {
      setNewComment("");
      onCancel();
    }
  };
  return (
    <form onSubmit={handleSubmit} className="space-y-3">
      <div className="flex items-center justify-between">
        <span className="text-xs font-bold text-gray-500 uppercase">
          {hasComment ? "コメントを編集" : "新しいコメント"}
        </span>
        {hasComment && (
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
            pending ||
            !newComment.trim() ||
            (hasComment && newComment === savedContent)
          }
          className={`flex items-center gap-2 rounded-xl bg-blue-600 px-6 py-2 text-sm font-bold text-white transition-all hover:bg-blue-700 disabled:opacity-50 ${focusRingClassName}`}
        >
          <Send size={16} />
          {pending ? "送信中..." : hasComment ? "更新する" : "投稿する"}
        </button>
      </div>
    </form>
  );
}
