import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { resolve } from "node:path";
import { setTimeout } from "node:timers/promises";
import { fileURLToPath, pathToFileURL } from "node:url";

// Run after worker-build. The optional argument is the installed Wrangler
// package directory, whose existing Miniflare dependency drives the real Wasm.
const require = createRequire(
  process.argv[2] ? resolve(process.argv[2], "package.json") : import.meta.url,
);
const { Miniflare, convertV4MiniflareOptions } = await import(
  pathToFileURL(require.resolve("miniflare"))
);
const buildPath = (name) =>
  fileURLToPath(new URL(`../build/${name}`, import.meta.url));

const calls = [];
const subject = (id) => ({
  id,
  name: "原題",
  name_cn: "测试动画",
  date: "2026-04-01",
  platform: "TV",
  eps: 12,
  total_episodes: 12,
  images: {
    common: "https://example.com/common.jpg",
    large: "https://example.com/large.jpg",
  },
  summary: "summary",
  infobox: [],
  meta_tags: ["动画"],
  rating: { score: 8 },
  nsfw: false,
});
const anime = {
  id: 999,
  title: {
    romaji: "Mapped Title",
    native: "Mapped Title",
    english: "Mapped Title",
  },
  format: "TV",
  status: "FINISHED",
  description: "test",
  coverImage: {},
  siteUrl: "https://anilist.co/anime/999",
  isAdult: false,
  relations: { edges: [] },
  characters: { edges: [] },
  staff: { nodes: [] },
  studios: { nodes: [] },
  externalLinks: [],
  streamingEpisodes: [],
};
const options = {
  modules: [
    {
      type: "ESModule",
      path: buildPath("index.js"),
      contents: readFileSync(buildPath("index.js"), "utf8"),
    },
    {
      type: "CompiledWasm",
      path: buildPath("index_bg.wasm"),
      contents: readFileSync(buildPath("index_bg.wasm")),
    },
  ],
  compatibilityDate: "2024-02-07",
  d1Databases: ["DB"],
  bindings: { TMDB_TOKEN: "test-only" },
  outboundService: async (request) => {
    const url = new URL(request.url);
    const body = request.method === "POST" ? await request.json() : null;
    calls.push({ host: url.host, path: url.pathname, body });
    if (url.host === "api.themoviedb.org") {
      if (url.pathname === "/3/movie/42")
        return Response.json({
          id: 42,
          title: "Resolved TMDb Title",
          status: "Released",
        });
      if (url.searchParams.get("query") === "Resolved TMDb Title")
        return Response.json({
          results: [
            {
              id: 42,
              media_type: "movie",
              title: "Resolved TMDb Title",
              release_date: "2026-04-01",
            },
          ],
        });
      return Response.json({ results: [] });
    }
    if (url.host === "api.jikan.moe")
      return Response.json({ error: "Not found" }, { status: 404 });
    if (url.host === "graphql.anilist.co") {
      if (body.variables.idMal)
        return Response.json(
          {
            errors: [{ message: "Not Found.", status: 404 }],
            data: { Media: null },
          },
          { status: 404 },
        );
      if (body.variables.search === "Mapped Title")
        return Response.json({
          data: {
            Page: {
              media: [
                { id: 999, title: anime.title, format: "TV", seasonYear: 2026 },
              ],
            },
          },
        });
      if (body.variables.id === 999)
        return Response.json({ data: { Media: anime } });
      return Response.json({ data: { Page: { media: [] } } });
    }
    if (url.host === "api.bgm.tv") {
      if (url.pathname === "/v0/subjects/101")
        return Response.json(
          { error: "temporary unavailable" },
          { status: 503 },
        );
      if (url.pathname.startsWith("/v0/subjects/"))
        return Response.json(subject(Number(url.pathname.split("/").at(-1))));
      return Response.json({
        data: body.keyword === "原題" ? [subject(100)] : [],
      });
    }
    throw new Error(`Unexpected outbound host ${url.host}`);
  },
};
const mf = new Miniflare(
  convertV4MiniflareOptions ? convertV4MiniflareOptions(options) : options,
);
async function lookup(request) {
  const response = await mf.dispatchFetch("http://housou.test/api/metadata", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify([{ ...request, request_id: "one" }]),
  });
  assert.equal(response.status, 200);
  const [result] = await response.json();
  if (typeof response.waitUntil === "function") await response.waitUntil();
  return result.metadata;
}
try {
  const first = await lookup({ bangumi_id: "100" });
  assert.equal(first.sourceSite, "bangumi");
  assert.equal(first.id, "100");
  const count = calls.length;
  assert.equal((await lookup({ bangumi_id: "100" })).id, "100");
  assert.equal(calls.length, count, "fresh metadata must avoid providers");
  const db = await mf.getD1Database("DB");
  const entry = await db.prepare("SELECT * FROM metadata_cache").first();
  assert.equal(entry.source, "bangumi");
  assert.equal(entry.refresh_token, null);
  assert.equal(entry.retry_after, null);
  console.log("PASS: Bangumi direct ID persisted in D1 and reused");

  assert.equal(
    await lookup({ title: "Localized Title", year: 2026, media_type: "tv" }),
    null,
  );
  const misses = calls.length;
  assert.equal(
    await lookup({ title: "Localized Title", year: 2026, media_type: "tv" }),
    null,
  );
  assert.equal(
    calls.length,
    misses,
    "negative cache must prevent new provider calls",
  );
  const aliased = await lookup({
    title: "Localized Title",
    aliases: ["原題"],
    year: 2026,
    media_type: "tv",
  });
  assert.equal(aliased.id, "100");
  console.log("PASS: negative D1 caching and alias-enriched recovery");

  const fallback = await lookup({
    title: "Mapped Title",
    mal_id: "123456",
    year: 2026,
    media_type: "tv",
  });
  assert.equal(fallback.sourceSite, "aniList");
  assert.equal(fallback.id, "999");
  assert(
    calls.some(
      (call) =>
        call.host === "graphql.anilist.co" &&
        call.body.variables.idMal === 123456,
    ),
  );
  assert(
    calls.some(
      (call) =>
        call.host === "graphql.anilist.co" &&
        call.body.variables.search === "Mapped Title",
    ),
  );
  console.log(
    "PASS: missing MAL cross-reference retains AniList title fallback",
  );

  assert.equal(await lookup({ bangumi_id: "101" }), null);
  const unavailableKey =
    "v3-" +
    createHash("sha256")
      .update(JSON.stringify(["bangumi", "101"]))
      .digest("hex");
  const unavailable = await db
    .prepare("SELECT retry_after FROM metadata_cache WHERE cache_key = ?")
    .bind(unavailableKey)
    .first();
  assert(
    unavailable.retry_after > Date.now() &&
      unavailable.retry_after - Date.now() <= 300_000,
    "503 must use 5-minute backoff, including ID-only Bangumi lookup",
  );
  console.log(
    "PASS: transient failure backoff applies to the complete provider chain",
  );

  const caches = await mf.getCaches();
  const edge = await caches.open("housou-metadata-v3");
  async function refresh(request, key) {
    const now = Date.now();
    await db
      .prepare(
        "UPDATE metadata_cache SET refresh_after = ?, retry_after = ? WHERE cache_key = ?",
      )
      .bind(now - 1, now - 1, key)
      .run();
    await edge.delete(`http://housou.test/__metadata_cache/${key}`);
    const start = calls.length;
    const stale = await lookup(request);
    assert(
      stale,
      "stale metadata must remain available during background refresh",
    );
    // Dispatch returns before waitUntil work finishes in some Miniflare versions.
    const deadline = Date.now() + 5_000;
    while (true) {
      const row = await db
        .prepare(
          "SELECT refresh_after, refresh_token FROM metadata_cache WHERE cache_key = ?",
        )
        .bind(key)
        .first();
      if (row.refresh_after > Date.now() && row.refresh_token === null) break;
      assert(Date.now() < deadline, "background refresh did not finish");
      await setTimeout(5);
    }
    return calls.slice(start);
  }

  const resolvedRequest = {
    title: "Resolved TMDb Title",
    year: 2026,
    media_type: "movie",
  };
  assert.equal((await lookup(resolvedRequest)).id, "movie/42");
  const resolved = await db
    .prepare("SELECT cache_key FROM metadata_cache WHERE source = 'tmdb'")
    .first();
  const tmdbRefresh = await refresh(resolvedRequest, resolved.cache_key);
  assert.deepEqual(
    tmdbRefresh.map((call) => call.path),
    ["/3/movie/42"],
  );

  const mapped = await db
    .prepare("SELECT cache_key FROM metadata_cache WHERE source = 'anilist'")
    .first();
  const anilistRefresh = await refresh(
    { title: "Mapped Title", mal_id: "123456", year: 2026, media_type: "tv" },
    mapped.cache_key,
  );
  const graphCalls = anilistRefresh.filter(
    (call) => call.host === "graphql.anilist.co",
  );
  assert.equal(graphCalls.length, 1);
  assert.equal(graphCalls[0].body.variables.id, 999);

  const searched = await db
    .prepare(
      "SELECT cache_key FROM metadata_cache WHERE source = 'bangumi' AND cache_key LIKE 'v4-%'",
    )
    .first();
  const bangumiRefresh = await refresh(
    {
      title: "Localized Title",
      aliases: ["原題"],
      year: 2026,
      media_type: "tv",
    },
    searched.cache_key,
  );
  assert.deepEqual(
    bangumiRefresh
      .filter((call) => call.host === "api.bgm.tv")
      .map((call) => call.path),
    ["/v0/subjects/100"],
  );
  console.log(
    "PASS: stale refreshes reuse resolved TMDb, AniList and Bangumi IDs",
  );

  const directKey =
    "v3-" +
    createHash("sha256")
      .update(JSON.stringify(["bangumi", "100"]))
      .digest("hex");
  await edge.put(
    `http://housou.test/__metadata_cache/${directKey}`,
    new Response(JSON.stringify({ ...first, id: "obsolete" }), {
      headers: {
        "Cache-Control": "public, max-age=3600",
        "X-Housou-Metadata-Expires-At": String(Date.now() - 1),
      },
    }),
  );
  const beforeEdgeLookup = calls.length;
  assert.equal((await lookup({ bangumi_id: "100" })).id, "100");
  assert.equal(
    calls.length,
    beforeEdgeLookup,
    "expired L1 must fall through to fresh D1, not providers",
  );
  console.log("PASS: absolute edge expiry checked on the real cache read path");

  const { results } = await db
    .prepare(
      "SELECT source, refresh_after, retry_after FROM metadata_cache WHERE source IS NOT NULL",
    )
    .all();
  assert(results.every((row) => row.refresh_after > Date.now()));
  console.log(
    `PASS: ${results.length} positive D1 rows; ${calls.length} mocked upstream calls`,
  );
} finally {
  await mf.dispose();
}
