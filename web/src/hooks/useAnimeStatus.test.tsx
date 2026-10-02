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
      useAnimeStatus({
        title: "Test Anime",
        initialStatus: 2,
      }),
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
      body: JSON.stringify({ title: "Test Anime", status: 2 }),
    });

    // Check onUpdate callback
    expect(onUpdate).toHaveBeenCalled();
  });

  it("should revert status on API failure", async () => {
    mockApiFetch.mockRejectedValue(new Error("API Error"));

    const { result } = renderHook(() =>
      useAnimeStatus({ title: "Test Anime", initialStatus: 1 }),
    );

    await act(async () => {
      await result.current.updateStatus("3");
    });

    // Should revert to original status
    expect(result.current.currentStatus).toBe(1);

    expect(result.current.error).toBe("API Error");
  });
  it("rolls back HTTP failures and exposes the error", async () => {
    mockApiFetch.mockResolvedValue(
      new Response("Unavailable", { status: 503 }),
    );
    const onUpdate = vi.fn();
    const { result } = renderHook(() =>
      useAnimeStatus({ title: "Test Anime", initialStatus: 1, onUpdate }),
    );
    await act(async () => {
      expect(await result.current.updateStatus("2")).toBe(false);
    });
    expect(result.current.currentStatus).toBe(1);
    expect(result.current.error).toBe("Unavailable");
    expect(onUpdate).not.toHaveBeenCalled();
  });
  it("prevents overlapping writes and accepts the next change after completion", async () => {
    let complete!: (response: Response) => void;
    mockApiFetch.mockImplementation(
      () =>
        new Promise<Response>((resolve) => {
          complete = resolve;
        }),
    );
    const { result } = renderHook(() =>
      useAnimeStatus({ title: "Test Anime", initialStatus: 1 }),
    );
    let request!: Promise<boolean>;
    act(() => {
      request = result.current.updateStatus("2");
    });
    expect(result.current.updating).toBe(true);
    await act(async () => {
      expect(await result.current.updateStatus("3")).toBe(false);
    });
    expect(mockApiFetch).toHaveBeenCalledTimes(1);
    await act(async () => {
      complete(Response.json({ message: "Updated" }));
      await request;
    });
    expect(result.current.currentStatus).toBe(2);
    expect(result.current.updating).toBe(false);
  });
  it("rejects invalid status values without sending a request", async () => {
    const { result } = renderHook(() =>
      useAnimeStatus({ title: "Test Anime" }),
    );
    await act(async () => {
      expect(await result.current.updateStatus("6")).toBe(false);
      expect(await result.current.updateStatus("2abc")).toBe(false);
    });
    expect(mockApiFetch).not.toHaveBeenCalled();
  });
});
