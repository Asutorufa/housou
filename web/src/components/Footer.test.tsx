import React from "react";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import {
  describe,
  it,
  expect,
  vi,
  beforeAll,
  afterAll,
  beforeEach,
} from "vitest";
import Footer from "./Footer";

describe("Footer Component", () => {
  const onOpenAttribution = vi.fn();

  beforeAll(() => {
    // Only fake Date to avoid interfering with userEvent's timers (setTimeout, etc.)
    vi.useFakeTimers({ toFake: ["Date"] });
    vi.setSystemTime(new Date("2024-01-01"));
  });

  afterAll(() => {
    vi.useRealTimers();
  });

  beforeEach(() => {
    onOpenAttribution.mockClear();
    render(<Footer onOpenAttribution={onOpenAttribution} />);
  });

  it("renders correctly with copyright text", () => {
    const copyrightText = `© 2024 Housou. All rights reserved.`;
    expect(screen.getByText(copyrightText)).toBeInTheDocument();
  });

  it("renders the GitHub link with correct attributes", () => {
    // Find the link by its href attribute since the accessible name depends on the SVG title
    // which might vary in implementation details across environments.
    // However, finding by role is preferred for accessibility testing.
    // The SVG has <title>GitHub</title>, so the link should have the name "GitHub".
    const githubLink = screen.getByRole("link", { name: /GitHub/i });

    expect(githubLink).toBeInTheDocument();
    expect(githubLink).toHaveAttribute(
      "href",
      "https://github.com/Asutorufa/housou",
    );
    expect(githubLink).toHaveAttribute("target", "_blank");
    expect(githubLink).toHaveAttribute("rel", "noopener noreferrer");
  });

  it("calls onOpenAttribution when the attribution button is clicked", async () => {
    // No need for advanceTimers since we are only faking Date
    const user = userEvent.setup();

    const button = screen.getByRole("button", {
      name: /Data Sources & Attribution/i,
    });

    await user.click(button);

    expect(onOpenAttribution).toHaveBeenCalledTimes(1);
  });
});
