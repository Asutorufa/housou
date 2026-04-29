import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import DetailsModal from ".";
import { DisplayAnimeItem, SiteMeta, UnifiedMetadata } from "../../types";

// Mock AuthContext
vi.mock("../../contexts/AuthContext", () => ({
  useAuth: () => ({
    loggedIn: false,
    user: null,
  }),
}));

// Mock MetadataContext
vi.mock("../../contexts/MetadataContext", () => ({
  useMetadata: () => ({
    fetchMetadata: vi.fn(),
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
        authEnabled={true}
      />,
    );

    // Find the description element
    const titleElement = screen.getByText("あらすじ");
    expect(titleElement).toBeTruthy();

    const descriptionContainer = titleElement.nextElementSibling;
    expect(descriptionContainer).toBeTruthy();

    expect(descriptionContainer?.innerHTML).not.toContain("<script>");
    expect(descriptionContainer?.textContent).not.toContain(
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
        authEnabled={true}
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
        authEnabled={true}
      />,
    );

    // "Valid Site" should be present
    expect(screen.getByText("Valid Site")).toBeTruthy();

    // "Malicious Site" should NOT be present as link
    // It might be present as text if rendered differently, but here the loop filters it out completely if url is invalid.
    expect(screen.queryByText("Malicious Site")).toBeNull();
  });

  it("converts <br> tags to newlines", () => {
    const mockAnimeWithBr = {
      ...mockAnime,
      info: {
        ...mockAnime.info,
        description: "Line 1<br>Line 2<br />Line 3",
      } as UnifiedMetadata,
    };

    render(
      <DetailsModal
        isOpen={true}
        onClose={() => {}}
        anime={mockAnimeWithBr}
        items={mockItems}
        siteMeta={mockSiteMeta}
        authEnabled={true}
      />,
    );

    const descriptionContainer =
      screen.getByText("あらすじ").nextElementSibling;
    expect(descriptionContainer).toBeTruthy();
    // Check if innerHTML has newlines (which will be rendered due to whitespace-pre-wrap)
    // The current implementation replaces <br> with \n.
    // So innerHTML should be "Line 1\nLine 2\nLine 3" (approximately)
    expect(descriptionContainer?.innerHTML).toContain("Line 1\nLine 2\nLine 3");
  });

  it("adds rel='noopener noreferrer' to links with target='_blank'", () => {
    const mockAnimeWithLink = {
      ...mockAnime,
      info: {
        ...mockAnime.info,
        description:
          '<a href="http://example.com" target="_blank">External Link</a>',
      } as UnifiedMetadata,
    };

    render(
      <DetailsModal
        isOpen={true}
        onClose={() => {}}
        anime={mockAnimeWithLink}
        items={mockItems}
        siteMeta={mockSiteMeta}
        authEnabled={true}
      />,
    );

    const descriptionContainer =
      screen.getByText("あらすじ").nextElementSibling;
    expect(descriptionContainer).toBeTruthy();

    const link = descriptionContainer?.querySelector("a");
    expect(link).toBeTruthy();
    expect(link?.getAttribute("href")).toBe("http://example.com");
    expect(link?.getAttribute("target")).toBe("_blank");
    expect(link?.getAttribute("rel")).toBe("noopener noreferrer");
  });

  it("adds rel='noopener noreferrer' to links with case-insensitive target='_BLANK'", () => {
    const mockAnimeWithLink = {
      ...mockAnime,
      info: {
        ...mockAnime.info,
        description:
          '<a href="http://example.com" target="_BLANK">External Link</a>',
      } as UnifiedMetadata,
    };

    render(
      <DetailsModal
        isOpen={true}
        onClose={() => {}}
        anime={mockAnimeWithLink}
        items={mockItems}
        siteMeta={mockSiteMeta}
        authEnabled={true}
      />,
    );

    const descriptionContainer =
      screen.getByText("あらすじ").nextElementSibling;
    expect(descriptionContainer).toBeTruthy();

    const link = descriptionContainer?.querySelector("a");
    expect(link).toBeTruthy();
    expect(link?.getAttribute("target")).toBe("_BLANK");
    expect(link?.getAttribute("rel")).toBe("noopener noreferrer");
  });

  it("handles existing rel attributes and prevents partial matches", () => {
    const mockAnimeWithLink = {
      ...mockAnime,
      info: {
        ...mockAnime.info,
        description:
          '<a href="http://example.com" target="_blank" rel="nofollow fake-noopener">External Link</a>',
      } as UnifiedMetadata,
    };

    render(
      <DetailsModal
        isOpen={true}
        onClose={() => {}}
        anime={mockAnimeWithLink}
        items={mockItems}
        siteMeta={mockSiteMeta}
        authEnabled={true}
      />,
    );

    const descriptionContainer =
      screen.getByText("あらすじ").nextElementSibling;
    expect(descriptionContainer).toBeTruthy();

    const link = descriptionContainer?.querySelector("a");
    expect(link).toBeTruthy();
    expect(link?.getAttribute("target")).toBe("_blank");
    // Should preserve 'nofollow' and 'fake-noopener', and add 'noopener' and 'noreferrer'
    // Order in Set iteration is insertion order, but Set implementation might vary slightly?
    // Usually it preserves insertion order.
    // 'nofollow', 'fake-noopener', 'noopener', 'noreferrer'
    const rel = link?.getAttribute("rel") || "";
    expect(rel).toContain("nofollow");
    expect(rel).toContain("fake-noopener");
    expect(rel).toContain("noopener");
    expect(rel).toContain("noreferrer");
    // Ensure tokens are distinct
    const tokens = rel.split(" ");
    expect(tokens).toContain("noopener");
    expect(tokens).toContain("noreferrer");
  });
});
