import type {
  PublicKeyCredentialCreationOptionsJSON,
  PublicKeyCredentialRequestOptionsJSON,
} from "@simplewebauthn/browser";
import type { LoginData, RegisterData, TelegramAuthData, User } from "../types";
import { checkResponse } from "../utils/fetcher";
import { validatePasswordComplexity } from "../utils/password";

export type ApiFetch = (url: string, init?: RequestInit) => Promise<Response>;
export interface PasskeySummary {
  id: string;
  name: string;
  createdAt: number;
  lastUsedAt: number;
}
export interface ProfileUpdate {
  username: string;
  email?: string;
  avatar_url?: string;
}
export interface PasswordUpdate {
  old_password?: string;
  new_password: string;
}
interface MessageResponse {
  message: string;
}

export function createAuthApi(apiFetch: ApiFetch) {
  async function request<T>(
    url: string,
    method: string,
    body?: unknown,
  ): Promise<T> {
    const res = await checkResponse(
      await apiFetch(url, {
        method,
        ...(body === undefined
          ? {}
          : {
              headers: { "Content-Type": "application/json" },
              body: JSON.stringify(body),
            }),
      }),
    );
    return res.json() as Promise<T>;
  }
  async function legacyHash(password: string) {
    const { hashPassword } = await import("../utils/authUtils");
    return hashPassword(password);
  }
  return {
    async login(data: LoginData) {
      return request<User>("/api/auth/login", "POST", {
        ...data,
        legacy_password_hash: await legacyHash(data.password),
      });
    },
    async register(data: RegisterData) {
      validatePasswordComplexity(data.password);
      return request<User>("/api/auth/register", "POST", data);
    },
    logout: () => request<MessageResponse>("/api/auth/logout", "POST"),
    updateProfile: (data: ProfileUpdate) =>
      request<User>("/api/auth/profile", "PUT", data),
    async changePassword(data: PasswordUpdate) {
      validatePasswordComplexity(data.new_password);
      return request<MessageResponse>("/api/auth/password", "PUT", {
        ...data,
        legacy_old_password_hash: data.old_password
          ? await legacyHash(data.old_password)
          : undefined,
      });
    },
    async loginPasskey() {
      const { startAuthentication } = await import("@simplewebauthn/browser");
      const optionsJSON = await request<PublicKeyCredentialRequestOptionsJSON>(
        "/api/auth/passkey/login/start",
        "POST",
      );
      const credential = await startAuthentication({ optionsJSON });
      return request<User>(
        "/api/auth/passkey/login/finish",
        "POST",
        credential,
      );
    },
    async registerPasskey(name?: string) {
      const { startRegistration } = await import("@simplewebauthn/browser");
      const optionsJSON = await request<PublicKeyCredentialCreationOptionsJSON>(
        "/api/auth/passkey/register/start",
        "POST",
      );
      const credential = await startRegistration({ optionsJSON });
      await request<MessageResponse>(
        "/api/auth/passkey/register/finish",
        "POST",
        { ...credential, name },
      );
    },
    listPasskeys: () => request<PasskeySummary[]>("/api/auth/passkey", "GET"),
    async deletePasskey(id: string) {
      await request<MessageResponse>(
        `/api/auth/passkey?${new URLSearchParams({ id })}`,
        "DELETE",
      );
    },
    async renamePasskey(id: string, name: string) {
      await request<MessageResponse>("/api/auth/passkey", "PATCH", {
        id,
        name,
      });
    },
    async unbindGithub() {
      await request<MessageResponse>("/api/auth/github", "DELETE");
    },
    loginTelegram: (data: TelegramAuthData) =>
      request<User>("/api/auth/telegram/login", "POST", data),
    bindTelegram: (data: TelegramAuthData) =>
      request<User>("/api/auth/telegram/bind", "POST", data),
    async unbindTelegram() {
      await request<MessageResponse>("/api/auth/telegram", "DELETE");
    },
  };
}
