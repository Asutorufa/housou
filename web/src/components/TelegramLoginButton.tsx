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

    // Ensure callback namespace exists
    if (!window.TelegramAuthCallbacks) {
      window.TelegramAuthCallbacks = {};
    }

    // Generate unique callback name
    const callbackId = `cb_${Math.random().toString(36).substring(2, 9)}`;
    const callbackName = `window.TelegramAuthCallbacks.${callbackId}`;
    script.setAttribute("data-onauth", `${callbackName}(user)`);

    // Attach global callback
    window.TelegramAuthCallbacks[callbackId] = (user: TelegramAuthData) => {
      onAuthRef.current(user);
    };

    ref.current.appendChild(script);

    return () => {
      // Cleanup unique global function
      if (window.TelegramAuthCallbacks?.[callbackId]) {
        delete window.TelegramAuthCallbacks[callbackId];
      }
    };
  }, [botName, buttonSize, cornerRadius, requestAccess, usePic]);

  return <div ref={ref} className="flex justify-center" />;
}
