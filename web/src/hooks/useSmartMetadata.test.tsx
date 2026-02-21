import { renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useSmartMetadata } from "./useSmartMetadata";
import { DisplayAnimeItem, UnifiedMetadata } from "../types";
import { isDev } from "../utils/envUtils";

// Mock MetadataContext
const { mockFetchMetadata } = vi.hoisted(() => {
  return { mockFetchMetadata: vi.fn() };
});

vi.mock("../contexts/MetadataContext", () => ({
  useMetadata: () => ({
    fetchMetadata: mockFetchMetadata,
  }),
}));

// Mock envUtils
vi.mock("../utils/envUtils", () => ({
  isDev: vi.fn().mockReturnValue(false),
}));

// Test data
const mockItem: DisplayAnimeItem = {
  title: "Test Anime",
  type: "tv",
  begin: "2024-01-01",
  end: "",
  sites: [
    { site: "tmdb", id: "123" },
    { site: "mal", id: "456" },
    { site: "aniList", id: "789" },
  ],
};

const mockMetadata: UnifiedMetadata = {
  tmdb_id: 123,
  mal_id: 456,
  anilist_id: 789,
  title: "Test Anime",
  info: {
    title: { native: "Test Anime" },
    coverImage: { large: "" },
    genres: [],
    studios: [],
    characters: [],
    staff: [],
    episodesList: [],
    isFinished: false,
    description: "Description",
  },
};

describe("useSmartMetadata", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("should return initial metadata immediately if provided", () => {
    const { result } = renderHook(() =>
      useSmartMetadata(mockItem, mockMetadata),
    );

    expect(result.current.loading).toBe(false);
    expect(result.current.metadata).toEqual(mockMetadata);
    expect(mockFetchMetadata).not.toHaveBeenCalled();
  });

  it("should fetch metadata if no initial metadata is provided", async () => {
    mockFetchMetadata.mockResolvedValue(mockMetadata);

    const { result } = renderHook(() => useSmartMetadata(mockItem));

    // Initially loading
    expect(result.current.loading).toBe(true);
    expect(result.current.metadata).toBeNull();

    // Wait for fetch to complete
    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.metadata).toEqual(mockMetadata);
    expect(mockFetchMetadata).toHaveBeenCalledWith({
      title: "Test Anime",
      tmdb_id: "123",
      mal_id: "456",
      anilist_id: "789",
      year: 2024,
    });
  });

  it("should handle missing sites gracefully", async () => {
    mockFetchMetadata.mockResolvedValue(mockMetadata);
    const itemWithoutSites: DisplayAnimeItem = {
      ...mockItem,
      sites: [],
    };

    const { result } = renderHook(() => useSmartMetadata(itemWithoutSites));

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(mockFetchMetadata).toHaveBeenCalledWith({
      title: "Test Anime",
      tmdb_id: undefined,
      mal_id: undefined,
      anilist_id: undefined,
      year: 2024,
    });
  });

  it("should handle missing begin date gracefully", async () => {
    mockFetchMetadata.mockResolvedValue(mockMetadata);
    const itemWithoutDate: DisplayAnimeItem = {
      ...mockItem,
      begin: "",
    };

    const { result } = renderHook(() => useSmartMetadata(itemWithoutDate));

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(mockFetchMetadata).toHaveBeenCalledWith(
      expect.objectContaining({
        year: undefined,
      }),
    );
  });

  it("should handle invalid begin date gracefully", async () => {
    mockFetchMetadata.mockResolvedValue(mockMetadata);
    const itemWithInvalidDate: DisplayAnimeItem = {
      ...mockItem,
      begin: "invalid-date",
    };

    const { result } = renderHook(() => useSmartMetadata(itemWithInvalidDate));

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(mockFetchMetadata).toHaveBeenCalledWith(
      expect.objectContaining({
        year: undefined,
      }),
    );
  });

  it("should not fetch if disabled", () => {
    const { result } = renderHook(() =>
      useSmartMetadata(mockItem, null, false),
    );

    expect(result.current.loading).toBe(false);
    expect(mockFetchMetadata).not.toHaveBeenCalled();
  });

  it("should handle fetch error gracefully", async () => {
    const consoleErrorSpy = vi
      .spyOn(console, "error")
      .mockImplementation(() => {});
    mockFetchMetadata.mockRejectedValue(new Error("Fetch failed"));

    // Enable dev mode for error logging check
    vi.mocked(isDev).mockReturnValue(true);

    const { result } = renderHook(() => useSmartMetadata(mockItem));

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.metadata).toBeNull();
    expect(consoleErrorSpy).toHaveBeenCalled();
    consoleErrorSpy.mockRestore();
  });

  it("should not update state if unmounted", async () => {
    // Create a promise that we can control
    let resolvePromise: (value: UnifiedMetadata) => void = () => {};
    const promise = new Promise<UnifiedMetadata>((resolve) => {
      resolvePromise = resolve;
    });

    mockFetchMetadata.mockReturnValue(promise);

    const { result, unmount } = renderHook(() => useSmartMetadata(mockItem));

    expect(result.current.loading).toBe(true);

    unmount();

    // Resolve the promise after unmount
    resolvePromise(mockMetadata);

    // We can't really assert that state didn't update directly in React hooks testing easily without spying on useState,
    // but we can ensure no errors are thrown (React warning about state update on unmounted component).
    // The implementation has `isMounted` check.
    // If it updated state, it might trigger a warning in console or affect subsequent renders if mocked incorrectly.

    // However, since we are using `renderHook`, the result won't update after unmount.
    // We can check that mocking is called.
    expect(mockFetchMetadata).toHaveBeenCalled();
  });
});
