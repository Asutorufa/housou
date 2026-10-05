import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { SWRConfig } from "swr";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { AuthProvider, useAuth } from "./AuthContext";
import { useAnimeData } from "../hooks/useAnimeData";
import type { AnimeItem, User } from "../types";

const account = (id: number): User => ({
  id,
  username: `user${id}`,
  email: `user${id}@example.com`,
  created_at: 0,
  has_password: true,
  github_id: "github",
  telegram_id: "telegram",
});
const anime: AnimeItem = {
  title: "Audit Anime",
  type: "tv",
  lang: "ja",
  officialSite: "",
  begin: "2024-01-01",
  end: "",
};
const fetchMock = vi.fn<typeof fetch>();
let session: User | null;
const storageValues = new Map<string, string>();
const storage = {
  getItem: (key: string) => storageValues.get(key) ?? null,
  setItem: (key: string, value: string) => {
    storageValues.set(key, value);
  },
  clear: () => storageValues.clear(),
};
function wrapper({ children }: { children: ReactNode }) {
  return (
    <SWRConfig
      value={{
        provider: () => new Map(),
        shouldRetryOnError: false,
        dedupingInterval: 0,
      }}
    >
      <AuthProvider enabled>{children}</AuthProvider>
    </SWRConfig>
  );
}
beforeEach(() => {
  session = account(1);
  storage.clear();
  storage.setItem(
    "housou_selections",
    JSON.stringify({ year: "2024", season: "all", site: "all", status: "all" }),
  );
  vi.stubGlobal("localStorage", storage);
  fetchMock.mockReset();
  fetchMock.mockImplementation(async (input) => {
    const url = String(input);
    if (url === "/api/auth/me")
      return session
        ? Response.json(session)
        : new Response("Unauthorized", { status: 401 });
    if (url.startsWith("/api/config"))
      return Response.json({
        years: [2024],
        site_meta: {},
        auth_enabled: true,
      });
    if (url.startsWith("/api/items")) return Response.json([anime]);
    if (url.startsWith("/api/user/status"))
      return Response.json({
        [anime.title]: { status: session?.id === 1 ? 1 : 3 },
      });
    if (url === "/api/auth/login") {
      session = account(2);
      return Response.json(session);
    }
    if (url === "/api/auth/logout") {
      session = null;
      return Response.json({ message: "Logged out" });
    }
    throw new Error(`Unexpected request: ${url}`);
  });
  vi.stubGlobal("fetch", fetchMock);
});
afterEach(() => vi.unstubAllGlobals());
describe("authentication and private state", () => {
  it("accepts the password JSON response and refreshes has_password", async () => {
    session = { ...account(1), has_password: false };
    const fallback = fetchMock.getMockImplementation()!;
    fetchMock.mockImplementation(async (url, init) => {
      if (String(url) === "/api/auth/password") {
        session = { ...session!, has_password: true };
        return Response.json({ message: "Password updated" });
      }
      return fallback(url, init);
    });
    const { result } = renderHook(() => useAuth(), { wrapper });
    await waitFor(() => expect(result.current.user?.has_password).toBe(false));
    await act(async () => {
      await result.current.changePassword({ new_password: "Abcdefgh1" });
    });
    expect(result.current.user?.has_password).toBe(true);
  });
  it.each(["github", "telegram"] as const)(
    "refreshes the profile after unbinding %s",
    async (service) => {
      const fallback = fetchMock.getMockImplementation()!;
      fetchMock.mockImplementation(async (url, init) => {
        if (String(url) === `/api/auth/${service}`) {
          session = { ...session!, [`${service}_id`]: undefined };
          return Response.json({ message: "Disconnected" });
        }
        return fallback(url, init);
      });
      const { result } = renderHook(() => useAuth(), { wrapper });
      await waitFor(() => expect(result.current.loggedIn).toBe(true));
      await act(async () => {
        await (service === "github"
          ? result.current.unbindGithub()
          : result.current.unbindTelegram());
      });
      expect(result.current.user?.[`${service}_id`]).toBeUndefined();
    },
  );
  it("keeps a valid session when an incorrect old password returns 401", async () => {
    const fallback = fetchMock.getMockImplementation()!;
    fetchMock.mockImplementation((url, init) =>
      String(url) === "/api/auth/password"
        ? Promise.resolve(new Response("Invalid old password", { status: 401 }))
        : fallback(url, init),
    );
    const { result } = renderHook(() => useAuth(), { wrapper });
    await waitFor(() => expect(result.current.loggedIn).toBe(true));
    await act(async () => {
      await expect(
        result.current.changePassword({
          old_password: "wrong",
          new_password: "Abcdefgh1",
        }),
      ).rejects.toThrow("Invalid old password");
    });
    expect(result.current.loggedIn).toBe(true);
  });
  it("clears the user when the session itself has expired", async () => {
    const { result } = renderHook(() => useAuth(), { wrapper });
    await waitFor(() => expect(result.current.loggedIn).toBe(true));
    session = null;
    const fallback = fetchMock.getMockImplementation()!;
    fetchMock.mockImplementation((url, init) =>
      String(url) === "/api/user/item"
        ? Promise.resolve(new Response("Unauthorized", { status: 401 }))
        : fallback(url, init),
    );
    await act(async () => {
      await expect(
        result.current.apiFetch("/api/user/item", { method: "POST" }),
      ).rejects.toThrow("Unauthorized");
    });
    expect(result.current.loggedIn).toBe(false);
  });
  it("loads the new account statuses instead of reusing the previous account cache", async () => {
    const { result } = renderHook(
      () => ({ auth: useAuth(), data: useAnimeData() }),
      { wrapper },
    );
    await waitFor(() =>
      expect(result.current.data.items[0]?.userStatus).toBe(1),
    );
    await act(async () => {
      await result.current.auth.login({
        email: "user2@example.com",
        password: "abcdefgh",
      });
    });
    await waitFor(() =>
      expect(result.current.data.items[0]?.userStatus).toBe(3),
    );
    expect(result.current.auth.user?.id).toBe(2);
  });
  it("ignores persisted private status filters after logout", async () => {
    storage.setItem(
      "housou_selections",
      JSON.stringify({ year: "2024", season: "all", site: "all", status: "1" }),
    );
    const { result } = renderHook(
      () => ({ auth: useAuth(), data: useAnimeData() }),
      { wrapper },
    );
    await waitFor(() =>
      expect(result.current.data.filteredItems).toHaveLength(1),
    );
    await act(async () => {
      await result.current.auth.logout();
    });
    expect(result.current.auth.loggedIn).toBe(false);
    expect(result.current.data.filteredItems).toHaveLength(1);
    expect(result.current.data.items[0].userStatus).toBeUndefined();
  });
  it("does not clear the session when logout fails", async () => {
    const fallback = fetchMock.getMockImplementation()!;
    fetchMock.mockImplementation((url, init) =>
      String(url) === "/api/auth/logout"
        ? Promise.resolve(new Response("Unavailable", { status: 503 }))
        : fallback(url, init),
    );
    const { result } = renderHook(() => useAuth(), { wrapper });
    await waitFor(() => expect(result.current.loggedIn).toBe(true));
    await expect(result.current.logout()).rejects.toThrow("Unavailable");
    expect(result.current.loggedIn).toBe(true);
  });
});
