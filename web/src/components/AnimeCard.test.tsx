import { render, screen } from "@testing-library/react";
import AnimeCard from "./AnimeCard";
import { describe, it, expect, vi } from "vitest";
import { SiteMeta, DisplayAnimeItem } from "../types";

// Mock AuthContext if needed (AnimeCard doesn't use it directly but might have imports)
// It uses `onOpenModal` prop.

const mockItem: DisplayAnimeItem = {
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
        item={mockItem}
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
