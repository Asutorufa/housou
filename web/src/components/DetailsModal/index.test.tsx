import React from "react";
import { render, screen } from "@testing-library/react";
import DetailsModal from ".";
import { describe, it, expect, vi } from "vitest";
import { SiteMeta, DisplayAnimeItem, UnifiedMetadata } from "../../types";

// Mock AuthContext
vi.mock("../../contexts/AuthContext", () => ({
  useAuth: () => ({
    loggedIn: false,
    user: null,
  }),
}));

// Mock anime data
const mockAnime = {
  title: "Test Anime",
  info: {
    id: "1",
    title: { native: "Test Anime" },
    coverImage: { large: "http://example.com/image.jpg" },
    genres: [],
    studios: [],
    characters: [],
    staff: [],
    episodesList: [],
    isFinished: false,
    description: '<script>alert("xss")</script>',
  } as UnifiedMetadata,
};

const mockItems: DisplayAnimeItem[] = [
  {
    title: "Test Anime",
    type: "tv",
    lang: "ja",
    officialSite: "javascript:alert(1)", // Malicious official site
    begin: "",
    end: "",
    sites: [
      {
        site: "malicious-site",
        url: "javascript:alert(2)", // Malicious site link
      },
      {
        site: "valid-site",
        url: "https://example.com",
      },
    ],
  },
];

const mockSiteMeta: SiteMeta = {
  "malicious-site": {
    urlTemplate: "",
    type: "onair",
    title: "Malicious Site",
  },
  "valid-site": {
    urlTemplate: "",
    type: "onair",
    title: "Valid Site",
  },
};

describe("DetailsModal XSS Prevention", () => {
  it("renders description safely", () => {
    render(
      <DetailsModal
        isOpen={true}
        onClose={() => {}}
        anime={mockAnime}
        items={mockItems}
        siteMeta={mockSiteMeta}
      />,
    );

    // Find the description element
    const titleElement = screen.getByText("あらすじ");
    expect(titleElement).toBeTruthy();

    const descriptionContainer = titleElement.nextElementSibling;
    expect(descriptionContainer).toBeTruthy();

    expect(descriptionContainer?.innerHTML).not.toContain("<script>");
    expect(descriptionContainer?.textContent).toContain(
      '<script>alert("xss")</script>',
    );
  });

  it("does not render malicious official site links", () => {
    render(
      <DetailsModal
        isOpen={true}
        onClose={() => {}}
        anime={mockAnime}
        items={mockItems}
        siteMeta={mockSiteMeta}
      />,
    );

    // The "公式サイト" (Official Site) link should NOT be present because it's malicious
    // Note: The text might be rendered inside the h4, but the <a> tag should be missing
    const links = screen.queryAllByRole("link");
    const maliciousLink = links.find((link) =>
      link.getAttribute("href")?.startsWith("javascript:"),
    );
    expect(maliciousLink).toBeUndefined();
  });

  it("does not render malicious site links in list", () => {
    render(
      <DetailsModal
        isOpen={true}
        onClose={() => {}}
        anime={mockAnime}
        items={mockItems}
        siteMeta={mockSiteMeta}
      />,
    );

    // "Valid Site" should be present
    expect(screen.getByText("Valid Site")).toBeTruthy();

    // "Malicious Site" should NOT be present as link
    // It might be present as text if rendered differently, but here the loop filters it out completely if url is invalid.
    expect(screen.queryByText("Malicious Site")).toBeNull();
  });
});
