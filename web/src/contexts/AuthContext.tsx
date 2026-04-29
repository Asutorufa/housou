import {
  startAuthentication,
  startRegistration,
} from "@simplewebauthn/browser";
import { createContext, useCallback, useContext, type ReactNode } from "react";
import useSWR from "swr";
import type { LoginData, RegisterData, TelegramAuthData, User } from "../types";
import { hashPassword, validatePasswordComplexity } from "../utils/authUtils";
import { fetcher } from "../utils/fetcher";

export interface PasskeySummary {
  id: string;
  name: string;
  createdAt: number;
  lastUsedAt: number;
}

interface AuthContextType {
  user: User | undefined;
  loading: boolean;
  loggedIn: boolean;
  login: (data: LoginData) => Promise<void>;
  register: (data: RegisterData) => Promise<void>;
  logout: () => Promise<void>;
  updateProfile: (data: {
    username: string;
    email?: string;
    avatar_url?: string;
  }) => Promise<User>;
  changePassword: (data: {
    old_password?: string;
    new_password: string;
  }) => Promise<void>;
  apiFetch: (url: string, init?: RequestInit) => Promise<Response>;
  loginPasskey: () => Promise<void>;
  registerPasskey: (name?: string) => Promise<void>;
  listPasskeys: () => Promise<PasskeySummary[]>;
  deletePasskey: (id: string) => Promise<void>;
  renamePasskey: (id: string, name: string) => Promise<void>;
  bindGithub: () => void;
  unbindGithub: () => Promise<void>;
  loginTelegram: (data: TelegramAuthData) => Promise<void>;
  bindTelegram: (data: TelegramAuthData) => Promise<void>;
  unbindTelegram: () => Promise<void>;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

const handleResponse = async (res: Response, defaultError: string) => {
  if (!res.ok) {
    let message = defaultError;
    try {
      const json = await res.json();
      if (json.error) message = json.error;
    } catch {
      // Ignore
    }
    throw new Error(message);
  }
  return res.json();
};

export function AuthProvider({
  children,
  enabled = false,
}: {
  children: ReactNode;
  enabled?: boolean;
}) {
  const {
    data: user,
    mutate,
    isLoading,
  } = useSWR<User>(enabled ? "/api/auth/me" : null, fetcher, {
    shouldRetryOnError: false,
    revalidateOnFocus: false,
  });

  const loggedIn = !!user;
  const loading = isLoading;

  const login = useCallback(
    async (data: LoginData) => {
      const res = await fetch("/api/auth/login", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(data),
      });
      const user = await handleResponse(res, "Login failed");
      mutate(user, false);
    },
    [mutate],
  );

