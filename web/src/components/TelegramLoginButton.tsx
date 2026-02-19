import { useEffect, useRef } from "react";
import type { TelegramAuthData } from "../types";

interface TelegramLoginButtonProps {
  botName: string;
  onAuth: (user: TelegramAuthData) => void;
  buttonSize?: "large" | "medium" | "small";
  cornerRadius?: number;
  requestAccess?: "write";
  usePic?: boolean;
}

export default function TelegramLoginButton({
  botName,
  onAuth,
  buttonSize = "large",
  cornerRadius,
  requestAccess = "write",
  usePic = true,
}: TelegramLoginButtonProps) {
  const ref = useRef<HTMLDivElement>(null);
  const onAuthRef = useRef(onAuth);

  useEffect(() => {
    onAuthRef.current = onAuth;
  }, [onAuth]);

  useEffect(() => {
    if (!ref.current) return;

    // Remove any existing script inside the container
    while (ref.current.firstChild) {
      ref.current.removeChild(ref.current.firstChild);
    }

    const script = document.createElement("script");
    script.src = "https://telegram.org/js/telegram-widget.js?22";
    script.async = true;
    script.setAttribute("data-telegram-login", botName);
    script.setAttribute("data-size", buttonSize);
    if (cornerRadius !== undefined) {
      script.setAttribute("data-radius", cornerRadius.toString());
    }
    script.setAttribute("data-request-access", requestAccess);
    script.setAttribute("data-userpic", usePic.toString());
    script.setAttribute("data-onauth", "onTelegramAuth(user)");

    // Attach global callback
    window.onTelegramAuth = (user: TelegramAuthData) => {
      onAuthRef.current(user);
    };

    ref.current.appendChild(script);

    return () => {
      // Cleanup global function
      // If multiple buttons exist, this might be problematic, but typically only one auth modal is open.
      if (window.onTelegramAuth) {
        // We can't easily check if it's "our" function, but resetting to undefined is safe enough
        delete window.onTelegramAuth;
      }
    };
  }, [botName, buttonSize, cornerRadius, requestAccess, usePic]);

  return <div ref={ref} className="flex justify-center" />;
}

declare global {
  interface Window {
    onTelegramAuth?: (user: TelegramAuthData) => void;
  }
}
