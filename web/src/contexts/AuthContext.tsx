import { startAuthentication, startRegistration } from "@simplewebauthn/browser";
import { createContext, useCallback, useContext, type ReactNode } from "react";
import useSWR from "swr";
import type { LoginData, Passkey, RegisterData, User } from "../types";
import { hashPassword } from "../utils/authUtils";

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
  registerPasskey: (name?: string) => Promise<void>;
  loginPasskey: (email: string) => Promise<void>;
  getPasskeys: () => Promise<Passkey[]>;
  deletePasskey: (id: number) => Promise<void>;
}

// Separate Error type for API responses
class ApiError extends Error {
  status?: number;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

const fetcher = async (url: string) => {
  const res = await fetch(url);
  if (res.status === 401) {
    const error = new ApiError("Unauthorized");
    error.status = 401;
    throw error;
  }
  if (!res.ok) throw new Error("Failed to fetch user");
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
      const hashedPassword = await hashPassword(data.password);
      const res = await fetch("/api/auth/login", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ ...data, password: hashedPassword }),
      });
      if (!res.ok) {
        let message = "Login failed";
        try {
          const json = await res.json();
          if (json.error) {
            message = json.error;
          }
        } catch {
          // Ignore if parsing fails, use default message
        }
        throw new Error(message);
      }
      const user = await res.json();
      mutate(user, false);
    },
    [mutate],
  );

  const register = useCallback(
    async (data: RegisterData) => {
      const hashedPassword = await hashPassword(data.password);
      const res = await fetch("/api/auth/register", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ ...data, password: hashedPassword }),
      });
      if (!res.ok) {
        let message = "Registration failed";
        try {
          const json = await res.json();
          if (json.error) {
            message = json.error;
          }
        } catch {
          // Ignore if parsing fails
        }
        throw new Error(message);
      }
      const user = await res.json();
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
      if (!res.ok) {
        let message = "Update failed";
        try {
          const json = await res.json();
          if (json.error) {
            message = json.error;
          }
        } catch {
          // Ignore
        }
        throw new Error(message);
      }
      const user = await res.json();
      mutate(user, false);
      return user;
    },
    [apiFetch, mutate],
  );

  const changePassword = useCallback(
    async (data: { old_password?: string; new_password: string }) => {
      const hashedOld = data.old_password
        ? await hashPassword(data.old_password)
        : undefined;
      const hashedNew = await hashPassword(data.new_password);

      const res = await apiFetch("/api/auth/password", {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          old_password: hashedOld,
          new_password: hashedNew,
        }),
      });

      if (!res.ok) {
        let message = "Password update failed";
        try {
          const json = await res.json();
          if (json.error) {
            message = json.error;
          }
        } catch {
          // Ignore
        }
        throw new Error(message);
      }
    },
    [apiFetch],
  );

  const registerPasskey = useCallback(
    async (name?: string) => {
      const resp = await apiFetch("/api/auth/passkey/register/start", {
        method: "POST",
      });
      if (!resp.ok) throw new Error("Failed to start registration");
      const { state_id, options } = await resp.json();

      const attResp = await startRegistration({ optionsJSON: options });

      const finishResp = await apiFetch("/api/auth/passkey/register/finish", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ state_id, register_response: attResp, name }),
      });
      if (!finishResp.ok) throw new Error("Failed to finish registration");
    },
    [apiFetch],
  );

  const loginPasskey = useCallback(
    async (email: string) => {
      const resp = await fetch("/api/auth/passkey/login/start", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email }),
      });
      if (!resp.ok) throw new Error("Failed to start login");
      const { state_id, options } = await resp.json();

      const asseResp = await startAuthentication({ optionsJSON: options });

      const finishResp = await fetch("/api/auth/passkey/login/finish", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ state_id, login_response: asseResp }),
      });
      if (!finishResp.ok) throw new Error("Failed to finish login");

      const user = await finishResp.json();
      mutate(user, false);
    },
    [mutate],
  );

  const getPasskeys = useCallback(async () => {
    const resp = await apiFetch("/api/user/passkeys");
    if (!resp.ok) throw new Error("Failed to fetch passkeys");
    return resp.json();
  }, [apiFetch]);

  const deletePasskey = useCallback(
    async (id: number) => {
      const resp = await apiFetch(`/api/user/passkeys/${id}`, {
        method: "DELETE",
      });
      if (!resp.ok) throw new Error("Failed to delete passkey");
    },
    [apiFetch],
  );

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
        registerPasskey,
        loginPasskey,
        getPasskeys,
        deletePasskey,
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
