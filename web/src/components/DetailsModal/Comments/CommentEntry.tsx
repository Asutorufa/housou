import { Edit2, Trash2, User as UserIcon } from "lucide-react";
import {
  USER_STATUS_LABELS,
  type CommentWithUser,
  type UserStatus,
} from "../../../types";
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

interface CommentEntryProps {
  comment: CommentWithUser;
  viewerStatus?: UserStatus;
  onEdit?: () => void;
  onDelete?: () => void;
  disabled?: boolean;
}
export default function CommentEntry({
  comment,
  viewerStatus = comment.status,
  onEdit,
  onDelete,
  disabled,
}: CommentEntryProps) {
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

  if (onEdit)
    return (
      <div className="rounded-2xl border border-blue-100 bg-blue-50/30 p-4 dark:border-blue-900/30 dark:bg-blue-900/10">
        <div className="mb-2 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <span className="text-xs font-bold text-blue-600 dark:text-blue-400 uppercase">
              あなたのコメント
            </span>
            {renderCommentStatus(viewerStatus)}
            {renderCommentScore(comment.score)}
          </div>
          <div className="flex gap-2">
            <button
              onClick={onEdit}
              disabled={disabled}
              className="text-gray-400 hover:text-blue-500"
              title="編集"
            >
              <Edit2 size={14} />
            </button>
            <button
              onClick={onDelete}
              disabled={disabled}
              className="text-gray-400 hover:text-red-500"
              title="削除"
            >
              <Trash2 size={14} />
            </button>
          </div>
        </div>
        {renderCommentMeta(comment.createdAt, comment.updatedAt)}
        <p className="text-sm whitespace-pre-wrap text-gray-700 dark:text-gray-300">
          {comment.content}
        </p>
      </div>
    );
  return (
    <div className="group relative flex gap-4 rounded-2xl bg-gray-50/50 p-4 transition-colors hover:bg-gray-50 dark:bg-gray-900/30 dark:hover:bg-gray-900/50">
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
  );
}
