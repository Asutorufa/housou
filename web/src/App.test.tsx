import { fireEvent, render, screen } from "@testing-library/react";
import { expect, it, vi } from "vitest";
import App from "./App";
import type { UnifiedMetadata } from "./types";
const route = vi.hoisted(() => ({ search: "" }));
vi.mock("wouter", () => ({
  useSearch: () => route.search,
  useLocation: () => [
    "/",
    (url: string) => {
      route.search = url.split("?")[1] ?? "";
    },
  ],
}));
vi.mock("./hooks/useAnimeData", () => ({
  useAnimeData: () => ({
    config: null,
    selections: {},
    setSelections: vi.fn(),
    searchQuery: "",
    setSearchQuery: vi.fn(),
    items: [],
    filteredItems: [],
    loading: false,
    error: null,
    mutateStatuses: vi.fn(),
  }),
}));
vi.mock("./components/Header", () => ({ default: () => null }));
vi.mock("./components/Footer", () => ({ default: () => null }));
vi.mock("./components/AttributionModal", () => ({ default: () => null }));
vi.mock("./components/TabbedGrid", () => ({
  default: ({
    onOpenModal,
  }: {
    onOpenModal: (title: string, info: Pick<UnifiedMetadata, "id">) => void;
  }) => (
    <button onClick={() => onOpenModal("B", { id: "B-info" })}>open B</button>
  ),
}));
vi.mock("./components/DetailsModal", () => ({
  default: ({
    anime,
  }: {
    anime: { title: string; info: UnifiedMetadata | null };
  }) => <div data-testid="detail">{JSON.stringify(anime)}</div>,
}));
it("does not reuse another anime metadata when URL history changes", async () => {
  const { rerender } = render(<App />);
  fireEvent.click(screen.getByText("open B"));
  expect(await screen.findByTestId("detail")).toHaveTextContent("B-info");
  route.search = "anime=A";
  rerender(<App />);
  expect(screen.getByTestId("detail").textContent).toBe(
    JSON.stringify({ title: "A", info: null }),
  );
});
