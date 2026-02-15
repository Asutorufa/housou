import { http, HttpResponse } from "msw";

export const handlers = [
  // Existing handlers...
  http.get("/api/config", () => {
    return HttpResponse.json({
      site_meta: {},
      years: [2024, 2023],
      attribution: {
        tmdb: {
          logo_square: "",
          logo_long: "",
          logo_alt_long: "",
        },
      },
      auth_enabled: true,
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
    if (body.email === "user@example.com" && body.password === "password") {
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
