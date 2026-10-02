import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { createMetadataClient } from "./metadata";
import type { MetadataRequest, UnifiedMetadata } from "../types";

const metadata: UnifiedMetadata = {
  id: "1",
  title: { native: "A" },
  coverImage: {},
  genres: [],
  studios: [],
  characters: [],
  staff: [],
  episodesList: [],
  isFinished: false,
};
const fetchMock = vi.fn<typeof fetch>();
const clients: ReturnType<typeof createMetadataClient>[] = [];
function client() {
  const value = createMetadataClient();
  clients.push(value);
  return value;
}
function response(init?: RequestInit) {
  const requests = JSON.parse(String(init?.body)) as (MetadataRequest & {
    request_id: string;
  })[];
  return Response.json(
    requests.map((request) => ({ request_id: request.request_id, metadata })),
  );
}
beforeEach(() => {
  vi.useFakeTimers();
  fetchMock.mockReset();
  vi.stubGlobal("fetch", fetchMock);
});
afterEach(() => {
  clients.splice(0).forEach((value) => value.dispose());
  vi.useRealTimers();
  vi.unstubAllGlobals();
});
describe("metadata batching", () => {
  it("deduplicates requests and reuses successful results until expiry", async () => {
    fetchMock.mockImplementation(async (_url, init) => response(init));
    const api = client();
    const first = api.fetchMetadata({ title: "A" });
    expect(api.fetchMetadata({ title: "A" })).toBe(first);
    await vi.advanceTimersByTimeAsync(120);
    await expect(first).resolves.toEqual(metadata);
    expect(api.fetchMetadata({ title: "A" })).toBe(first);
    await vi.advanceTimersByTimeAsync(30 * 60_000 + 1);
    const next = api.fetchMetadata({ title: "A" });
    await vi.advanceTimersByTimeAsync(120);
    await next;
    expect(fetchMock).toHaveBeenCalledTimes(2);
  });
  it("evicts failures so a later request can recover", async () => {
    fetchMock
      .mockResolvedValueOnce(new Response("Offline", { status: 503 }))
      .mockImplementation(async (_url, init) => response(init));
    const api = client();
    const first = api.fetchMetadata({ title: "A" });
    const rejected = expect(first).rejects.toThrow("Offline");
    await vi.advanceTimersByTimeAsync(120);
    await rejected;
    const retry = api.fetchMetadata({ title: "A" });
    await vi.advanceTimersByTimeAsync(120);
    await expect(retry).resolves.toEqual(metadata);
    expect(fetchMock).toHaveBeenCalledTimes(2);
  });
  it("bounds concurrency to two batches and runs queued work immediately after completion", async () => {
    const completions: (() => void)[] = [];
    fetchMock.mockImplementation(
      (_url, init) =>
        new Promise((resolve) =>
          completions.push(() => resolve(response(init))),
        ),
    );
    const api = client();
    const requests = Array.from({ length: 30 }, (_, index) =>
      api.fetchMetadata({ title: String(index) }),
    );
    expect(fetchMock).toHaveBeenCalledTimes(2);
    expect(
      fetchMock.mock.calls.every(
        ([, init]) =>
          (JSON.parse(String(init?.body)) as unknown[]).length <= 10,
      ),
    ).toBe(true);
    completions.shift()!();
    await vi.advanceTimersByTimeAsync(0);
    expect(fetchMock).toHaveBeenCalledTimes(3);
    completions.splice(0).forEach((complete) => complete());
    await Promise.all(requests);
  });
  it("prioritizes a requested detail already waiting in the card queue", async () => {
    const completions: (() => void)[] = [];
    fetchMock.mockImplementation(
      (_url, init) =>
        new Promise((resolve) =>
          completions.push(() => resolve(response(init))),
        ),
    );
    const api = client();
    const requests = Array.from({ length: 31 }, (_, index) =>
      api.fetchMetadata({ title: String(index) }),
    );
    const detail = api.fetchMetadata({ title: "30" }, "detail");
    expect(detail).toBe(requests[30]);
    completions.shift()!();
    await vi.advanceTimersByTimeAsync(0);
    const batch = JSON.parse(
      String(fetchMock.mock.calls[2][1]?.body),
    ) as MetadataRequest[];
    expect(batch[0].title).toBe("30");
    while (completions.length) {
      completions.shift()!();
      await vi.advanceTimersByTimeAsync(0);
    }
    await Promise.all(requests);
  });
  it("rejects queued requests on disposal rather than leaving them pending", async () => {
    const api = client();
    const pending = api.fetchMetadata({ title: "A" });
    const rejected = expect(pending).rejects.toThrow("unmounted");
    api.dispose();
    await rejected;
    await vi.advanceTimersByTimeAsync(120);
    expect(fetchMock).not.toHaveBeenCalled();
  });
});
