import { render, screen, waitFor, act } from "@testing-library/react";
import AnimeCard from "./AnimeCard";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { SiteMeta, DisplayAnimeItem } from "../types";
import React from "react";
import { isDev } from "../utils/envUtils";
import { MetadataProvider } from "../contexts/MetadataContext";

// Mock envUtils
vi.mock("../utils/envUtils", () => ({
  isDev: vi.fn(),
}));

const mockItemXSS: DisplayAnimeItem = {
  title: "Test Anime",
  type: "tv",
  lang: "ja",
  officialSite: "",
  begin: "2023-01-01",
  end: "",
  userStatus: 0,
  sites: [
    {
      site: "malicious-site",
      url: "javascript:alert(1)", // Malicious URL
    },
    {
      site: "valid-site",
      url: "https://example.com",
    },
  ],
};

const mockSiteMeta: SiteMeta = {
  "malicious-site": {
    urlTemplate: "",
    type: "onair", // AnimeCard only shows 'onair' sites
    title: "Malicious Site",
  },
  "valid-site": {
    urlTemplate: "",
    type: "onair",
    title: "Valid Site",
  },
};

describe("AnimeCard XSS Prevention", () => {
  it("does not render malicious site links", () => {
    render(
      <MetadataProvider>
        <AnimeCard
          item={mockItemXSS}
          siteMeta={mockSiteMeta}
          onOpenModal={() => {}}
        />
      </MetadataProvider>,
    );

    // "Valid Site" should be present
    expect(screen.getByText("Valid Site")).toBeTruthy();

    // "Malicious Site" should NOT be present
    // If vulnerable, this will fail because "Malicious Site" will be found
    expect(screen.queryByText("Malicious Site")).toBeNull();
  });
});

const mockItemFetch: DisplayAnimeItem = {
  title: "Test Anime",
  type: "tv",
  lang: "ja",
  officialSite: "",
  begin: "2023-01-01",
  end: "",
  userStatus: 0,
  sites: [
    { site: "tmdb", id: "123" },
    { site: "mal", id: "456" },
    { site: "aniList", id: "789" },
  ],
};

describe("AnimeCard fetchMetadata", () => {
  let observerCallback: IntersectionObserverCallback;
  const observeMock = vi.fn();
  const disconnectMock = vi.fn();
  const originalIntersectionObserver = window.IntersectionObserver;

  beforeEach(() => {
    // Mock IntersectionObserver
    const MockIntersectionObserver = vi.fn();
    MockIntersectionObserver.mockImplementation(function (
      cb: IntersectionObserverCallback,
    ) {
      observerCallback = cb;
      return {
        observe: observeMock,
        disconnect: disconnectMock,
        unobserve: vi.fn(),
        takeRecords: vi.fn(),
        root: null,
        rootMargin: "",
        thresholds: [],
      };
    });
    vi.stubGlobal("IntersectionObserver", MockIntersectionObserver);

    // Mock fetch
    vi.spyOn(global, "fetch").mockResolvedValue({
      ok: true,
      json: () => Promise.resolve([{}]) as Promise<any>,
    } as Response);
  });

  afterEach(() => {
    window.IntersectionObserver = originalIntersectionObserver;
    vi.restoreAllMocks();
  });

  it("fetches metadata with correct URL parameters", async () => {
    render(
      <MetadataProvider>
        <AnimeCard item={mockItemFetch} onOpenModal={() => {}} />
      </MetadataProvider>,
    );

    // Simulate intersection
    const mockEntry = { isIntersecting: true } as IntersectionObserverEntry;
    if (observerCallback) {
      act(() => {
        observerCallback([mockEntry], {} as IntersectionObserver);
      });
    }

    // Allow debounce to fire
    await waitFor(
      () => {
        expect(global.fetch).toHaveBeenCalledTimes(1);
      },
      { timeout: 1000 },
    );

    const urlString = vi.mocked(global.fetch).mock.calls[0][0] as string;
    const opts = vi.mocked(global.fetch).mock.calls[0][1];

    expect(urlString).toBe("/api/metadata");
    expect(opts?.method).toBe("POST");

    const body = JSON.parse(opts?.body as string);
    // Should be an array with one request
    expect(Array.isArray(body)).toBe(true);
    expect(body).toHaveLength(1);
    const params = body[0];

    expect(params.title).toBe("Test Anime");
    expect(params.tmdb_id).toBe("123");
    expect(params.mal_id).toBe("456");
    expect(params.anilist_id).toBe("789");
    expect(params.year).toBe(2023); // 2023-01-01 -> 2023
  });

  it("handles fetch failure gracefully without logging in production", async () => {
    // Mock isDev to return false (Production)
    vi.mocked(isDev).mockReturnValue(false);

    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    vi.mocked(global.fetch).mockRejectedValue(new Error("Network error"));

    render(
      <MetadataProvider>
        <AnimeCard item={mockItemFetch} onOpenModal={() => {}} />
      </MetadataProvider>,
    );

    // Simulate intersection
    const mockEntry = { isIntersecting: true } as IntersectionObserverEntry;
    if (observerCallback) {
      act(() => {
        observerCallback([mockEntry], {} as IntersectionObserver);
      });
    }

    // Wait for the "No image" text which appears when loading is false and no cover image
    // Note: With batching, failure of fetch returns null to promises.
    await waitFor(
      () => {
        expect(screen.getByText("No image")).toBeInTheDocument();
      },
      { timeout: 1000 },
    );

    // In context: catch(err) -> console.error("Batch fetch error:", err) -> resolve(null).
    // So "Metadata error:" is NOT logged by AnimeCard.
    // But "Batch fetch error:" IS logged by Context.
    expect(consoleSpy).toHaveBeenCalledWith(
      "Batch fetch error:",
      expect.any(Error),
    );
    expect(consoleSpy).not.toHaveBeenCalledWith(
      "Metadata error:",
      expect.any(Error),
    );

    consoleSpy.mockRestore();
  });

  it("logs fetch failure in development", async () => {
    // Mock isDev to return true (Development)
    vi.mocked(isDev).mockReturnValue(true);

    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    vi.mocked(global.fetch).mockRejectedValue(new Error("Network error"));

    render(
      <MetadataProvider>
        <AnimeCard item={mockItemFetch} onOpenModal={() => {}} />
      </MetadataProvider>,
    );

    // Simulate intersection
    const mockEntry = { isIntersecting: true } as IntersectionObserverEntry;
    if (observerCallback) {
      act(() => {
        observerCallback([mockEntry], {} as IntersectionObserver);
      });
    }

    // Wait for the "No image" text
    await waitFor(
      () => {
        expect(screen.getByText("No image")).toBeInTheDocument();
      },
      { timeout: 1000 },
    );

    // With batching, context catches error and returns null.
    // So "Metadata error:" is NOT logged by AnimeCard.
    // But "Batch fetch error:" IS logged by Context.
    expect(consoleSpy).toHaveBeenCalledWith(
      "Batch fetch error:",
      expect.any(Error),
    );
    consoleSpy.mockRestore();
  });
});
