import { render, screen } from "@testing-library/react";
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
});
