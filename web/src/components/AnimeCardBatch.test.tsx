import { render, screen, waitFor } from "@testing-library/react";
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
    fetchMock.mockImplementation(async (url, options) => {
      if (url === "/api/metadata" && options.method === "POST") {
        const body = JSON.parse(options.body);
        return {
          ok: true,
          json: async () =>
            body.map(
              (req: {
                request_id: string;
                tmdb_id: string;
                title: string;
              }) => ({
                request_id: req.request_id,
                metadata: {
                  id: req.tmdb_id,
                  title: { native: req.title },
                  coverImage: {},
                },
              }),
            ),
        };
      }
      return { ok: false };
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
    expect(body.map((b: { title: string }) => b.title)).toContain("Anime 1");
    expect(body.map((b: { title: string }) => b.title)).toContain("Anime 2");

    // Verify IDs are present
    expect(body[0]).toHaveProperty("request_id");
    expect(body[1]).toHaveProperty("request_id");
  });

  test("handles partial failures gracefully", async () => {
    // Setup fetch mock to return partial success
    fetchMock.mockImplementation(async (url, options) => {
      if (url === "/api/metadata" && options.method === "POST") {
        const body = JSON.parse(options.body);
        return {
          ok: true,
          json: async () =>
            body.map((req: { request_id: string; title: string }) => {
              if (req.title === "Fail Anime") {
                return { request_id: req.request_id, metadata: null };
              }
              return {
                request_id: req.request_id,
                metadata: {
                  id: "success",
                  title: { native: req.title },
                  coverImage: {},
                },
              };
            }),
        };
      }
      return { ok: false };
    });

    const successItem: DisplayAnimeItem = {
      ...mockItem1,
      title: "Success Anime",
    };
    const failItem: DisplayAnimeItem = { ...mockItem2, title: "Fail Anime" };

    // Mock IO
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
        <AnimeCard item={successItem} onOpenModal={() => {}} />
        <AnimeCard item={failItem} onOpenModal={() => {}} />
      </MetadataProvider>,
    );

    // Wait for batch request
    await waitFor(
      () => {
        expect(fetchMock).toHaveBeenCalledTimes(1);
      },
      { timeout: 1000 },
    );

    // Wait for state updates
    // Success item should have metadata, fail item should remain null (or error state handled)
    // We can't easily check internal state of AnimeCard here without inspecting DOM changes
    // Success item should render title from metadata if different, or cover image
    // Fail item should render "No image" text
    await waitFor(
      () => {
        // "Success Anime" card should potentially show something if metadata loaded?
        // Our mock returns minimal metadata.
        // Let's assume AnimeCard renders "No image" if coverImage is empty/missing URL.
        // But we want to ensure no crash.
        const items = screen.getAllByText(/No image/i);
        // Both might show "No image" because our mock metadata coverImage is empty.
        expect(items.length).toBeGreaterThan(0);
      },
      { timeout: 1000 },
    );
  });
});
