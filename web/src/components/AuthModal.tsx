import * as Dialog from "@radix-ui/react-dialog";
import * as Tabs from "@radix-ui/react-tabs";
import { Eye, EyeOff, X } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useState } from "react";
import { useAuth } from "../contexts/AuthContext";
import { formInputClassName } from "../styles/uiClasses";
import TelegramLoginButton from "./TelegramLoginButton";

interface AuthModalProps {
  isOpen: boolean;
  onClose: () => void;
  initialTab?: "login" | "register";
  githubEnabled?: boolean;
  telegramBotName?: string;
}

export default function AuthModal({
  isOpen,
  onClose,
  initialTab = "login",
  githubEnabled = false,
  telegramBotName,
}: AuthModalProps) {
  const { login, register, loginPasskey, loginTelegram } = useAuth();
  const [activeTab, setActiveTab] = useState<"login" | "register">(initialTab);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Form states
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [username, setUsername] = useState("");
  const [showPassword, setShowPassword] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setLoading(true);

    try {
      if (activeTab === "login") {
        await login({ email, password });
      } else {
        if (password !== confirmPassword) {
          throw new Error("パスワードが一致しません");
        }
        await register({ email, username, password });
      }
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Authentication failed");
    } finally {
      setLoading(false);
    }
  };

  const handleGithubLogin = () => {
    window.location.href = "/api/auth/github/authorize";
  };

  return (
    <Dialog.Root open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <AnimatePresence>
        {isOpen && (
          <Dialog.Portal forceMount>
            <Dialog.Overlay asChild>
              <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                className="fixed inset-0 z-50 bg-black/50 backdrop-blur-sm"
              />
            </Dialog.Overlay>
            <Dialog.Content asChild>
              <motion.div
                initial={{ opacity: 0, scale: 0.95, x: "-50%", y: "-48%" }}
                animate={{ opacity: 1, scale: 1, x: "-50%", y: "-50%" }}
                exit={{ opacity: 0, scale: 0.95, x: "-50%", y: "-48%" }}
                transition={{ type: "spring", damping: 25, stiffness: 300 }}
                className="fixed left-[50%] top-[50%] z-50 w-full max-w-md rounded-2xl border border-gray-200 bg-white p-6 shadow-xl dark:border-gray-800 dark:bg-gray-900 focus:outline-none"
              >
                <div className="flex items-center justify-between mb-4">
                  <Dialog.Title className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                    {activeTab === "login" ? "ログイン" : "登録"}
                  </Dialog.Title>
                  <Dialog.Close className="rounded-full p-1.5 text-gray-500 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-800">
                    <X size={18} />
                  </Dialog.Close>
                </div>

                <Tabs.Root
                  value={activeTab}
                  onValueChange={(v) => setActiveTab(v as "login" | "register")}
                >
                  <Tabs.List className="mb-6 grid w-full grid-cols-2 rounded-lg bg-gray-100 p-1 dark:bg-gray-800">
                    <Tabs.Trigger
                      value="login"
                      className="rounded-md py-1.5 text-sm font-medium text-gray-600 transition-all data-[state=active]:bg-white data-[state=active]:text-gray-900 data-[state=active]:shadow-sm dark:text-gray-400 dark:data-[state=active]:bg-gray-700 dark:data-[state=active]:text-gray-100"
                    >
                      ログイン
                    </Tabs.Trigger>
                    <Tabs.Trigger
                      value="register"
                      className="rounded-md py-1.5 text-sm font-medium text-gray-600 transition-all data-[state=active]:bg-white data-[state=active]:text-gray-900 data-[state=active]:shadow-sm dark:text-gray-400 dark:data-[state=active]:bg-gray-700 dark:data-[state=active]:text-gray-100"
                    >
                      登録
                    </Tabs.Trigger>
                  </Tabs.List>

                  <form onSubmit={handleSubmit} className="space-y-4">
                    <div>
                      <label className="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300">
                        メールアドレス
                      </label>
                      <input
                        type="email"
                        required
                        value={email}
                        onChange={(e) => setEmail(e.target.value)}
                        className={formInputClassName}
                        placeholder="name@example.com"
                      />
                    </div>

                    {activeTab === "register" && (
                      <div>
                        <label className="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300">
                          ユーザー名
                        </label>
                        <input
                          type="text"
                          required
                          value={username}
                          onChange={(e) => setUsername(e.target.value)}
                          className={formInputClassName}
                          placeholder="Username"
                        />
                      </div>
                    )}

                    <div>
                      <label className="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300">
                        パスワード
                      </label>
                      <div className="relative">
                        <input
                          type={showPassword ? "text" : "password"}
                          required
                          value={password}
                          onChange={(e) => setPassword(e.target.value)}
                          className={`${formInputClassName} pr-10`}
                          placeholder="••••••••"
                        />
                        <button
                          type="button"
                          onClick={() => setShowPassword(!showPassword)}
                          className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200"
                        >
                          {showPassword ? (
                            <EyeOff size={16} />
                          ) : (
                            <Eye size={16} />
                          )}
                        </button>
                      </div>
                    </div>

                    {activeTab === "register" && (
                      <div>
                        <label className="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300">
                          パスワード確認
                        </label>
                        <input
                          type={showPassword ? "text" : "password"}
                          required
                          value={confirmPassword}
                          onChange={(e) => setConfirmPassword(e.target.value)}
                          className={formInputClassName}
                          placeholder="••••••••"
                        />
                      </div>
                    )}

                    {error && (
                      <div className="rounded-lg bg-red-50 p-3 text-sm text-red-500 dark:bg-red-900/20">
                        {error}
                      </div>
                    )}

                    <button
                      type="submit"
                      disabled={loading}
                      className="w-full rounded-lg bg-blue-600 px-4 py-2 text-sm font-semibold text-white transition-colors hover:bg-blue-700 disabled:opacity-50"
                    >
                      {loading
                        ? "読み込み中..."
                        : activeTab === "login"
                          ? "ログイン"
                          : "登録"}
                    </button>
                  </form>

                  <div className="relative my-6">
                    <div className="absolute inset-0 flex items-center">
                      <div className="w-full border-t border-gray-200 dark:border-gray-700" />
                    </div>
                    <div className="relative flex justify-center text-xs uppercase">
                      <span className="bg-white px-2 text-gray-500 dark:bg-gray-900">
                        または
                      </span>
                    </div>
                  </div>

                  {githubEnabled && (
                    <button
                      type="button"
                      onClick={handleGithubLogin}
                      className="flex w-full items-center justify-center gap-2 rounded-lg border border-gray-300 bg-white px-4 py-2 text-sm font-semibold text-gray-700 transition-colors hover:bg-gray-50 dark:border-gray-700 dark:bg-gray-800 dark:text-gray-200 dark:hover:bg-gray-700"
                    >
                      <svg
                        role="img"
                        viewBox="0 0 24 24"
                        xmlns="http://www.w3.org/2000/svg"
                        className="h-5 w-5 fill-current"
                      >
                        <path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12" />
                      </svg>
                      GitHub
                    </button>
                  )}

                  {telegramBotName && (
                    <div className="flex justify-center mt-2">
                      <TelegramLoginButton
                        botName={telegramBotName}
                        cornerRadius={8}
                        buttonSize="large"
                        onAuth={async (user) => {
                          try {
                            setLoading(true);
                            await loginTelegram(user);
                            onClose();
                          } catch (err) {
                            setError(
                              err instanceof Error
                                ? err.message
                                : "Telegram login failed",
                            );
                          } finally {
                            setLoading(false);
                          }
                        }}
                      />
                    </div>
                  )}

                  <button
                    type="button"
                    onClick={async () => {
                      try {
                        setLoading(true);
                        await loginPasskey();
                        onClose();
                      } catch (err) {
                        setError(
                          err instanceof Error
                            ? err.message
                            : "Passkey login failed",
                        );
                      } finally {
                        setLoading(false);
                      }
                    }}
                    className="mt-2 flex w-full items-center justify-center gap-2 rounded-lg border border-gray-300 bg-white px-4 py-2 text-sm font-semibold text-gray-700 transition-colors hover:bg-gray-50 dark:border-gray-700 dark:bg-gray-800 dark:text-gray-200 dark:hover:bg-gray-700"
                  >
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      width="20"
                      height="20"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth="2"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      className="lucide lucide-key-round"
                    >
                      <path d="M2 18v3c0 .6.4 1 1 1h4v-3h3v-3h2l1.4-1.4a6.5 6.5 0 1 0-4-4Z" />
                      <circle cx="16.5" cy="7.5" r=".5" fill="currentColor" />
                    </svg>
                    Passkey
                  </button>
                </Tabs.Root>
              </motion.div>
            </Dialog.Content>
          </Dialog.Portal>
        )}
      </AnimatePresence>
    </Dialog.Root>
  );
}
