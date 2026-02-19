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

    // Verify search button is present (search input is hidden initially)
    expect(screen.getByLabelText("検索を開く")).toBeInTheDocument();
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

  it("displays scheduled label for future seasons", async () => {
    const user = userEvent.setup();
    const currentYear = new Date().getFullYear();
    const futureYear = currentYear + 1;
    const configWithFutureYear: Config = {
      ...mockConfig,
      years: [currentYear, futureYear],
    };

    render(
      <Header
        config={configWithFutureYear}
        selectedYear={futureYear.toString()}
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

    // Since selectedYear is in the future, Winter should be "冬 (予定)"
    const label = screen.getByText(/冬 \(予定\)/);
    expect(label).toBeInTheDocument();

    const trigger = label.closest("button");
    expect(trigger).toBeInTheDocument();

    await user.click(trigger!);

    // Verify other seasons also have (予定)
    expect(await screen.findByText(/春 \(予定\)/)).toBeInTheDocument();
    expect(screen.getByText("夏 (予定)")).toBeInTheDocument();
  });
});
