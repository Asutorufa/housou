import { useMemo, useRef, useState } from "react";
import useSWRInfinite from "swr/infinite";
import { useAuth } from "../../../contexts/AuthContext";
import type { PaginatedComments } from "../../../types";
import { checkResponse, fetcher } from "../../../utils/fetcher";

const PAGE_SIZE = 10;
const commentsFetcher = (url: string) =>
  fetcher<PaginatedComments>(url, { cache: "no-store" });

export function useComments(title: string) {
  const { user, apiFetch } = useAuth();
  const {
    data,
    size,
    setSize,
    mutate,
    isLoading,
    isValidating,
    error: loadError,
  } = useSWRInfinite<PaginatedComments>(
    (page, previous) => {
      if (previous && previous.comments.length < PAGE_SIZE) return null;
      return `/api/comments?${new URLSearchParams({ title, limit: String(PAGE_SIZE), offset: String(page * PAGE_SIZE), viewer: String(user?.id ?? "guest") })}`;
    },
    commentsFetcher,
    { refreshInterval: 30_000, revalidateAll: true },
  );
  const comments = useMemo(
    () =>
      Array.from(
        new Map(
          data
            ?.flatMap((page) => page.comments)
            .map((comment) => [comment.id, comment]),
        ).values(),
      ),
    [data],
  );
  const total = data?.[0]?.total ?? 0;
  const userComment = comments.find((comment) => comment.userId === user?.id);
  const [pending, setPending] = useState(false);
  const [writeError, setWriteError] = useState<string | null>(null);
  const busy = useRef(false);

  async function write(
    body: { content?: string; score?: number | null } | { deleteId: number },
  ): Promise<boolean> {
    if (!user || busy.current) return false;
    busy.current = true;
    setPending(true);
    setWriteError(null);
    try {
      const deleting = "deleteId" in body;
      await checkResponse(
        await apiFetch(
          deleting ? `/api/comments/${body.deleteId}` : "/api/comments",
          {
            method: deleting ? "DELETE" : "POST",
            ...(deleting
              ? {}
              : {
                  headers: { "Content-Type": "application/json" },
                  body: JSON.stringify({ title, ...body }),
                }),
          },
        ),
      );
      try {
        await mutate();
      } catch {
        setWriteError("保存しましたが、一覧を更新できませんでした。");
      }
      return true;
    } catch (error) {
      setWriteError(
        error instanceof Error ? error.message : "保存に失敗しました。",
      );
      return false;
    } finally {
      busy.current = false;
      setPending(false);
    }
  }
  return {
    comments,
    userComment,
    total,
    pending,
    isLoading,
    isValidating,
    hasMore:
      (data?.reduce((count, page) => count + page.comments.length, 0) ?? 0) <
      total,
    loadMore: () => setSize(size + 1),
    retry: () => mutate(),
    error:
      writeError ||
      (loadError instanceof Error
        ? loadError.message
        : loadError
          ? "読み込めませんでした。"
          : null),
    saveComment: (content: string) => write({ content }),
    saveRating: (score: number | null) => write({ score }),
    deleteComment: (id: number) => write({ deleteId: id }),
  };
}
