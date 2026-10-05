import { describe, expect, it, vi } from "vitest";
import { createAuthApi } from "./auth";
import { hashPassword } from "../utils/authUtils";

function okJson(value: unknown) {
  return Response.json(value);
}

describe("createAuthApi password protocol", () => {
  it("sends raw login passwords with a legacy fallback hash", async () => {
    const apiFetch = vi.fn(async (_url: string, _init?: RequestInit) =>
      okJson({ id: 1 }),
    );
    const api = createAuthApi(apiFetch);

    await api.login({
      email: "user@example.com",
      password: "Password1",
    });

    const [url, init] = apiFetch.mock.calls[0];
    const body = JSON.parse(String(init?.body));
    expect(url).toBe("/api/auth/login");
    expect(body.password).toBe("Password1");
    expect(body.legacy_password_hash).toBe(await hashPassword("Password1"));
  });

  it("registers with the raw password after client-side validation", async () => {
    const apiFetch = vi.fn(async (_url: string, _init?: RequestInit) =>
      okJson({ id: 1 }),
    );
    const api = createAuthApi(apiFetch);

    await api.register({
      email: "user@example.com",
      username: "user",
      password: "Password1",
    });

    const [, init] = apiFetch.mock.calls[0];
    const body = JSON.parse(String(init?.body));
    expect(body.password).toBe("Password1");
    expect(body.legacy_password_hash).toBeUndefined();
  });

  it("rejects weak registration passwords before sending a request", async () => {
    const apiFetch = vi.fn(async (_url: string, _init?: RequestInit) =>
      okJson({ id: 1 }),
    );
    const api = createAuthApi(apiFetch);

    await expect(
      api.register({
        email: "user@example.com",
        username: "user",
        password: "password",
      }),
    ).rejects.toThrow();
    expect(apiFetch).not.toHaveBeenCalled();
  });

  it("sends raw password changes with a legacy hash for the old password", async () => {
    const apiFetch = vi.fn(async (_url: string, _init?: RequestInit) =>
      okJson({ message: "Password updated" }),
    );
    const api = createAuthApi(apiFetch);

    await api.changePassword({
      old_password: "Oldpass1",
      new_password: "Newpass2",
    });

    const [url, init] = apiFetch.mock.calls[0];
    const body = JSON.parse(String(init?.body));
    expect(url).toBe("/api/auth/password");
    expect(body.old_password).toBe("Oldpass1");
    expect(body.new_password).toBe("Newpass2");
    expect(body.legacy_old_password_hash).toBe(await hashPassword("Oldpass1"));
  });
});
