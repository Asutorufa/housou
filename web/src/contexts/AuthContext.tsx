import { createContext, useContext, type ReactNode, useCallback } from "react";
import useSWR from "swr";
import type { User, LoginData, RegisterData } from "../types";
import { hashPassword } from "../utils/authUtils";

interface AuthContextType {
  user: User | undefined;
  loading: boolean;
  loggedIn: boolean;
  login: (data: LoginData) => Promise<void>;
  register: (data: RegisterData) => Promise<void>;
  logout: () => Promise<void>;
  updateProfile: (data: { username: string }) => Promise<User>;
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

  const logout = useCallback(async () => {
    await fetch("/api/auth/logout", { method: "POST" });
    mutate(undefined, false);
  }, [mutate]);

  const updateProfile = useCallback(
    async (data: { username: string }) => {
      const res = await fetch("/api/auth/profile", {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(data),
      });
      if (!res.ok) throw new Error("Update failed");
      const user = await res.json();
      mutate(user, false);
      return user;
    },
    [mutate],
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
