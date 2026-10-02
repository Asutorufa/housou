import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { SWRConfig } from "swr";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useComments } from "./useComments";
import type { CommentWithUser } from "../../../types";
const apiFetch = vi.hoisted(() => vi.fn<typeof fetch>());
vi.mock("../../../contexts/AuthContext", () => ({
  useAuth: () => ({ user: { id: 1 }, loggedIn: true, apiFetch }),
}));
const comment: CommentWithUser = {
  id: 1,
  userId: 1,
  username: "A",
  content: "Original",
  score: 80,
  status: 1,
  createdAt: 0,
  updatedAt: 0,
};
let saved: CommentWithUser;
const fetchMock = vi.fn<typeof fetch>();
function wrapper({ children }: { children: ReactNode }) {
  return (
    <SWRConfig value={{ provider: () => new Map(), shouldRetryOnError: false }}>
      {children}
    </SWRConfig>
  );
}
beforeEach(() => {
  saved = { ...comment };
  fetchMock.mockReset();
  apiFetch.mockReset();
  fetchMock.mockImplementation(async () =>
    Response.json({ comments: [saved], total: 1 }),
  );
  vi.stubGlobal("fetch", fetchMock);
});
afterEach(() => vi.unstubAllGlobals());
describe("comment writes", () => {
  it("preserves the saved comment and exposes HTTP and network errors", async () => {
    const { result } = renderHook(() => useComments("A"), { wrapper });
    await waitFor(() =>
      expect(result.current.userComment?.content).toBe("Original"),
    );
    apiFetch.mockResolvedValueOnce(new Response("Failed", { status: 500 }));
    await act(async () => {
      expect(await result.current.saveComment("New")).toBe(false);
    });
    expect(result.current.error).toBe("Failed");
    expect(result.current.userComment?.content).toBe("Original");
    apiFetch.mockRejectedValueOnce(new Error("Offline"));
    await act(async () => {
      expect(await result.current.saveRating(95)).toBe(false);
    });
    expect(result.current.error).toBe("Offline");
    expect(result.current.userComment?.score).toBe(80);
    expect(result.current.pending).toBe(false);
  });
  it("shares a write lock between rating, commenting and deleting", async () => {
    let complete!: (response: Response) => void;
    apiFetch.mockImplementation(
      () =>
        new Promise((resolve) => {
          complete = resolve;
        }),
    );
    const { result } = renderHook(() => useComments("A"), { wrapper });
    await waitFor(() => expect(result.current.userComment).toBeDefined());
    let request!: Promise<boolean>;
    act(() => {
      request = result.current.saveRating(95);
    });
    await act(async () => {
      expect(await result.current.deleteComment(1)).toBe(false);
      expect(await result.current.saveComment("New")).toBe(false);
    });
    expect(apiFetch).toHaveBeenCalledTimes(1);
    saved.score = 95;
    await act(async () => {
      complete(Response.json({ ...saved }));
      await request;
    });
    expect(result.current.userComment?.score).toBe(95);
    expect(result.current.pending).toBe(false);
  });
  it("refreshes the saved score after clearing it", async () => {
    const { result } = renderHook(() => useComments("A"), { wrapper });
    await waitFor(() => expect(result.current.userComment?.score).toBe(80));
    apiFetch.mockImplementation(async () => {
      saved.score = null;
      return Response.json(saved);
    });
    await act(async () => {
      expect(await result.current.saveRating(null)).toBe(true);
    });
    expect(result.current.userComment?.score).toBeNull();
  });
});