  const register = useCallback(
    async (data: RegisterData) => {
      validatePasswordComplexity(data.password);
      const res = await fetch("/api/auth/register", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(data),
      });
      const user = await handleResponse(res, "Registration failed");
      mutate(user, false);
    },
    [mutate],
  );

  const apiFetch = useCallback(
    async (url: string, init?: RequestInit) => {
      const res = await fetch(url, init);
      if (res.status === 401) {
        mutate(undefined, false);
      }
      return res;
    },
    [mutate],
  );

  const logout = useCallback(async () => {
    await apiFetch("/api/auth/logout", { method: "POST" });
    mutate(undefined, false);
  }, [apiFetch, mutate]);

  const updateProfile = useCallback(
    async (data: { username: string; email?: string; avatar_url?: string }) => {
      const res = await apiFetch("/api/auth/profile", {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(data),
      });
      const user = await handleResponse(res, "Update failed");
      mutate(user, false);
      return user;
    },
    [apiFetch, mutate],
  );

  const changePassword = useCallback(
    async (data: { old_password?: string; new_password: string }) => {
      validatePasswordComplexity(data.new_password);

      const res = await apiFetch("/api/auth/password", {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(data),
      });

      await handleResponse(res, "Password update failed");
    },
    [apiFetch],
  );

  const loginPasskey = useCallback(async () => {
    // 1. Get options from server
    const resp = await fetch("/api/auth/passkey/login/start", {
      method: "POST",
    });
    if (!resp.ok) throw new Error("Failed to start passkey login");
    const options = await resp.json();

    // 2. Pass options to browser
    let asseResp;
    try {
      asseResp = await startAuthentication(options);
    } catch (error) {
      console.error(error);
      throw error;
    }

    // 3. Send response to server
    const verificationResp = await fetch("/api/auth/passkey/login/finish", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(asseResp),
    });

    if (!verificationResp.ok) {
      const json = await verificationResp.json().catch(() => ({}));
      throw new Error(json.error || "Passkey verification failed");
    }

    const user = await verificationResp.json();
    mutate(user, false);
  }, [mutate]);

  const registerPasskey = useCallback(async (name?: string) => {
    const resp = await fetch("/api/auth/passkey/register/start", {
      method: "POST",
    });
    if (!resp.ok) throw new Error("Failed to start passkey registration");
    const options = await resp.json();

    let attResp;
    try {
      attResp = await startRegistration(options);
    } catch (error) {
      console.error(error);
      throw error;
    }

    const verificationResp = await fetch("/api/auth/passkey/register/finish", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ ...attResp, name }),
    });

    if (!verificationResp.ok) {
      const json = await verificationResp.json().catch(() => ({}));
      throw new Error(json.error || "Passkey registration failed");
    }
  }, []);

  const listPasskeys = useCallback(async () => {
    const res = await fetch("/api/auth/passkey");
    if (!res.ok) throw new Error("Failed to list passkeys");
    return res.json();
  }, []);

  const deletePasskey = useCallback(async (id: string) => {
    const res = await fetch(`/api/auth/passkey?id=${id}`, {
      method: "DELETE",
    });
    if (!res.ok) throw new Error("Failed to delete passkey");
  }, []);

  const renamePasskey = useCallback(async (id: string, name: string) => {
    const res = await fetch("/api/auth/passkey", {
      method: "PATCH",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ id, name }),
    });
    if (!res.ok) throw new Error("Failed to rename passkey");
  }, []);

  const bindGithub = useCallback(() => {
    window.location.href = "/api/auth/github/bind";
  }, []);

  const unbindGithub = useCallback(async () => {
    const res = await apiFetch("/api/auth/github", {
      method: "DELETE",
    });
    await handleResponse(res, "Unbind failed");
    mutate();
  }, [apiFetch, mutate]);

  const loginTelegram = useCallback(
    async (data: TelegramAuthData) => {
      const res = await fetch("/api/auth/telegram/login", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(data),
      });
      const user = await handleResponse(res, "Login failed");
      mutate(user, false);
    },
    [mutate],
  );

  const bindTelegram = useCallback(
    async (data: TelegramAuthData) => {
      const res = await apiFetch("/api/auth/telegram/bind", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(data),
      });
      const user = await handleResponse(res, "Bind failed");
      mutate(user, false);
    },
    [apiFetch, mutate],
  );

  const unbindTelegram = useCallback(async () => {
    const res = await apiFetch("/api/auth/telegram", {
      method: "DELETE",
    });
    await handleResponse(res, "Unbind failed");
    mutate();
  }, [apiFetch, mutate]);

  return (
    <AuthContext.Provider
      value={{
        user,
        loading,
        loggedIn,
        login,
        register,
        logout,
        updateProfile,
        changePassword,
        apiFetch,
        loginPasskey,
        registerPasskey,
        listPasskeys,
        deletePasskey,
        renamePasskey,
        bindGithub,
        unbindGithub,
        loginTelegram,
        bindTelegram,
        unbindTelegram,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}

/* eslint-disable react-refresh/only-export-components */
export function useAuth() {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error("useAuth must be used within an AuthProvider");
  }
  return context;
}
