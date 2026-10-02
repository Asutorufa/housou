import { MessageSquare } from "lucide-react";
import { useState } from "react";
import { useAuth } from "../../contexts/AuthContext";
import type { UserStatus } from "../../types";
import CommentEditor from "./Comments/CommentEditor";
import CommentEntry from "./Comments/CommentEntry";
import RatingEditor from "./Comments/RatingEditor";
import { useComments } from "./Comments/useComments";

interface CommentSectionProps {
  title: string;
  viewerStatus?: UserStatus;
}
export default function CommentSection(props: CommentSectionProps) {
  const { user } = useAuth();
  return (
    <CommentsContent key={`${user?.id ?? "guest"}:${props.title}`} {...props} />
  );
}
function CommentsContent({ title, viewerStatus = 0 }: CommentSectionProps) {
  const { user, loggedIn } = useAuth();
  const {
    comments,
    userComment,
    total,
    pending,
    isLoading,
    isValidating,
    hasMore,
    loadMore,
    retry,
    error,
    saveComment,
    saveRating,
    deleteComment,
  } = useComments(title);
  const [editing, setEditing] = useState(false);
  const ownComment = userComment?.content.trim() ? userComment : undefined;

  async function remove() {
    if (!ownComment || !confirm("コメントを削除しますか？")) return;
    if (await deleteComment(ownComment.id)) setEditing(false);
  }
  return (
    <div className="mt-8 space-y-6 border-t border-gray-100 pt-8 dark:border-gray-700">
      <div className="flex items-center gap-2">
        <MessageSquare className="text-blue-500" size={20} />
        <h3 className="text-lg font-black text-gray-900 dark:text-white">
          コメント ({total})
        </h3>
      </div>
      {error && (
        <div role="alert" className="text-sm text-red-500">
          {error}{" "}
          <button onClick={() => void retry()} className="underline">
            再試行
          </button>
        </div>
      )}
      {loggedIn ? (
        <>
          <RatingEditor
            score={userComment?.score ?? null}
            pending={pending || isLoading}
            onSave={saveRating}
          />
          {!ownComment || editing ? (
            <CommentEditor
              savedContent={ownComment?.content}
              pending={pending || isLoading}
              onSave={saveComment}
              onCancel={() => setEditing(false)}
            />
          ) : (
            <CommentEntry
              comment={ownComment}
              viewerStatus={viewerStatus}
              disabled={pending}
              onEdit={() => setEditing(true)}
              onDelete={() => void remove()}
            />
          )}
        </>
      ) : (
        <div className="rounded-2xl bg-gray-50 p-6 text-center text-sm text-gray-500 dark:bg-gray-900/50">
          コメントを投稿するにはログインが必要です。
        </div>
      )}
      <div className="space-y-4">
        {comments
          .filter((comment) => comment.userId !== user?.id)
          .map((comment) => (
            <CommentEntry key={comment.id} comment={comment} />
          ))}
        {hasMore && (
          <button
            onClick={() => void loadMore()}
            disabled={isValidating}
            className="w-full rounded-xl border border-gray-200 py-3 text-sm font-bold text-gray-500 transition-colors hover:bg-gray-50 dark:border-gray-700 dark:text-gray-400 dark:hover:bg-gray-900/50"
          >
            {isValidating ? "読み込み中..." : "さらに読み込む"}
          </button>
        )}
        {isLoading && (
          <p role="status" className="text-center text-sm text-gray-400">
            読み込み中...
          </p>
        )}
        {!isLoading && !error && comments.length === 0 && (
          <div className="py-8 text-center text-sm text-gray-400">
            まだコメントはありません。
          </div>
        )}
      </div>
    </div>
  );
}
