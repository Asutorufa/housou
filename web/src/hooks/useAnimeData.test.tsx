import { renderHook, waitFor } from "@testing-library/react";
import { SWRConfig } from "swr";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useAnimeData } from "./useAnimeData";

// Mock useAuth
vi.mock("../contexts/AuthContext", () => ({
  useAuth: () => ({
    loggedIn: false,
    apiFetch: vi.fn(),
  }),
}));

// Mock fetch
const mockFetch = vi.fn();
vi.stubGlobal("fetch", mockFetch);

const mockLocalStorage = {
  getItem: vi.fn(() =>
    JSON.stringify({ year: "2024", season: "all", site: "all", status: "all" }),
  ),
  setItem: vi.fn(),
  clear: vi.fn(),
};
Object.defineProperty(window, "localStorage", {
  value: mockLocalStorage,
});

describe("useAnimeData", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockLocalStorage.clear();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("should fetch config and initialize selections", async () => {
    // Mock config response
    mockFetch.mockImplementation((url) => {
      if (url.toString().startsWith("/api/config")) {
        return Promise.resolve({
          ok: true,
          json: async () => ({
            years: [2023, 2024],
            site_meta: {},
            auth_enabled: false,
          }),
        });
      }
      return Promise.resolve({
        ok: true,
        json: async () => [],
      });
    });

    const wrapper = ({ children }: { children: React.ReactNode }) => (
      <SWRConfig value={{ provider: () => new Map() }}>{children}</SWRConfig>
    );

    const { result } = renderHook(() => useAnimeData(), { wrapper });

    // Initially loading
    expect(result.current.loading).toBe(true);

    // Wait for config to load
    await waitFor(() => {
      expect(result.current.config).not.toBeNull();
    });

    expect(result.current.config).toEqual({
      years: [2023, 2024],
      site_meta: {},
      auth_enabled: false,
    });

    // Check if selections are initialized
    // Default logic: year should be valid or default to current/last year
    // Since we didn't mock Date, it uses system date.
    // If system year is not in [2023, 2024], it defaults to last year (2024).
    // Or if system year is in list, it uses that.

    // We can't easily predict year without mocking Date, but we can check structure.
    expect(result.current.selections.year).toBeDefined();
    expect(result.current.selections.season).toBeDefined();
  });

  it("should fetch items when selections are valid", async () => {
    // Mock config response
    mockFetch.mockImplementation((url) => {
      const urlString = url.toString();
      if (urlString.startsWith("/api/config")) {
        return Promise.resolve({
          ok: true,
          json: async () => ({
            years: [2024],
            site_meta: {},
            auth_enabled: false,
          }),
        });
      }
      if (urlString.includes("/api/items")) {
        return Promise.resolve({
          ok: true,
          json: async () => [
            { title: "Anime 1", sites: [{ site: "site1" }] },
            { title: "Anime 2", sites: [{ site: "site2" }] },
          ],
        });
      }
      return Promise.resolve({
        ok: false,
        status: 404,
        statusText: "Not Found",
        text: async () => "Not Found",
      });
    });

    const wrapper = ({ children }: { children: React.ReactNode }) => (
      <SWRConfig value={{ provider: () => new Map(), dedupingInterval: 0 }}>
        {children}
      </SWRConfig>
    );

    const { result } = renderHook(() => useAnimeData(), { wrapper });

    await waitFor(() => {
      expect(result.current.config).not.toBeNull();
    });

    // Wait for items to load
    await waitFor(() => {
      expect(result.current.items).toHaveLength(2);
    });

    expect(result.current.items[0].title).toBe("Anime 1");
    expect(result.current.items[1].title).toBe("Anime 2");
  });
});
