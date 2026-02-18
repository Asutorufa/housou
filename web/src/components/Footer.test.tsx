import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi } from "vitest";
import Footer from "./Footer";

describe("Footer Component", () => {
  it("renders correctly with copyright text", () => {
    const onOpenAttribution = vi.fn();
    render(<Footer onOpenAttribution={onOpenAttribution} />);

    const currentYear = new Date().getFullYear();
    const copyrightText = `© ${currentYear} Housou. All rights reserved.`;

    expect(screen.getByText(copyrightText)).toBeInTheDocument();
  });

  it("renders the GitHub link with correct attributes", () => {
    const onOpenAttribution = vi.fn();
    render(<Footer onOpenAttribution={onOpenAttribution} />);

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
    const onOpenAttribution = vi.fn();
    const user = userEvent.setup();

    render(<Footer onOpenAttribution={onOpenAttribution} />);

    const button = screen.getByRole("button", {
      name: /Data Sources & Attribution/i,
    });

    await user.click(button);

    expect(onOpenAttribution).toHaveBeenCalledTimes(1);
  });
});
