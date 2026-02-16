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

    // Since loggedIn is false, there are 3 dropdowns: Year, Season, Site.
    // "all" -> "全て".
    // Year usually defaults to string, Season/Site to "all".
    // We need to target the Site dropdown specifically.
    // Based on the code, Site is the 3rd one.
    // But Radix UI Select triggers are buttons.
    // Let's try to find it by the "placeholder" text if possible, or by the initial value label.
    // The placeholder is "サイト" (Site).
    // The CustomSelect implementation renders `Select.Value placeholder={placeholder}` inside the button.

    // If selectedSite="all", the button should contain "全て".
    // There are multiple "全て" (Season, Site).
    // Let's look for the one associated with "site".
    // The `CustomSelect` doesn't seem to have a unique accessibility label prop exposed easily besides the trigger content.
    // However, the test environment (jsdom) + userEvent should handle simple clicks.

    // Let's try clicking the button that has "全て" AND is the 3rd one, or try to be more specific.
    // Or we can rely on the fact that the test suggestion used `screen.getByRole('button', { name: '全て' })`.
    // But `getByRole` might throw if multiple elements match.
    // Let's use `getAllByRole` and pick the last one (Site).
    // Year: value="2024" (label "2024").
    // Season: value="Winter" (label "冬").
    // Site: value="all" (label "全て").
    // Status: (hidden because loggedIn=false).
    // So there should be only ONE "全て" button visible if season is "Winter".
    // Wait, in the render call above: selectedSeason="Winter".
    // So Season dropdown shows "冬".
    // Site dropdown shows "全て" (selectedSite="all").
    // So `getByRole('button', { name: '全て' })` should be unique and safe!

    // Using getAllByRole to be safe in case there are other buttons with the same text.
    // However, the CustomSelect renders the label as "text" inside a span, not necessarily as the accessible name of the button.
    // The button might not have an aria-label set to the selected value.
    // Let's inspect the DOM structure from the failure log.
    // The button has: <span style="pointer-events: none;">全て</span>
    // But it doesn't seem to have aria-label="全て".
    // So getByRole("combobox", { name: "全て" }) fails because the button's accessible name is not "全て".
    // Wait, accessible name computation usually includes text content.
    // <button><span>text</span></button> -> accessible name "text".
    // But the span has pointer-events: none... that shouldn't affect a11y name.

    // Let's try finding by text content instead, which is robust for user-visible text.
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
