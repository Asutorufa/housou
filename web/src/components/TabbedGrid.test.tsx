/* eslint-disable @typescript-eslint/no-unused-vars */
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi } from "vitest";
import TabbedGrid from "./TabbedGrid";
import type { DisplayAnimeItem } from "../types";
import "@testing-library/jest-dom";
import React from "react";

// Mock MetadataContext
vi.mock("../contexts/MetadataContext", () => ({
  useMetadata: () => ({
    fetchMetadata: vi.fn().mockResolvedValue(null),
  }),
}));

// Mock framer-motion (motion/react) to skip animations
vi.mock("motion/react", () => ({
  AnimatePresence: ({ children }: { children: React.ReactNode }) => <>{children}</>,
  motion: {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    div: ({ children, ...props }: any) => {
      // Filter out framer-motion specific props that might cause React warnings on div
      const { layoutId, layout, initial, animate, exit, variants, transition, custom, whileHover, ...rest } = props;
      return <div {...rest}>{children}</div>;
    },
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    h3: ({ children, ...props }: any) => {
      const { layoutId, ...rest } = props;
      return <h3 {...rest}>{children}</h3>;
    },
  },
}));

// Mock ResizeObserver
global.ResizeObserver = class ResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
};

// Mock IntersectionObserver
global.IntersectionObserver = class IntersectionObserver {
  constructor(_callback: IntersectionObserverCallback, _options?: IntersectionObserverInit) {}
  observe() {}
  unobserve() {}
  disconnect() {}
  takeRecords() { return []; }
  root = null;
  rootMargin = "";
  thresholds = [];
};

// Mock window.scrollTo
window.scrollTo = vi.fn();

const mockItems: DisplayAnimeItem[] = [
  {
    title: "Sunday Anime",
    type: "tv",
    lang: "ja",
    officialSite: "http://example.com",
    begin: "2023-10-01T00:00:00.000Z", // Sunday
    end: "2023-12-24T00:00:00.000Z",
  },
  {
    title: "Monday Anime",
    type: "tv",
    lang: "ja",
    officialSite: "http://example.com",
    begin: "2023-10-02T00:00:00.000Z", // Monday
    end: "2023-12-25T00:00:00.000Z",
  },
];

describe("TabbedGrid", () => {
  it("renders items for the current day initially", () => {
    // Mock date to be Sunday (Day 0)
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2023-10-01T12:00:00.000Z"));

    render(
      <TabbedGrid
        items={mockItems}
        onOpenModal={() => {}}
      />
    );

    // Should show Sunday Anime
    expect(screen.getByText("Sunday Anime")).toBeInTheDocument();
    // Should NOT show Monday Anime
    expect(screen.queryByText("Monday Anime")).not.toBeInTheDocument();

    vi.useRealTimers();
  });

  it("switches tabs and shows correct items", async () => {
     vi.useFakeTimers();
     vi.setSystemTime(new Date("2023-10-01T12:00:00.000Z")); // Sunday

     render(
      <TabbedGrid
        items={mockItems}
        onOpenModal={() => {}}
      />
    );

    // Switch to real timers immediately after render so interactions work
    vi.useRealTimers();

    const user = userEvent.setup();

    // Tabs.Trigger usually has role="tab"
    const mondayTab = screen.getByRole("tab", { name: "月" });
    await user.click(mondayTab);

    // Should show Monday Anime
    expect(await screen.findByText("Monday Anime")).toBeInTheDocument();

    // Should NOT show Sunday Anime
    await waitFor(() => {
        expect(screen.queryByText("Sunday Anime")).not.toBeInTheDocument();
    });
  });
});
