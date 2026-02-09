import type { SiteMeta } from "../types";

export function getRegionRank(
  site: { site: string; regions?: string[] },
  siteMeta?: SiteMeta,
): number {
  const regions = site.regions || siteMeta?.[site.site]?.regions || [];

  if (regions.includes("JP")) return 1;
  if (regions.length === 0) return 2;
  if (regions.some((r) => ["CN", "TW", "HK", "MO"].includes(r))) return 3;

  return 2; // Default to middle for other regions
}

export function sortSites<T extends { site: string; regions?: string[] }>(
  sites: T[],
  siteMeta?: SiteMeta,
): T[] {
  return [...sites].sort((a, b) => {
    // 1. Sort by region rank
    const regionRankA = getRegionRank(a, siteMeta);
    const regionRankB = getRegionRank(b, siteMeta);
    if (regionRankA !== regionRankB) return regionRankA - regionRankB;

    // 2. Special case: bangumi always last within its group
    if (a.site === "bangumi" && b.site !== "bangumi") return 1;
    if (a.site !== "bangumi" && b.site === "bangumi") return -1;

    // 3. Sort by display title (if available) or site key
    const titleA = siteMeta?.[a.site]?.title || a.site;
    const titleB = siteMeta?.[b.site]?.title || b.site;

    return titleA.localeCompare(titleB);
  });
}
