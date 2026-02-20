import { useState } from "react";
import { useAuth } from "../contexts/AuthContext";
import type { UserStatus } from "../types";

interface UseAnimeStatusProps {
  title: string;
  initialStatus?: UserStatus;
  initialScore?: number;
  beginAt?: string;
  onUpdate?: () => void;
}

export function useAnimeStatus({
  title,
  initialStatus,
  initialScore,
  beginAt,
  onUpdate,
}: UseAnimeStatusProps) {
  const { apiFetch } = useAuth();
  const [localStatus, setLocalStatus] = useState<UserStatus | null>(null);

  // Determine effective status: local state > initial status > default (0)
  const currentStatus = localStatus ?? initialStatus ?? 0;

  const updateStatus = async (statusString: string) => {
    if (!title) return;

    const status = parseInt(statusString) as UserStatus;
    const previousStatus = localStatus;

    // Optimistic update
    setLocalStatus(status);

    // Convert ISO date string to Unix timestamp (milliseconds)
    const beginAtTs = beginAt ? new Date(beginAt).getTime() : undefined;

    try {
      await apiFetch("/api/user/item", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          title,
          status,
          score: initialScore,
          begin_at: beginAtTs,
        }),
      });
      // Optionally notify parent to refresh list
      onUpdate?.();
    } catch (err) {
      console.error("Failed to update status", err);
      // Revert on error
      setLocalStatus(previousStatus);
    }
  };

  return {
    currentStatus,
    updateStatus,
  };
}
