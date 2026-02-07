export const INTERNATIONAL_SITES = ['netflix', 'amazon', 'primevideo','prime', 'disney', 'hulu', 'crunchyroll', 'funimation', 'apple', 'google', 'itunes'];
export const JAPANESE_SITES = ['abema', 'unext', 'dazn', 'nicovideo', 'danime', 'bandaichannel', 'dmm', 'fod', 'paravi', 'telasa', 'wowow', 'videomarket', 'musicjp'];
export const CHINESE_SITES = ['bilibili', 'bilibili_hk_mo_tw', 'bilibili_hk_mo', 'bilibili_tw', 'iqiyi', 'qq', 'youku', 'letv', 'pptv', 'mgtv', 'acfun', 'sohu', 'tudou'];

export function getSiteRank(site: string): number {
    const s = site.toLowerCase();
    if (JAPANESE_SITES.some(p => s.includes(p))) return 1;
    if (INTERNATIONAL_SITES.some(p => s.includes(p))) return 2;
    if (CHINESE_SITES.some(p => s.includes(p))) return 4;
    return 3; // Default for others
}

export function sortSites<T extends { site: string }>(sites: T[]): T[] {
    return [...sites].sort((a, b) => {
        const rankA = getSiteRank(a.site);
        const rankB = getSiteRank(b.site);
        
        if (rankA !== rankB) return rankA - rankB;
        
        // Secondary sort by site name if ranks are equal
        return a.site.localeCompare(b.site);
    });
}
