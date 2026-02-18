import { ExternalLink } from "lucide-react";
import type { DisplayAnimeItem, Site, SiteMeta } from "../../types";
import { isValidUrl } from "../../utils/urlUtils";

interface ExternalLinksProps {
  originalItem?: DisplayAnimeItem;
  siteMeta?: SiteMeta;
  sites: Site[];
}

export default function ExternalLinks({
  originalItem,
  siteMeta,
  sites,
}: ExternalLinksProps) {
  return (
    <>
      {originalItem?.officialSite && isValidUrl(originalItem.officialSite) && (
        <div className="mb-4">
          <h4 className="mb-2 text-sm font-black tracking-wider text-gray-400 uppercase dark:text-gray-500">
            公式サイト
          </h4>
          <a
            href={originalItem.officialSite}
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-1.5 rounded-xl border border-purple-100 bg-purple-50 px-3 py-1.5 text-sm font-bold text-purple-600 transition-colors hover:bg-purple-100 dark:border-purple-800/50 dark:bg-purple-900/20 dark:text-purple-400 dark:hover:bg-purple-900/30"
          >
            公式サイト
            <ExternalLink size={12} />
          </a>
        </div>
      )}
      {Object.entries({
        onair: {
          label: "配信",
          sites: sites.filter((s) => siteMeta?.[s.site]?.type === "onair"),
        },
        info: {
          label: "情報",
          sites: sites.filter((s) => siteMeta?.[s.site]?.type === "info"),
        },
        resource: {
          label: "リソース",
          sites: sites.filter((s) => siteMeta?.[s.site]?.type === "resource"),
        },
        other: {
          label: "その他",
          sites: sites.filter(
            (s) =>
              !siteMeta?.[s.site]?.type ||
              !["onair", "info", "resource"].includes(siteMeta[s.site]!.type),
          ),
        },
      }).map(([key, group]) => {
        if (group.sites.length === 0) return null;

        return (
          <div key={key}>
            <h4 className="mb-2 text-sm font-black tracking-wider text-gray-400 uppercase dark:text-gray-500">
              {group.label}
            </h4>
            <div className="mb-4 flex flex-wrap gap-2">
              {group.sites.map((site, idx) => {
                const meta = siteMeta?.[site.site];
                const url =
                  site.url ||
                  meta?.urlTemplate?.replace("{{id}}", site.id || "");
                if (!url || !isValidUrl(url)) return null;

                return (
                  <a
                    key={`${site.site}-${idx}`}
                    href={url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="flex items-center gap-1.5 rounded-xl border border-blue-100 bg-blue-50 px-3 py-1.5 text-sm font-bold text-blue-600 transition-colors hover:bg-blue-100 dark:border-blue-800/50 dark:bg-blue-900/20 dark:text-blue-400 dark:hover:bg-blue-900/30"
                  >
                    {meta?.title || site.site}
                    <ExternalLink size={12} />
                  </a>
                );
              })}
            </div>
          </div>
        );
      })}
    </>
  );
}
