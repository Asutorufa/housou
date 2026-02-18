import { render, waitFor } from "@testing-library/react";
import { describe, expect, test, vi } from "vitest";
import { MetadataProvider } from "../contexts/MetadataContext";
import AnimeCard from "./AnimeCard";
import type { DisplayAnimeItem } from "../types";

// Mock fetch
const fetchMock = vi.fn();
global.fetch = fetchMock;

const mockItem1: DisplayAnimeItem = {
  title: "Anime 1",
  type: "tv",
  lang: "ja",
  officialSite: "",
  begin: "2023-01-01",
  end: "",
  sites: [{ site: "tmdb", id: "1" }],
};

const mockItem2: DisplayAnimeItem = {
  title: "Anime 2",
  type: "tv",
  lang: "ja",
  officialSite: "",
  begin: "2023-01-01",
  end: "",
  sites: [{ site: "tmdb", id: "2" }],
};

describe("AnimeCard Batching", () => {
  test("batches multiple metadata requests into one", async () => {
    // Setup fetch mock to return success
    fetchMock.mockResolvedValue({
      ok: true,
      json: async () => [
        { id: "1", title: { native: "Anime 1" }, coverImage: {} }, // Result for Item 1
        { id: "2", title: { native: "Anime 2" }, coverImage: {} }, // Result for Item 2
      ],
    });

    // Mock IntersectionObserver to trigger immediately
    const MockIntersectionObserver = vi.fn();
    MockIntersectionObserver.mockImplementation(function (
      callback: IntersectionObserverCallback,
    ) {
      setTimeout(() => {
        callback(
          [{ isIntersecting: true } as IntersectionObserverEntry],
          {} as IntersectionObserver,
        );
      }, 10);
      return {
        observe: vi.fn(),
        disconnect: vi.fn(),
        unobserve: vi.fn(),
        takeRecords: vi.fn(),
        root: null,
        rootMargin: "",
        thresholds: [],
      };
    });
    vi.stubGlobal("IntersectionObserver", MockIntersectionObserver);

    render(
      <MetadataProvider>
        <AnimeCard item={mockItem1} onOpenModal={() => {}} />
        <AnimeCard item={mockItem2} onOpenModal={() => {}} />
      </MetadataProvider>,
    );

    // Wait for debounce (100ms) + small buffer
    await waitFor(
      () => {
        expect(fetchMock).toHaveBeenCalledTimes(1);
      },
      { timeout: 1000 },
    );

    // Verify the request body
    const call = fetchMock.mock.calls[0];
    expect(call[0]).toBe("/api/metadata");
    expect(call[1].method).toBe("POST");
    const body = JSON.parse(call[1].body);
    expect(body).toHaveLength(2);
    // Order depends on rendering, but usually consistent
    expect(body.map((b: any) => b.title)).toContain("Anime 1");
    expect(body.map((b: any) => b.title)).toContain("Anime 2");
  });
});
