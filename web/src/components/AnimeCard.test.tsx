import { render, screen, waitFor } from "@testing-library/react";
import AnimeCard from "./AnimeCard";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { SiteMeta, DisplayAnimeItem } from "../types";
import React from "react";

// Mock AuthContext if needed (AnimeCard doesn't use it directly but might have imports)
// It uses `onOpenModal` prop.

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
      <AnimeCard
        item={mockItemXSS}
        siteMeta={mockSiteMeta}
        onOpenModal={() => {}}
      />,
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
      json: () => Promise.resolve({}),
    } as Response);
  });

  afterEach(() => {
    window.IntersectionObserver = originalIntersectionObserver;
    vi.restoreAllMocks();
  });

  it("fetches metadata with correct URL parameters", async () => {
    render(<AnimeCard item={mockItemFetch} onOpenModal={() => {}} />);

    // Simulate intersection
    const mockEntry = { isIntersecting: true } as IntersectionObserverEntry;
    if (observerCallback) {
      observerCallback([mockEntry], {} as IntersectionObserver);
    }

    await waitFor(() => {
      expect(global.fetch).toHaveBeenCalledTimes(1);
    });

    const urlString = vi.mocked(global.fetch).mock.calls[0][0] as string;
    const url = new URL(urlString, "http://localhost");
    const params = url.searchParams;

    expect(url.pathname).toBe("/api/metadata");
    expect(params.get("title")).toBe("Test Anime");
    expect(params.get("tmdb_id")).toBe("123");
    expect(params.get("mal_id")).toBe("456");
    expect(params.get("anilist_id")).toBe("789");
    expect(params.get("begin")).toBe("2023-01-01");
  });
});
