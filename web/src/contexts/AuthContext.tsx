import { createContext, useContext, ReactNode, useCallback } from "react";
import useSWR from "swr";
import { User } from "../types";

interface AuthContextType {
  user: User | undefined;
  loading: boolean;
  loggedIn: boolean;
  login: (data: any) => Promise<void>;
  register: (data: any) => Promise<void>;
  logout: () => Promise<void>;
  updateProfile: (data: { username: string }) => Promise<User>;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

const fetcher = async (url: string) => {
  const res = await fetch(url);
  if (res.status === 401) {
    const error = new Error("Unauthorized");
    (error as any).status = 401;
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
    error,
    mutate,
    isLoading,
  } = useSWR<User>(enabled ? "/api/auth/me" : null, fetcher, {
    shouldRetryOnError: false,
    revalidateOnFocus: false,
  });

  const loggedIn = !!user;
  // If SWR is loading initial data or revalidating, isLoading is true.
  // We want loading state mainly for initial check.
  const loading = isLoading;

  const login = useCallback(
    async (data: any) => {
      const res = await fetch("/api/auth/login", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(data),
      });
      if (!res.ok) {
        // Try to parse error message
        try {
          const json = await res.json();
          if (json.error) throw new Error(json.error);
        } catch (e) {
          // If json parse fails or no error field, throw generic
          if (e instanceof Error && e.message !== "Login failed") throw e;
        }
        throw new Error("Login failed");
      }
      const user = await res.json();
      mutate(user, false);
    },
    [mutate],
  );

  const register = useCallback(
    async (data: any) => {
      const res = await fetch("/api/auth/register", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(data),
      });
      if (!res.ok) {
        try {
          const json = await res.json();
          if (json.error) throw new Error(json.error);
        } catch (e) {
          if (e instanceof Error && e.message !== "Registration failed")
            throw e;
        }
        throw new Error("Registration failed");
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

export function useAuth() {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error("useAuth must be used within an AuthProvider");
  }
  return context;
}
