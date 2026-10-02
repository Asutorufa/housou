import { render, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import ProfileModal from "./index";
vi.mock("../../contexts/AuthContext", () => ({
  useAuth: () => ({
    user: { id: 1, email: "a@example.com", username: "A" },
    updateProfile: vi.fn(),
  }),
}));
afterEach(() => vi.unstubAllGlobals());
it("observes content when an initially closed profile opens and observes it again after reopening", async () => {
  const observe = vi.fn();
  const disconnect = vi.fn();
  vi.stubGlobal(
    "ResizeObserver",
    class {
      observe = observe;
      disconnect = disconnect;
    },
  );
  const { rerender } = render(
    <ProfileModal isOpen={false} onClose={() => {}} />,
  );
  expect(observe).not.toHaveBeenCalled();
  rerender(<ProfileModal isOpen onClose={() => {}} />);
  await waitFor(() => expect(observe).toHaveBeenCalledTimes(1));
  rerender(<ProfileModal isOpen={false} onClose={() => {}} />);
  await waitFor(() => expect(disconnect).toHaveBeenCalled());
  rerender(<ProfileModal isOpen onClose={() => {}} />);
  await waitFor(() => expect(observe).toHaveBeenCalledTimes(2));
});
