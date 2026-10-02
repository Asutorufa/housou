import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import DeferredDialog from "./DeferredDialog";

describe("DeferredDialog", () => {
  it("mounts on first open and keeps the dialog mounted on close", () => {
    const dialog = <div>Dialog shell</div>;
    const { rerender } = render(
      <DeferredDialog open={false}>{dialog}</DeferredDialog>,
    );
    expect(screen.queryByText("Dialog shell")).not.toBeInTheDocument();

    rerender(<DeferredDialog open>{dialog}</DeferredDialog>);
    expect(screen.getByText("Dialog shell")).toBeInTheDocument();

    rerender(<DeferredDialog open={false}>{dialog}</DeferredDialog>);
    expect(screen.getByText("Dialog shell")).toBeInTheDocument();
  });
});
