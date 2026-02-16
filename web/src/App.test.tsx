import { render, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import App from "./App";
import { SWRConfig } from "swr";

// Mock global fetch
const originalFetch = global.fetch;
const mockFetch = vi.fn();

// Mock AuthContext
vi.mock("./contexts/AuthContext", () => ({
  useAuth: () => ({
    loggedIn: false,
    user: undefined,
    apiFetch: vi.fn(),
  }),
}));

const getUrlFromFetchArgs = (args: any[]): string => {
  const input = args[0];
  if (typeof input === "string") {
    return input;
  }
  if (input instanceof Request) {
    return input.url;
  }
  if (input && typeof input === "object" && "url" in input) {
    // Handle URL object or similar
    return input.toString();
  }
  return String(input);
};

describe("App Component Caching", () => {
  beforeEach(() => {
    global.fetch = mockFetch;
    mockFetch.mockImplementation(async (...args) => {
      const urlStr = getUrlFromFetchArgs(args);
      if (urlStr.startsWith("/api/config")) {
        return {
          ok: true,
          json: async () => ({
            years: [2024],
            site_meta: {},
            attribution: { tmdb: {} },
            auth_enabled: false,
          }),
        };
      }
      if (urlStr.startsWith("/api/items")) {
        return {
          ok: true,
          json: async () => [],
        };
      }
      if (urlStr.startsWith("/api/user/status")) {
        return {
          ok: true,
          json: async () => ({}),
        };
      }
      return {
        ok: false,
        statusText: "Not Found",
      };
    });
  });

  afterEach(() => {
    global.fetch = originalFetch;
    vi.clearAllMocks();
  });

  it("fetches config without cache-busting timestamp", async () => {
    render(
      <SWRConfig value={{ provider: () => new Map() }}>
        <App />
      </SWRConfig>,
    );

    await waitFor(() => {
      expect(mockFetch).toHaveBeenCalled();
    });

    const calls = mockFetch.mock.calls;
    // Find the call for config
    const configCall = calls.find((call) => {
      const url = getUrlFromFetchArgs(call);
      return url.startsWith("/api/config");
    });

    expect(configCall).toBeDefined();
    // This assertion should fail if the timestamp is present
    if (configCall) {
      expect(getUrlFromFetchArgs(configCall)).toBe("/api/config");
    }
  });
});
