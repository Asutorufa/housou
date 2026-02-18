import { renderHook, act } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useAnimeStatus } from "./useAnimeStatus";

// Mock useAuth
const mockApiFetch = vi.fn();
vi.mock("../contexts/AuthContext", () => ({
  useAuth: () => ({
    loggedIn: true,
    apiFetch: mockApiFetch,
  }),
}));

describe("useAnimeStatus", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("should initialize with default status (0) if not provided", () => {
    const { result } = renderHook(() =>
      useAnimeStatus({ title: "Test Anime" }),
    );
    expect(result.current.currentStatus).toBe(0);
  });

  it("should initialize with provided initialStatus", () => {
    const { result } = renderHook(() =>
      useAnimeStatus({ title: "Test Anime", initialStatus: 2 }),
    );
    expect(result.current.currentStatus).toBe(2);
  });

  it("should update status optimistically and call API", async () => {
    mockApiFetch.mockResolvedValue({ ok: true });
    const onUpdate = vi.fn();
    const { result } = renderHook(() =>
      useAnimeStatus({
        title: "Test Anime",
        initialStatus: 1,
        initialScore: 8,
        onUpdate,
      }),
    );

    expect(result.current.currentStatus).toBe(1);

    await act(async () => {
      await result.current.updateStatus("2"); // Change to "Completed" (2)
    });

    // Check optimistic update
    expect(result.current.currentStatus).toBe(2);

    // Check API call
    expect(mockApiFetch).toHaveBeenCalledWith("/api/user/item", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ title: "Test Anime", status: 2, score: 8 }),
    });

    // Check onUpdate callback
    expect(onUpdate).toHaveBeenCalled();
  });

  it("should not crash on API failure", async () => {
    mockApiFetch.mockRejectedValue(new Error("API Error"));
    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});

    const { result } = renderHook(() =>
      useAnimeStatus({ title: "Test Anime", initialStatus: 1 }),
    );

    await act(async () => {
      await result.current.updateStatus("3");
    });

    // Should still have optimistic update (based on current implementation)
    expect(result.current.currentStatus).toBe(3);

    expect(consoleSpy).toHaveBeenCalledWith(
      "Failed to update status",
      expect.any(Error),
    );
  });
});
