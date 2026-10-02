import { useRef, useState } from "react";
import { useAuth } from "../contexts/AuthContext";
import type { UserStatus } from "../types";
import { checkResponse } from "../utils/fetcher";

interface UseAnimeStatusProps {
  title: string;
  initialStatus?: UserStatus;
  beginAt?: string;
  onUpdate?: () => void | Promise<unknown>;
}
export function useAnimeStatus({
  title,
  initialStatus = 0,
  beginAt,
  onUpdate,
}: UseAnimeStatusProps) {
  const { apiFetch, user } = useAuth();
  const owner = `${user?.id ?? "guest"}:${title}`;
  const [optimistic, setOptimistic] = useState<{
    owner: string;
    base: UserStatus;
    status: UserStatus;
  } | null>(null);
  const [updating, setUpdating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const pending = useRef(false);
  const currentStatus =
    optimistic?.owner === owner && optimistic.base === initialStatus
      ? optimistic.status
      : initialStatus;

  const updateStatus = async (value: string): Promise<boolean> => {
    const status = Number(value);
    if (
      !title ||
      pending.current ||
      !Number.isInteger(status) ||
      status < 0 ||
      status > 5
    )
      return false;
    pending.current = true;
    setUpdating(true);
    setError(null);
    const previous = optimistic;
    setOptimistic({ owner, base: initialStatus, status: status as UserStatus });
    const beginAtTs = beginAt ? new Date(beginAt).getTime() : undefined;
    try {
      await checkResponse(
        await apiFetch("/api/user/item", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            title,
            status,
            begin_at: Number.isFinite(beginAtTs) ? beginAtTs : undefined,
          }),
        }),
      );
    } catch (err) {
      setOptimistic(previous);
      setError(
        err instanceof Error ? err.message : "状態の更新に失敗しました。",
      );
      return false;
    } finally {
      pending.current = false;
      setUpdating(false);
    }
    // A refresh failure must not undo a write that the server already accepted.
    try {
      await onUpdate?.();
    } catch {
      setError("保存しましたが、一覧を更新できませんでした。");
    }
    return true;
  };
  return { currentStatus, updateStatus, updating, error };
}
