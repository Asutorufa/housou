import { useState } from "react";
import { useAuth } from "../contexts/AuthContext";
import type { UserStatus } from "../types";

interface UseAnimeStatusProps {
  title: string;
  initialStatus?: UserStatus;
  beginAt?: string;
  onUpdate?: () => void;
}

export function useAnimeStatus({
  title,
  initialStatus,
  beginAt,
  onUpdate,
}: UseAnimeStatusProps) {
  const { apiFetch } = useAuth();
  const [optimisticStatus, setOptimisticStatus] = useState<UserStatus | null>(
    null,
  );

  const currentStatus = optimisticStatus ?? initialStatus ?? 0;

  const persistItem = async (
    nextStatus: UserStatus,
    errorLabel: string,
  ): Promise<boolean> => {
    if (!title) return false;

    const previousStatus = optimisticStatus;
    setOptimisticStatus(nextStatus);

    // Convert ISO date string to Unix timestamp (milliseconds)
    const beginAtTs = beginAt ? new Date(beginAt).getTime() : undefined;

    try {
      await apiFetch("/api/user/item", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          title,
          status: nextStatus,
          begin_at: beginAtTs,
        }),
      });
      // Optionally notify parent to refresh list
      onUpdate?.();
      return true;
    } catch (err) {
      console.error(`Failed to ${errorLabel}`, err);
      // Revert on error
      setOptimisticStatus(previousStatus);
      return false;
    }
  };

  const updateStatus = async (statusString: string): Promise<boolean> => {
    const status = parseInt(statusString) as UserStatus;
    return persistItem(status, "update status");
  };

  return {
    currentStatus,
    updateStatus,
  };
}
