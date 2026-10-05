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
  it("resolves each streamed result before unrelated lookups finish", async () => {
    let stream!: ReadableStreamDefaultController<Uint8Array>;
    fetchMock.mockResolvedValue(
      new Response(
        new ReadableStream({
          start(controller) {
            stream = controller;
          },
        }),
        { headers: { "Content-Type": "application/x-ndjson" } },
      ),
    );
    const api = client();
    const slow = api.fetchMetadata({ title: "Slow" });
    const cached = api.fetchMetadata({ title: "Cached" });
    const cachedSettled = vi.fn();
    const slowSettled = vi.fn();
    void cached.then(cachedSettled, () => {});
    void slow.then(slowSettled, () => {});
    await vi.advanceTimersByTimeAsync(120);
    const requests = JSON.parse(String(fetchMock.mock.calls[0][1]?.body)) as {
      request_id: string;
    }[];
    const encoder = new TextEncoder();
    stream.enqueue(
      encoder.encode(
        JSON.stringify({ request_id: requests[1].request_id, metadata }) + "\n",
      ),
    );
    await vi.advanceTimersByTimeAsync(0);
    try {
      expect(cachedSettled).toHaveBeenCalledWith(metadata);
      expect(slowSettled).not.toHaveBeenCalled();
      expect(api.fetchMetadata({ title: "Cached" }, "detail")).toBe(cached);
    } finally {
      stream.enqueue(
        encoder.encode(
          JSON.stringify({ request_id: requests[0].request_id, metadata }) +
            "\n",
        ),
      );
      stream.close();
      await Promise.allSettled([slow, cached]);
    }
  });
  it.each(["truncated", "failed", "timed out"])(
    "keeps delivered results cached when the stream is %s",
    async (failure) => {
      let stream!: ReadableStreamDefaultController<Uint8Array>;
      fetchMock.mockImplementationOnce(async (_url, init) => {
        init?.signal?.addEventListener("abort", () =>
          stream.error(new Error("aborted")),
        );
        return new Response(
          new ReadableStream({
            start(controller) {
              stream = controller;
            },
          }),
          { headers: { "Content-Type": "application/x-ndjson" } },
        );
      });
      const api = client();
      const cachedRequest = { title: "Cached" };
      const missingRequest = { title: "Missing" };
      const cached = api.fetchMetadata(cachedRequest);
      const missing = api.fetchMetadata(missingRequest);
      const expectedError =
        failure === "truncated"
          ? "Incomplete"
          : failure === "failed"
            ? "Offline"
            : "aborted";
      const rejected = expect(missing).rejects.toThrow(expectedError);
      await vi.advanceTimersByTimeAsync(120);
      const requests = JSON.parse(String(fetchMock.mock.calls[0][1]?.body)) as {
        request_id: string;
      }[];
      stream.enqueue(
        new TextEncoder().encode(
          JSON.stringify({ request_id: requests[0].request_id, metadata }) +
            "\n",
        ),
      );
      await vi.advanceTimersByTimeAsync(0);
      await expect(cached).resolves.toEqual(metadata);
      if (failure === "truncated") stream.close();
      else if (failure === "failed") stream.error(new Error("Offline"));
      else await vi.advanceTimersByTimeAsync(30_000);
      await rejected;
      expect(api.fetchMetadata(cachedRequest)).toBe(cached);
      fetchMock.mockImplementation(async (_url, init) => response(init));
      const retry = api.fetchMetadata(missingRequest);
      await vi.advanceTimersByTimeAsync(120);
      await expect(retry).resolves.toEqual(metadata);
    },
  );
  it("decodes split UTF-8, multiple records per chunk and a final record without a newline", async () => {
    const translated = { ...metadata, title: { native: "日本語" } };
    fetchMock.mockImplementation(async (_url, init) => {
      const requests = JSON.parse(String(init?.body)) as {
        request_id: string;
      }[];
      const encoder = new TextEncoder();
      const first = encoder.encode(
        JSON.stringify({
          request_id: requests[1].request_id,
          metadata: translated,
        }) + "\n",
      );
      const second = encoder.encode(
        JSON.stringify({ request_id: requests[0].request_id, metadata: null }),
      );
      const split = first.findIndex((byte) => byte > 127) + 1;
      const last = new Uint8Array(first.length - split + second.length);
      last.set(first.slice(split));
      last.set(second, first.length - split);
      return new Response(
        new ReadableStream({
          start(controller) {
            controller.enqueue(first.slice(0, split));
            controller.enqueue(last);
            controller.close();
          },
        }),
        { headers: { "Content-Type": "application/x-ndjson; charset=utf-8" } },
      );
    });
    const api = client();
    const missing = api.fetchMetadata({ title: "Missing" });
    const cached = api.fetchMetadata({ title: "Cached" });
    await vi.advanceTimersByTimeAsync(120);
    await expect(missing).resolves.toBeNull();
    await expect(cached).resolves.toEqual(translated);
  });
  it.each([
    {
      field: "Bangumi IDs",
      first: { bangumi_id: "1" },
      second: { bangumi_id: "2" },
    },
    {
      field: "media types",
      first: { media_type: "tv" },
      second: { media_type: "movie" },
    },
    {
      field: "search aliases",
      first: { aliases: ["Original Title"] },
      second: { aliases: ["Another Original Title"] },
    },
  ])(
    "keeps lookups with different $field separate",
    async ({ first, second }) => {
      fetchMock.mockImplementation(async (_url, init) => {
        const requests = JSON.parse(String(init?.body)) as (MetadataRequest & {
          request_id: string;
        })[];
        return Response.json(
          requests.map((request) => ({
            request_id: request.request_id,
            metadata: { ...metadata, id: request.request_id },
          })),
        );
      });
      const api = client();
      const firstRequest = { title: "Shared Title", year: 2026, ...first };
      const secondRequest = { title: "Shared Title", year: 2026, ...second };
      const firstResult = api.fetchMetadata(firstRequest);
      const secondResult = api.fetchMetadata(secondRequest);
      await vi.advanceTimersByTimeAsync(120);
      const [a, b] = await Promise.all([firstResult, secondResult]);
      expect(a?.id).not.toBe(b?.id);
      expect(api.fetchMetadata(firstRequest)).toBe(firstResult);
      expect(api.fetchMetadata(secondRequest)).toBe(secondResult);
    },
  );

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
  it("bounds concurrent list batches and advances queued work while one batch remains slow", async () => {
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
    expect(fetchMock).toHaveBeenCalledTimes(3);

    const detailBatch = JSON.parse(
      String(fetchMock.mock.calls[2][1]?.body),
    ) as MetadataRequest[];
    expect(detailBatch).toHaveLength(1);
    expect(detailBatch[0].title).toBe("30");

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
