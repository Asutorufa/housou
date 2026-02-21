import { TelegramAuthData } from "./types";

declare global {
  interface Window {
    TelegramAuthCallbacks?: {
      [key: string]: (user: TelegramAuthData) => void;
    };
  }
}
