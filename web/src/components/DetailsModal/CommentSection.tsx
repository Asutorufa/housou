import { useEffect, useState } from "react";
import useSWRInfinite from "swr/infinite";
import { useAuth } from "../../contexts/AuthContext";
import { focusRingClassName } from "../../styles/uiClasses";
import type { PaginatedComments } from "../../types";
import { MessageSquare, Send, Trash2, User as UserIcon, Edit2, X } from "lucide-react";

interface CommentSectionProps {
  title: string;
}

const PAGE_SIZE = 10;

export default function CommentSection({ title }: CommentSectionProps) {
  const { loggedIn, user } = useAuth();
  const [newComment, setNewComment] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isEditing, setIsEditing] = useState(false);

  const getKey = (pageIndex: number, previousPageData: PaginatedComments | null) => {
    if (previousPageData && !previousPageData.comments.length) return null;
    return `/api/comments?title=${encodeURIComponent(title)}&limit=${PAGE_SIZE}&offset=${pageIndex * PAGE_SIZE}`;
  };

  const { data, size, setSize, mutate, isValidating } = useSWRInfinite<PaginatedComments>(getKey);

  const comments = data ? data.flatMap((page) => page?.comments || []) : [];
  const total = data?.[0]?.total ?? 0;
  const hasMore = comments.length < total;

  // Check if current user already has a comment
  const userComment = comments.find((c) => c.userId === user?.id);

  useEffect(() => {
    if (userComment && !isEditing) {
      setNewComment(userComment.content);
    } else if (!userComment) {
      setNewComment("");
      setIsEditing(false);
    }
  }, [userComment, isEditing]);

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
        setIsEditing(false);
        mutate();
      }
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleDelete = async (id: number) => {
    if (!confirm("确定要删除这条评论吗？")) return;

    const resp = await fetch(`/api/comments/${id}`, { method: "DELETE" });
    if (resp.ok) {
      mutate();
    }
  };

  return (
    <div className="mt-8 space-y-6 border-t border-gray-100 pt-8 dark:border-gray-700">
      <div className="flex items-center gap-2">
        <MessageSquare className="text-blue-500" size={20} />
        <h3 className="text-lg font-black text-gray-900 dark:text-white">
          评论 ({total})
        </h3>
      </div>

      {/* Post/Edit Comment Form */}
      {loggedIn ? (
        (!userComment || isEditing) ? (
          <form onSubmit={handleSubmit} className="space-y-3">
            <div className="flex items-center justify-between">
                <span className="text-xs font-bold text-gray-500 uppercase">
                    {userComment ? "编辑评论" : "发表评论"}
                </span>
                {userComment && (
                    <button
                        type="button"
                        onClick={() => setIsEditing(false)}
                        className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200"
                    >
                        <X size={14} />
                    </button>
                )}
            </div>
            <textarea
              value={newComment}
              onChange={(e) => setNewComment(e.target.value)}
              placeholder="分享你的感悟或评价..."
              className="w-full rounded-2xl border border-gray-200 bg-gray-50 p-4 text-sm outline-none transition-all focus:border-blue-500 focus:ring-2 focus:ring-blue-500/20 dark:border-gray-700 dark:bg-gray-900/50 dark:text-white dark:focus:border-blue-400"
              rows={3}
            />
            <div className="flex justify-end">
              <button
                type="submit"
                disabled={isSubmitting || !newComment.trim() || (userComment && newComment === userComment.content)}
                className={`flex items-center gap-2 rounded-xl bg-blue-600 px-6 py-2 text-sm font-bold text-white transition-all hover:bg-blue-700 disabled:opacity-50 ${focusRingClassName}`}
              >
                <Send size={16} />
                {isSubmitting ? "发送中..." : userComment ? "更新" : "发布"}
              </button>
            </div>
          </form>
        ) : (
            <div className="rounded-2xl border border-blue-100 bg-blue-50/30 p-4 dark:border-blue-900/30 dark:bg-blue-900/10">
                <div className="mb-2 flex items-center justify-between">
                    <span className="text-xs font-bold text-blue-600 dark:text-blue-400 uppercase">
                        你的评论
                    </span>
                    <div className="flex gap-2">
                        <button
                            onClick={() => setIsEditing(true)}
                            className="text-gray-400 hover:text-blue-500"
                            title="编辑"
                        >
                            <Edit2 size={14} />
                        </button>
                        <button
                            onClick={() => handleDelete(userComment.id)}
                            className="text-gray-400 hover:text-red-500"
                            title="删除"
                        >
                            <Trash2 size={14} />
                        </button>
                    </div>
                </div>
                <p className="text-sm text-gray-700 dark:text-gray-300 whitespace-pre-wrap">
                    {userComment.content}
                </p>
            </div>
        )
      ) : (
        <div className="rounded-2xl bg-gray-50 p-6 text-center text-sm text-gray-500 dark:bg-gray-900/50">
          请先登录后发表评论。
        </div>
      )}

      {/* Comment List */}
      <div className="space-y-4">
        {comments.filter(c => c.userId !== user?.id).map((comment) => (
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
                  {comment.score !== undefined && comment.score !== null && (
                    <span className="rounded-md bg-yellow-100 px-1.5 py-0.5 text-[10px] font-black text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400">
                      ★ {comment.score}
                    </span>
                  )}
                  <span className="text-[10px] text-gray-400">
                    {new Date(comment.createdAt).toLocaleDateString()}
                  </span>
                </div>
              </div>
              <p className="whitespace-pre-wrap text-sm leading-relaxed text-gray-600 dark:text-gray-300">
                {comment.content}
              </p>
            </div>
          </div>
        ))}

        {hasMore && (
          <button
            onClick={() => setSize(size + 1)}
            disabled={isValidating}
            className="w-full rounded-xl border border-gray-200 py-3 text-sm font-bold text-gray-500 transition-colors hover:bg-gray-50 dark:border-gray-700 dark:text-gray-400 dark:hover:bg-gray-900/50"
          >
            {isValidating ? "加载中..." : "加载更多"}
          </button>
        )}

        {!isValidating && comments.length === 0 && (
          <div className="py-8 text-center text-sm text-gray-400">
            暂无评论。
          </div>
        )}
      </div>
    </div>
  );
}
