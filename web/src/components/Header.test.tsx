import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi } from "vitest";
import Header from "./Header";
import type { Config } from "../types";

// Mock useAuth
vi.mock("../contexts/AuthContext", () => ({
  useAuth: () => ({
    loggedIn: false,
    user: undefined,
  }),
}));

const mockConfig: Config = {
  years: [2023, 2024],
  site_meta: {
    site1: { title: "Site One", type: "tv", regions: ["US"] },
    site2: { title: "Site Two", type: "tv", regions: ["US"] },
  },
  attribution: {
    tmdb: {
      logo_square: "tmdb_logo.png",
      logo_long: "tmdb_logo_long.png",
      logo_alt_long: "TMDB",
    },
  },
  auth_enabled: true,
};

describe("Header Component", () => {
  it("renders correctly with given config", () => {
    render(
      <Header
        config={mockConfig}
        selectedYear="2024"
        setSelectedYear={vi.fn()}
        selectedSeason="Winter"
        setSelectedSeason={vi.fn()}
        selectedSite="all"
        setSelectedSite={vi.fn()}
        selectedStatus="all"
        setSelectedStatus={vi.fn()}
        searchQuery=""
        setSearchQuery={vi.fn()}
      />,
    );

    // Verify search input is present
    expect(screen.getByPlaceholderText("検索...")).toBeInTheDocument();
  });

  it("displays site options when the site dropdown is clicked", async () => {
    const user = userEvent.setup();
    render(
      <Header
        config={mockConfig}
        selectedYear="2024"
        setSelectedYear={vi.fn()}
        selectedSeason="Winter"
        setSelectedSeason={vi.fn()}
        selectedSite="all"
        setSelectedSite={vi.fn()}
        selectedStatus="all"
        setSelectedStatus={vi.fn()}
        searchQuery=""
        setSearchQuery={vi.fn()}
      />,
    );

    // Find the site dropdown trigger by its text content "全て".
    // We use getAllByText because other dropdowns might also display "全て".
    // The Site dropdown is the last one in the list.
    const allTextMatches = screen.getAllByText("全て");
    // We want the one that is inside a button/combobox.
    // The structure is button -> div -> span -> "全て".

    // Let's find the button that *contains* this text.
    const siteDropdownTrigger =
      allTextMatches[allTextMatches.length - 1].closest("button");

    if (!siteDropdownTrigger) {
      throw new Error("Could not find site dropdown trigger button");
    }

    await user.click(siteDropdownTrigger);

    // Check that options from mockConfig are visible
    // Radix UI renders options in a Portal, so they should be in the document.
    expect(await screen.findByText("Site One")).toBeInTheDocument();
    expect(screen.getByText("Site Two")).toBeInTheDocument();
  });
});
