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
let failPosterRepair = true;
let releaseSlowLookup;
const slowLookup = new Promise((resolve) => {
  releaseSlowLookup = resolve;
});
let releaseUnrelatedProviders;
const unrelatedProviders = new Promise((resolve) => {
  releaseUnrelatedProviders = resolve;
});
const directLookupTitles = new Set([
  "Known AniList Title",
  "Known Bangumi Title",
  "Known MAL and Bangumi Title",
  "Known MAL and AniList Title",
]);
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
      if (url.pathname.startsWith("/3/tv/337336") && failPosterRepair)
        return Response.json(
          { error: "temporary unavailable" },
          { status: 503 },
        );
      if (url.pathname === "/3/tv/337335")
        return Response.json({
          id: 337335,
          name: "No poster",
          poster_path: null,
        });
      if (url.pathname === "/3/tv/337335/season/1")
        return Response.json({
          season_number: 1,
          poster_path: null,
          episodes: [],
        });
      if (url.pathname === "/3/tv/337336")
        return Response.json({ id: 337336, poster_path: "/recovered.jpg" });
      if (url.pathname === "/3/tv/337336/season/1")
        return Response.json({
          season_number: 1,
          poster_path: null,
          episodes: [],
        });
      if (url.pathname === "/3/tv/337334")
        return Response.json({
          id: 337334,
          name: "ブラッククローバー",
          poster_path: "/show-poster.jpg",
        });
      if (url.pathname.startsWith("/3/tv/337334/season/")) {
        const season = Number(url.pathname.split("/").at(-1));
        return Response.json({
          id: season,
          season_number: season,
          name: `シーズン ${season}`,
          poster_path: season === 1 ? null : "",
          episodes: [],
        });
      }
      if (directLookupTitles.has(url.searchParams.get("query")))
        await unrelatedProviders;
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
    if (url.host === "api.jikan.moe") {
      const id = Number(url.pathname.split("/")[3]);
      if ([999901, 999905].includes(id))
        return Response.json({
          data: {
            mal_id: id,
            url: `https://myanimelist.net/anime/${id}`,
            title: "Jikan Title",
            title_english: "Jikan Title",
            title_japanese: "日本語タイトル",
            images: {},
            aired: { from: null, to: null },
            studios: [],
            genres: [],
          },
        });
      return Response.json({ error: "Not found" }, { status: 404 });
    }
    if (url.host === "graphql.anilist.co") {
      if ([1001, 1002].includes(body.variables.id))
        return Response.json(
          {
            errors: [{ message: "Not Found.", status: 404 }],
            data: { Media: null },
          },
          { status: 404 },
        );
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
      if (url.pathname === "/v0/subjects/202") await slowLookup;
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

  // A distinct origin guarantees an edge miss, while the D1 row is already
  // populated. The uncached lookup stays blocked until the cached row arrives.
  const start = performance.now();
  const streamed = mf.dispatchFetch("http://stream.housou.test/api/metadata", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/x-ndjson",
    },
    body: JSON.stringify([
      { bangumi_id: "202", request_id: "slow" },
      { bangumi_id: "100", request_id: "cached" },
    ]),
  });
  let deadline;
  try {
    const cached = await Promise.race([
      (async () => {
        const response = await streamed;
        assert.equal(response.status, 200);
        assert.match(
          response.headers.get("Content-Type"),
          /application\/x-ndjson/,
        );
        const reader = response.body.getReader();
        const decoder = new TextDecoder();
        let text = "";
        while (!text.includes("\n")) {
          const { value, done } = await reader.read();
          assert(!done, "stream ended before the cached result arrived");
          text += decoder.decode(value, { stream: true });
        }
        const first = JSON.parse(text.slice(0, text.indexOf("\n")));
        assert.equal(first.request_id, "cached");
        assert.equal(first.metadata.id, "100");
        return { reader, decoder, rest: text.slice(text.indexOf("\n") + 1) };
      })(),
      new Promise((_, reject) => {
        deadline = globalThis.setTimeout(
          () =>
            reject(
              new Error(
                "cached D1 result blocked by unrelated upstream lookup",
              ),
            ),
          1_000,
        );
      }),
    ]);
    console.log(
      `PASS: cached D1 result streamed in ${(performance.now() - start).toFixed(1)}ms while another lookup is blocked`,
    );
    releaseSlowLookup();
    let remaining = cached.rest;
    while (true) {
      const { value, done } = await cached.reader.read();
      if (done) break;
      remaining += cached.decoder.decode(value, { stream: true });
    }
    remaining += cached.decoder.decode();
    const slow = JSON.parse(remaining.trim());
    assert.equal(slow.request_id, "slow");
    assert.equal(slow.metadata.id, "202");
    cached.reader.releaseLock();
  } finally {
    globalThis.clearTimeout(deadline);
    releaseSlowLookup();
  }

  const directLookups = [
    {
      request: { title: "Known AniList Title", anilist_id: "999" },
      source: "aniList",
      id: "999",
    },
    {
      request: { title: "Known Bangumi Title", bangumi_id: "303" },
      source: "bangumi",
      id: "303",
    },
    {
      request: {
        title: "Known MAL and Bangumi Title",
        mal_id: "999901",
        bangumi_id: "304",
      },
      source: "mal",
      id: "999901",
    },
    {
      request: {
        title: "Known MAL and AniList Title",
        mal_id: "999902",
        anilist_id: "999",
      },
      source: "aniList",
      id: "999",
    },
  ];
  try {
    for (const { request, source, id } of directLookups) {
      let deadline;
      const before = calls.length;
      const start = performance.now();
      let metadata;
      try {
        metadata = await Promise.race([
          lookup(request),
          new Promise((_, reject) => {
            deadline = globalThis.setTimeout(
              () =>
                reject(
                  new Error(
                    `${source} ID lookup blocked by an unrelated provider`,
                  ),
                ),
              1_000,
            );
          }),
        ]);
      } finally {
        globalThis.clearTimeout(deadline);
      }
      assert.equal(metadata.sourceSite, source);
      assert.equal(metadata.id, id);
      const directCalls = calls.slice(before);
      assert.equal(
        directCalls.length,
        1,
        "known ID should need one direct request",
      );
      assert.equal(
        directCalls[0].host,
        {
          aniList: "graphql.anilist.co",
          mal: "api.jikan.moe",
          bangumi: "api.bgm.tv",
        }[source],
      );
      console.log(
        `PASS: ${source} known ID returned in ${(performance.now() - start).toFixed(1)}ms without unrelated title searches`,
      );
    }
  } finally {
    releaseUnrelatedProviders();
  }

  let beforeDirectFallback = calls.length;
  const directFallback = await lookup({
    title: "Known Bangumi Fallback Title",
    anilist_id: "1001",
    bangumi_id: "305",
    mal_id: "999903",
  });
  assert.equal(directFallback.sourceSite, "bangumi");
  assert.deepEqual(
    calls.slice(beforeDirectFallback).map((call) => call.host),
    ["graphql.anilist.co", "api.jikan.moe", "api.bgm.tv"],
  );
  assert.equal(calls[beforeDirectFallback].body.variables.id, 1001);

  const beforeFailedTmdb = calls.length;
  const failedTmdb = await lookup({
    tmdb_id: "movie/invalid",
    anilist_id: "1001",
    mal_id: "999905",
    bangumi_id: "307",
  });
  assert.equal(failedTmdb.sourceSite, "mal");
  assert.deepEqual(
    calls.slice(beforeFailedTmdb).map((call) => call.host),
    ["graphql.anilist.co", "api.jikan.moe"],
    "after TMDb failure, try AniList before Jikan and keep Bangumi last",
  );
  console.log(
    "PASS: AniList → Jikan → Bangumi order for known IDs and failed TMDb lookups",
  );

  beforeDirectFallback = calls.length;
  const searchedFallback = await lookup({
    title: "Resolved TMDb Title",
    anilist_id: "1002",
    media_type: "movie",
    year: 2026,
  });
  assert.equal(searchedFallback.sourceSite, "tmdb");
  assert.equal(searchedFallback.id, "movie/42");
  const searchedCalls = calls.slice(beforeDirectFallback);
  const failedIdCalls = searchedCalls.filter(
    (call) => call.host === "graphql.anilist.co",
  );
  assert.equal(
    failedIdCalls.length,
    1,
    "failed known IDs must not be requested twice",
  );
  assert.equal(failedIdCalls[0].body.variables.id, 1002);
  assert(searchedCalls.some((call) => call.host === "api.themoviedb.org"));

  beforeDirectFallback = calls.length;
  const preferred = await lookup({
    tmdb_id: "movie/42",
    anilist_id: "999",
    bangumi_id: "306",
  });
  assert.equal(preferred.sourceSite, "tmdb");
  assert.deepEqual(
    calls.slice(beforeDirectFallback).map((call) => call.path),
    ["/3/movie/42"],
  );
  console.log(
    "PASS: failed known IDs retain direct-ID and title-search fallbacks; explicit TMDb ID remains preferred",
  );

  const directBangumi = await db
    .prepare(
      'SELECT cache_key FROM metadata_cache WHERE metadata_json LIKE \'%"id":"303"%\'',
    )
    .first();
  const directRefresh = await refresh(
    { title: "Known Bangumi Title", bangumi_id: "303" },
    directBangumi.cache_key,
  );
  assert.deepEqual(
    directRefresh.map((call) => call.path),
    ["/v0/subjects/303"],
  );
  console.log(
    "PASS: stale known-ID refresh avoids title searches and unrelated providers",
  );

  for (const season of [1, 2]) {
    const request = { tmdb_id: `tv/337334/season/${season}` };
    const before = calls.length;
    const metadata = await lookup(request);
    assert.equal(metadata.id, request.tmdb_id);
    assert.equal(
      metadata.coverImage.large,
      "https://image.tmdb.org/t/p/w500/show-poster.jpg",
    );
    assert.equal(
      metadata.coverImage.extraLarge,
      "https://image.tmdb.org/t/p/original/show-poster.jpg",
    );
    assert.equal(
      calls.length - before,
      2,
      "fallback uses the already-fetched show details",
    );
    assert.deepEqual((await lookup(request)).coverImage, metadata.coverImage);
    assert.equal(
      calls.length - before,
      2,
      "cache retains the show poster fallback",
    );
    if (season === 2) {
      // Old versions cached a CDN base URL for an empty season poster.
      const key =
        "v2-" +
        createHash("sha256")
          .update(JSON.stringify(["tmdb", request.tmdb_id]))
          .digest("hex");
      const broken = {
        ...metadata,
        coverImage: {
          large: "https://image.tmdb.org/t/p/w500",
          extraLarge: "https://image.tmdb.org/t/p/original",
        },
      };
      await db
        .prepare(
          "UPDATE metadata_cache SET metadata_json = ? WHERE cache_key = ?",
        )
        .bind(JSON.stringify(broken), key)
        .run();
      await edge.put(
        `http://housou.test/__metadata_cache/${key}`,
        new Response(JSON.stringify(broken), {
          headers: {
            "Cache-Control": "public, max-age=3600",
            "X-Housou-Metadata-Expires-At": String(Date.now() + 3_600_000),
          },
        }),
      );
      const repaired = await lookup(request);
      assert.deepEqual(
        repaired.coverImage,
        metadata.coverImage,
        "old malformed edge and D1 covers must be repaired before being returned",
      );
      const afterRepair = calls.length;
      assert.deepEqual((await lookup(request)).coverImage, metadata.coverImage);
      assert.equal(
        calls.length,
        afterRepair,
        "repaired covers remain reusable",
      );
    }
  }
  console.log(
    "PASS: null and empty season posters fall back to the show poster on the real request and cache path",
  );

  // Replay the homepage's known-ID lookup: its old MAL-keyed row resolves to
  // TMDb, but contains no poster even though a direct TMDb lookup has one.
  const homepageRequest = {
    title: "ブラッククローバー 2nd Season",
    mal_id: "61967",
    anilist_id: "195604",
    bangumi_id: "567896",
    year: 2026,
    media_type: "tv",
  };
  const homepageKey =
    "v2-" +
    createHash("sha256")
      .update(JSON.stringify(["mal", homepageRequest.mal_id]))
      .digest("hex");
  const legacy = {
    ...(await lookup({ tmdb_id: "tv/337334/season/1" })),
    coverImage: { large: null, extraLarge: null },
  };
  await db
    .prepare(
      "INSERT INTO metadata_cache (cache_key, metadata_json, source, fetched_at, refresh_after) VALUES (?, ?, 'tmdb', ?, ?)",
    )
    .bind(
      homepageKey,
      JSON.stringify(legacy),
      Date.now(),
      Date.now() + 259_200_000,
    )
    .run();
  await edge.put(
    `http://housou.test/__metadata_cache/${homepageKey}`,
    new Response(JSON.stringify(legacy), {
      headers: {
        "Cache-Control": "public, max-age=3600",
        "X-Housou-Metadata-Expires-At": String(Date.now() + 3_600_000),
      },
    }),
  );
  const beforeHomepageRepair = calls.length;
  const homepage = await lookup(homepageRequest);
  assert.equal(
    homepage.coverImage.large,
    "https://image.tmdb.org/t/p/w500/show-poster.jpg",
    "legacy null covers must be repaired even when both cache layers are fresh",
  );
  assert.deepEqual(
    calls
      .slice(beforeHomepageRepair)
      .map((call) => call.path)
      .sort(),
    ["/3/tv/337334", "/3/tv/337334/season/1"],
    "repair reuses the resolved TMDb ID rather than searching or changing providers",
  );
  const afterHomepageRepair = calls.length;
  assert.equal(
    homepage._housouPosterVersion,
    undefined,
    "cache markers stay internal",
  );
  assert.deepEqual(
    (await lookup(homepageRequest)).coverImage,
    homepage.coverImage,
  );
  assert.equal(calls.length, afterHomepageRepair, "repair must run only once");
  await edge.delete(`http://housou.test/__metadata_cache/${homepageKey}`);
  assert.deepEqual(
    (await lookup(homepageRequest)).coverImage,
    homepage.coverImage,
  );
  assert.equal(
    calls.length,
    afterHomepageRepair,
    "D1 retains the repaired row",
  );
  console.log(
    "PASS: legacy homepage null posters repaired once using the cached TMDb ID",
  );

  async function seedLegacyPoster(id) {
    const key =
      "v2-" +
      createHash("sha256")
        .update(JSON.stringify(["tmdb", id]))
        .digest("hex");
    await db
      .prepare(
        "INSERT INTO metadata_cache (cache_key, metadata_json, source, fetched_at, refresh_after) VALUES (?, ?, 'tmdb', ?, ?)",
      )
      .bind(
        key,
        JSON.stringify({ ...legacy, id }),
        Date.now(),
        Date.now() + 259_200_000,
      )
      .run();
    return key;
  }
  const noPosterRequest = { tmdb_id: "tv/337335/season/1" };
  const noPosterKey = await seedLegacyPoster(noPosterRequest.tmdb_id);
  const beforeMissingPoster = calls.length;
  assert.deepEqual(
    (await lookup(noPosterRequest)).coverImage,
    legacy.coverImage,
  );
  assert.equal(calls.length - beforeMissingPoster, 2);
  const afterMissingPoster = calls.length;
  assert.deepEqual(
    (await lookup(noPosterRequest)).coverImage,
    legacy.coverImage,
  );
  assert.equal(calls.length, afterMissingPoster);
  await edge.delete(`http://housou.test/__metadata_cache/${noPosterKey}`);
  assert.deepEqual(
    (await lookup(noPosterRequest)).coverImage,
    legacy.coverImage,
  );
  assert.equal(
    calls.length,
    afterMissingPoster,
    "confirmed absence stays cached in D1",
  );
  console.log(
    "PASS: genuine poster absence is checked once and cached in both layers",
  );

  const failedRequest = { tmdb_id: "tv/337336/season/1" };
  const failedKey = await seedLegacyPoster(failedRequest.tmdb_id);
  assert.equal((await lookup(failedRequest)).id, failedRequest.tmdb_id);
  const afterFailedRepair = calls.length;
  assert.equal((await lookup(failedRequest)).id, failedRequest.tmdb_id);
  assert.equal(
    calls.length,
    afterFailedRepair,
    "failed repair respects retry backoff",
  );
  const failedRow = await db
    .prepare("SELECT retry_after FROM metadata_cache WHERE cache_key = ?")
    .bind(failedKey)
    .first();
  assert(failedRow.retry_after > Date.now());
  failPosterRepair = false;
  await db
    .prepare("UPDATE metadata_cache SET retry_after = ? WHERE cache_key = ?")
    .bind(Date.now() - 1, failedKey)
    .run();
  assert.equal(
    (await lookup(failedRequest)).coverImage.large,
    "https://image.tmdb.org/t/p/w500/recovered.jpg",
    "legacy poster repair resumes after backoff even with a fresh metadata TTL",
  );
  console.log("PASS: repair failures preserve metadata, back off and recover");

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
  releaseSlowLookup();
  releaseUnrelatedProviders();
  await mf.dispose();
}
