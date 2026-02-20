import { type ReactNode, useState } from "react";
import { cn } from "../utils/cn";
import { isValidUrl } from "../utils/urlUtils";

interface SiteLinkProps {
  url: string;
  label: string;
  className?: string;
  icon?: ReactNode;
  children?: ReactNode;
  onClick?: (e: React.MouseEvent<HTMLAnchorElement>) => void;
  stopPropagation?: boolean;
}

export default function SiteLink({
  url,
  label,
  className,
  icon,
  children,
  onClick,
  stopPropagation = true,
}: SiteLinkProps) {
  const hostname = url && isValidUrl(url) ? new URL(url).hostname : "";

  const [hasError, setHasError] = useState(false);

  if (!url || !isValidUrl(url)) return null;

  const faviconSrc = hostname
    ? `/api/favicon?domain=${encodeURIComponent(hostname)}`
    : null;

  const handleClick = (e: React.MouseEvent<HTMLAnchorElement>) => {
    if (stopPropagation) {
      e.stopPropagation();
    }
    onClick?.(e);
  };

  return (
    <a
      href={url}
      target="_blank"
      rel="noopener noreferrer"
      onClick={handleClick}
      className={cn(
        "flex items-center gap-1.5 transition-all active:scale-95",
        className,
      )}
    >
      {icon ? (
        icon
      ) : faviconSrc && !hasError ? (
        <img
          src={faviconSrc}
          alt=""
          className="h-3.5 w-3.5 flex-shrink-0"
          loading="lazy"
          onError={() => setHasError(true)}
        />
      ) : null}
      <span className="flex items-center gap-1">{children || label}</span>
    </a>
  );
}
