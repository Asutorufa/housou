import { http, HttpResponse } from "msw";
import { hashPassword } from "../utils/authUtils";

export const handlers = [
  http.get("/api/config", () => {
    return HttpResponse.json({
      site_meta: {
        crunchyroll: {
          title: "Crunchyroll",
          type: "onair",
          urlTemplate: "https://www.crunchyroll.com/series/{{id}}",
        },
        netflix: { title: "Netflix", type: "onair" },
      },
      years: [2025, 2024, 2023],
      attribution: {
        tmdb: {
          logo_square:
            "https://www.themoviedb.org/assets/2/v4/logos/v2/blue_square_2-d537fb228cf3ded904ef09b136fe3fec72548ebc1fea3fbbd1ad9e36364db38b.svg",
          logo_long:
            "https://www.themoviedb.org/assets/2/v4/logos/v2/blue_short-8e7b30f73a4020692ccca9c88bafe5dcb6f8a62a4c6bc55cd9ba82bb2cd95f6c.svg",
          logo_alt_long:
            "https://www.themoviedb.org/assets/2/v4/logos/v2/blue_long_2-9665a76b1ae401a510ec1e0ca40ddcb3b0cfe45f1d51b77a308fea0845885648.svg",
        },
      },
      auth_enabled: true,
    });
  }),

  http.get("/api/items", () => {
    return HttpResponse.json([
      {
        title: "Test Anime 1",
        type: "tv",
        lang: "ja",
        officialSite: "https://example.com",
        begin: "2024-01-01",
        end: "2024-03-31",
        comment: "",
        sites: [{ site: "crunchyroll", id: "test-1" }],
        titleTranslate: {
          en: ["Test Anime One"],
          "zh-Hans": ["测试动画1"],
        },
      },
      {
        title: "Test Anime 2",
        type: "tv",
        lang: "ja",
        officialSite: "",
        begin: "2024-01-02",
        sites: [],
      },
    ]);
  }),

  http.get("/api/metadata", () => {
    return HttpResponse.json({
      id: "100",
      title: {
        native: "Test Anime 1",
        romaji: "Test Anime 1",
        english: "Test Anime One",
      },
      coverImage: { large: "https://placehold.co/400x600" },
      averageScore: 85,
      episodes: 12,
      genres: ["Action", "Comedy"],
      description: "This is a test description for the anime.",
      studios: ["Studio Test"],
      characters: [],
      staff: [],
      episodesList: [
        { number: 1, title: "Start", airDate: "2024-01-01" },
        { number: 2, title: "Next", airDate: "2024-01-08" },
      ],
      isFinished: false,
      totalSeasons: 1,
      currentSeason: 1,
      runtime: 24,
      contentRating: "PG-13",
    });
  }),

  // Auth Handlers
  http.post("/api/auth/register", async ({ request }) => {
    const body = (await request.json()) as { email: string; username: string };
    if (body.email === "fail@test.com") {
      return HttpResponse.json(
        { error: "Email already taken" },
        { status: 400 },
      );
    }
    return HttpResponse.json({
      id: 1,
      email: body.email,
      username: body.username,
      created_at: Date.now(),
    });
  }),

  http.post("/api/auth/login", async ({ request }) => {
    const body = (await request.json()) as {
      email: string;
      password?: string;
    };
    const expectedHash = await hashPassword("password");

    if (body.email === "user@example.com" && body.password === expectedHash) {
      return HttpResponse.json({
        id: 1,
        email: "user@example.com",
        username: "Test User",
        created_at: Date.now(),
      });
    }
    return HttpResponse.json({ error: "Invalid credentials" }, { status: 401 });
  }),

  http.post("/api/auth/logout", () => {
    return HttpResponse.json({ message: "Logged out" });
  }),

  http.get("/api/auth/me", () => {
    // By default, return null (not logged in) or mock checking cookies
    // For simplicity in generic mock, we might default to 401 or a user if we want auto-login
    // Let's default to 401 Unauthorized for initial state
    return new HttpResponse(null, { status: 401 });
  }),

  http.put("/api/auth/profile", async ({ request }) => {
    const body = (await request.json()) as { username: string };
    return HttpResponse.json({
      id: 1,
      email: "user@example.com",
      username: body.username,
      created_at: Date.now(),
    });
  }),

  // User Item Handlers
  http.get("/api/user/item", ({ request }) => {
    const url = new URL(request.url);
    const itemId = url.searchParams.get("item_id");

    if (itemId === "100") {
      return HttpResponse.json({
        user_id: 1,
        item_id: "100",
        status: 1, // Watching
        score: 8,
        updated_at: Date.now(),
      });
    }
    return new HttpResponse(null, { status: 404 });
  }),

  http.post("/api/user/item", async () => {
    return HttpResponse.json({ message: "Updated" });
  }),
];
