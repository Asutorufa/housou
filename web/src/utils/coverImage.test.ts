import { describe, expect, it } from "vitest";
import { getCardCoverUrl, getDetailCoverUrl } from "./coverImage";

describe("getDetailCoverUrl", () => {
  it("swaps TMDb's original upload for a modal-sized rendition", () => {
    expect(
      getDetailCoverUrl({
        large: "https://image.tmdb.org/t/p/w500/a.jpg",
        extraLarge: "https://image.tmdb.org/t/p/original/a.jpg",
      }),
    ).toBe("https://image.tmdb.org/t/p/w780/a.jpg");
  });

  it("keeps extraLarge for other providers and falls back to large", () => {
    expect(
      getDetailCoverUrl({
        large: "https://example.com/large.jpg",
        extraLarge: "https://example.com/xl.jpg",
      }),
    ).toBe("https://example.com/xl.jpg");
    expect(getDetailCoverUrl({ large: "https://example.com/large.jpg" })).toBe(
      "https://example.com/large.jpg",
    );
  });
});

describe("getCardCoverUrl", () => {
  it("avoids TMDb's original upload when a smaller image exists", () => {
    expect(
      getCardCoverUrl({
        large: "https://image.tmdb.org/t/p/w500/a.jpg",
        extraLarge: "https://image.tmdb.org/t/p/original/a.jpg",
      }),
    ).toBe("https://image.tmdb.org/t/p/w500/a.jpg");
  });

  it("falls back to the original when it is the only image", () => {
    expect(
      getCardCoverUrl({
        extraLarge: "https://image.tmdb.org/t/p/original/a.jpg",
      }),
    ).toBe("https://image.tmdb.org/t/p/original/a.jpg");
  });

  it("keeps extraLarge for other providers", () => {
    expect(
      getCardCoverUrl({
        large: "https://example.com/large.jpg",
        extraLarge: "https://example.com/xl.jpg",
      }),
    ).toBe("https://example.com/xl.jpg");
  });
});
