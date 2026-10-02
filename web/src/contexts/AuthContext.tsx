import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  type ReactNode,
} from "react";
import useSWR, { useSWRConfig } from "swr";
import {
  createAuthApi,
  type ApiFetch,
  type ProfileUpdate,
  type PasswordUpdate,
} from "../api/auth";
import type { LoginData, RegisterData, TelegramAuthData, User } from "../types";
import { ApiError, checkResponse, fetcher } from "../utils/fetcher";
export type { PasskeySummary } from "../api/auth";

interface AuthContextType {
  user: User | undefined;
  loading: boolean;
  loggedIn: boolean;
  login: (data: LoginData) => Promise<void>;
  register: (data: RegisterData) => Promise<void>;
  logout: () => Promise<void>;
  updateProfile: (data: ProfileUpdate) => Promise<User>;
  changePassword: (data: PasswordUpdate) => Promise<void>;
  apiFetch: ApiFetch;
  loginPasskey: () => Promise<void>;
  registerPasskey: ReturnType<typeof createAuthApi>["registerPasskey"];
  listPasskeys: ReturnType<typeof createAuthApi>["listPasskeys"];
  deletePasskey: ReturnType<typeof createAuthApi>["deletePasskey"];
  renamePasskey: ReturnType<typeof createAuthApi>["renamePasskey"];
  bindGithub: () => void;
  unbindGithub: () => Promise<void>;
  loginTelegram: (data: TelegramAuthData) => Promise<void>;
  bindTelegram: (data: TelegramAuthData) => Promise<void>;
  unbindTelegram: () => Promise<void>;
}
const AuthContext = createContext<AuthContextType | undefined>(undefined);

async function fetchSession(): Promise<User | null> {
  try {
    return await fetcher<User>("/api/auth/me");
  } catch (error) {
    if (error instanceof ApiError && error.status === 401) return null;
    throw error;
  }
}

export function AuthProvider({
  children,
  enabled = false,
}: {
  children: ReactNode;
  enabled?: boolean;
}) {
  const {
    data: session,
    mutate,
    isLoading,
  } = useSWR(enabled ? "/api/auth/me" : null, fetchSession, {
    shouldRetryOnError: false,
    revalidateOnFocus: false,
  });
  const { mutate: mutateCache } = useSWRConfig();
  const user = session ?? undefined;

  const clearPrivateCache = useCallback(async () => {
    await mutateCache(
      (key) => Array.isArray(key) && key[0] === "private",
      undefined,
      { revalidate: false },
    );
  }, [mutateCache]);

  const setSession = useCallback(
    async (next: User | null) => {
      if (next?.id !== user?.id) await clearPrivateCache();
      await mutate(next, false);
    },
    [mutate, user?.id, clearPrivateCache],
  );

  const apiFetch = useCallback<ApiFetch>(
    async (url, init) => {
      const res = await fetch(url, init);
      // A 401 can also mean invalid credentials for a mutation. Check the session
      // before clearing it, so an incorrect old password does not log the user out.
      if (
        res.status === 401 &&
        ![
          "/api/auth/login",
          "/api/auth/register",
          "/api/auth/telegram/login",
        ].includes(url)
      ) {
        try {
          const current = await fetchSession();
          await setSession(current);
        } catch {
          /* A failed session check cannot prove that the user logged out. */
        }
      }
      return checkResponse(res);
    },
    [setSession],
  );

  const api = useMemo(() => createAuthApi(apiFetch), [apiFetch]);
  const value = useMemo<AuthContextType>(
    () => ({
      user,
      loading: isLoading,
      loggedIn: !!user,
      apiFetch,
      login: async (data) => {
        await setSession(await api.login(data));
      },
      register: async (data) => {
        await setSession(await api.register(data));
      },
      logout: async () => {
        await api.logout();
        await setSession(null);
      },
      updateProfile: async (data) => {
        const updated = await api.updateProfile(data);
        await setSession(updated);
        return updated;
      },
      changePassword: async (data) => {
        await api.changePassword(data);
        await mutate();
      },
      loginPasskey: async () => {
        await setSession(await api.loginPasskey());
      },
      registerPasskey: api.registerPasskey,
      listPasskeys: api.listPasskeys,
      deletePasskey: api.deletePasskey,
      renamePasskey: api.renamePasskey,
      bindGithub: () => {
        window.location.href = "/api/auth/github/bind";
      },
      unbindGithub: async () => {
        await api.unbindGithub();
        await mutate();
      },
      loginTelegram: async (data) => {
        await setSession(await api.loginTelegram(data));
      },
      bindTelegram: async (data) => {
        await setSession(await api.bindTelegram(data));
      },
      unbindTelegram: async () => {
        await api.unbindTelegram();
        await mutate();
      },
    }),
    [user, isLoading, apiFetch, api, setSession, mutate],
  );
  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

// eslint-disable-next-line react-refresh/only-export-components
export function useAuth() {
  const context = useContext(AuthContext);
  if (!context) throw new Error("useAuth must be used within an AuthProvider");
  return context;
}
