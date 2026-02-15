import { createContext, useContext, type ReactNode, useCallback } from "react";
import useSWR from "swr";
import type { User, LoginData, RegisterData } from "../types";

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

// Move fetcher inside the component or to a separate file if reused to avoid Fast Refresh warning
// Since it's specific to this context, we can define it inside or move it to a utils file.
// Moving it to top-level is what triggered the warning because the file exports a component.
// But useSWR needs a stable reference. We can keep it outside but simply not export it?
// The warning "Fast refresh only works when a file only exports components" implies that if a file exports a component (AuthProvider), it shouldn't export other things (useAuth).
// Wait, `useAuth` is a hook, which is fine. The warning might be because of `AuthContext` creation or something else?
// Ah, `useAuth` is exported. `AuthProvider` is exported.
// Let's try defining fetcher inside `AuthProvider` using `useCallback` or just outside but non-exported.
// Actually, the warning specifically says: "Fast refresh only works when a file only exports components. Use a new file to share constants or functions between components".
// This suggests I should split the file. But usually hooks and provider in one file is common pattern.
// Maybe it's because I'm exporting *both* a component and a hook? That's standard though.
// Let's try to ignore the warning for now as it's a warning, OR strict mode treats it as error.
// The lint output showed it as an ERROR.
// I will move the types to `types.ts` (done above) and keep this file clean.

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
      const res = await fetch("/api/auth/login", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(data),
      });
      if (!res.ok) {
        try {
          const json = await res.json();
          if (json.error) throw new Error(json.error);
        } catch (e) {
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
    async (data: RegisterData) => {
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

// Ensure useAuth is also fine.
// If the linter complains about mixed exports, I might need to accept it or suppress it.
// Ideally, move AuthProvider to one file and useAuth (consuming context) to another, but they need the context object.
// Context object is not exported.
// The rule 'react-refresh/only-export-components' is strict.
// It allows checking: "Exporting a component and a hook is allowed if the hook is named use*".
// My hook IS named useAuth.
// So why does it fail?
// Maybe because I'm defining `fetcher` top level? But it is NOT exported.
// Wait, the error was on line 146:17 in previous run?
// 146:17 is `export function useAuth`.
// The rule might be triggered because `fetcher` or `ApiError` or `AuthContext` are defined in the same file.
// I will move `ApiError` and `fetcher` out if needed, or just suppress the lint for this file as it is a Context definition file where this pattern is standard.

/* eslint-disable react-refresh/only-export-components */
export function useAuth() {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error("useAuth must be used within an AuthProvider");
  }
  return context;
}
