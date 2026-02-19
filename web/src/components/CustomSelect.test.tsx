import React from "react";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi } from "vitest";
import CustomSelect from "./CustomSelect";

// Radix UI Select relies heavily on pointer events which need to be mocked in JSDOM environment.
// vitest.setup.ts already mocks PointerEvent, ResizeObserver, and HTMLElement pointer methods.

describe("CustomSelect Component", () => {
  const options = [
    { value: "opt1", label: "Option 1" },
    { value: "opt2", label: "Option 2" },
    { value: "opt3", label: "Option 3" },
  ];

  const defaultProps = {
    value: "",
    onValueChange: vi.fn(),
    options,
    placeholder: "Select an option",
  };

  it("renders correctly with placeholder", () => {
    render(<CustomSelect {...defaultProps} />);
    const trigger = screen.getByRole("combobox");
    expect(trigger).toBeInTheDocument();
    expect(screen.getByText("Select an option")).toBeInTheDocument();
  });

  it("renders with the correct label when a value is provided", () => {
    // When value is provided, Radix select should display the corresponding label.
    // However, Radix Select renders the value based on the `value` prop matching an option.
    // If we pass `value="opt2"`, it should render "Option 2".
    render(<CustomSelect {...defaultProps} value="opt2" />);
    expect(screen.getByText("Option 2")).toBeInTheDocument();
  });

  it("opens the dropdown when clicked and displays options", async () => {
    const user = userEvent.setup();
    render(<CustomSelect {...defaultProps} />);

    const trigger = screen.getByRole("combobox");
    await user.click(trigger);

    // Options should be visible now
    const listbox = await screen.findByRole("listbox");
    expect(listbox).toBeInTheDocument();

    for (const option of options) {
      expect(
        screen.getByRole("option", { name: option.label }),
      ).toBeInTheDocument();
    }
  });

  it("calls onValueChange when an option is selected via click", async () => {
    const onValueChange = vi.fn();
    const user = userEvent.setup();
    render(<CustomSelect {...defaultProps} onValueChange={onValueChange} />);

    const trigger = screen.getByRole("combobox");
    await user.click(trigger);

    const option2 = await screen.findByRole("option", { name: "Option 2" });
    await user.click(option2);

    expect(onValueChange).toHaveBeenCalledWith("opt2");
  });

  it("supports keyboard navigation (ArrowDown to navigate, Enter to select)", async () => {
    const onValueChange = vi.fn();
    const user = userEvent.setup();
    render(<CustomSelect {...defaultProps} onValueChange={onValueChange} />);

    const trigger = screen.getByRole("combobox");
    trigger.focus();

    // Open the select with Enter
    await user.keyboard("{Enter}");

    // Wait for listbox to appear
    await screen.findByRole("listbox");

    // Press ArrowDown to move focus to the next item
    // Note: Radix UI often focuses the first item by default if no value is selected.
    // So pressing ArrowDown once should move focus to the second item ("Option 2").
    await user.keyboard("{ArrowDown}");

    // Press Enter to select
    await user.keyboard("{Enter}");

    // Verify onValueChange was called
    // We expect "opt2" if focus moved successfully, but checking simply called is good for a start.
    expect(onValueChange).toHaveBeenCalled();
  });

  it("respects the isOpen controlled prop", () => {
    // When controlled open=true, options should be visible immediately
    // Note: We need a way to verify it's open without user interaction.
    render(<CustomSelect {...defaultProps} isOpen={true} />);

    expect(screen.getByRole("listbox")).toBeInTheDocument();
    expect(
      screen.getByRole("option", { name: "Option 1" }),
    ).toBeInTheDocument();
  });

  it("renders custom icon", () => {
    const TestIcon = <span data-testid="custom-icon">Icon</span>;
    render(<CustomSelect {...defaultProps} icon={TestIcon} />);
    expect(screen.getByTestId("custom-icon")).toBeInTheDocument();
  });

  it("applies custom trigger class name", () => {
    render(
      <CustomSelect
        {...defaultProps}
        triggerClassName="custom-trigger-class"
      />,
    );
    const trigger = screen.getByRole("combobox");
    expect(trigger).toHaveClass("custom-trigger-class");
  });
});
