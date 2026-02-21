import { describe, it, expect } from 'vitest';
import { getRegionRank, sortSites } from './siteUtils';
import type { SiteMeta } from '../types';

describe('getRegionRank', () => {
  it('should return 1 for JP region', () => {
    expect(getRegionRank({ site: 'test', regions: ['JP'] })).toBe(1);
    expect(getRegionRank({ site: 'test', regions: ['US', 'JP'] })).toBe(1);
  });

  it('should return 2 for empty regions or unknown regions', () => {
    expect(getRegionRank({ site: 'test', regions: [] })).toBe(2);
    expect(getRegionRank({ site: 'test', regions: ['US'] })).toBe(2);
    expect(getRegionRank({ site: 'test', regions: ['KR'] })).toBe(2);
  });

  it('should return 3 for CN, TW, HK, MO regions', () => {
    expect(getRegionRank({ site: 'test', regions: ['CN'] })).toBe(3);
    expect(getRegionRank({ site: 'test', regions: ['TW'] })).toBe(3);
    expect(getRegionRank({ site: 'test', regions: ['HK'] })).toBe(3);
    expect(getRegionRank({ site: 'test', regions: ['MO'] })).toBe(3);
    expect(getRegionRank({ site: 'test', regions: ['US', 'CN'] })).toBe(3);
  });

  it('should use siteMeta if site regions are missing', () => {
    const siteMeta: SiteMeta = {
      test: {
        title: 'Test Site',
        type: 'info',
        regions: ['JP'],
      },
    };
    expect(getRegionRank({ site: 'test' }, siteMeta)).toBe(1);

    const siteMeta2: SiteMeta = {
      test: {
        title: 'Test Site',
        type: 'info',
        regions: ['CN'],
      },
    };
    expect(getRegionRank({ site: 'test' }, siteMeta2)).toBe(3);
  });

  it('should prioritize site regions over siteMeta', () => {
    const siteMeta: SiteMeta = {
      test: {
        title: 'Test Site',
        type: 'info',
        regions: ['CN'],
      },
    };
    // site has JP (rank 1), meta has CN (rank 3) -> should result in rank 1
    expect(getRegionRank({ site: 'test', regions: ['JP'] }, siteMeta)).toBe(1);
  });
});

describe('sortSites', () => {
  const siteMeta: SiteMeta = {
    siteA: { title: 'Apple', type: 'info', regions: ['JP'] },
    siteB: { title: 'Banana', type: 'info', regions: ['US'] },
    siteC: { title: 'Carrot', type: 'info', regions: ['CN'] },
    bangumi: { title: 'Bangumi', type: 'info', regions: ['JP'] },
  };

  it('should sort by region rank (JP < others < CN/TW)', () => {
    const sites = [
      { site: 'siteC', regions: ['CN'] }, // Rank 3
      { site: 'siteB', regions: ['US'] }, // Rank 2
      { site: 'siteA', regions: ['JP'] }, // Rank 1
    ];

    const sorted = sortSites(sites, siteMeta);
    expect(sorted.map(s => s.site)).toEqual(['siteA', 'siteB', 'siteC']);
  });

  it('should sort "bangumi" last within its rank group', () => {
    // Both siteA and bangumi are JP (Rank 1)
    // Even if bangumi (title "Bangumi") is alphabetically before "Zebra", it should be last.
    const sites = [
      { site: 'bangumi', regions: ['JP'] }, // Title: Bangumi
      { site: 'siteZ', regions: ['JP'] }, // Title: Zebra
    ];

    const metaWithZ: SiteMeta = {
        ...siteMeta,
        siteZ: { title: 'Zebra', type: 'info', regions: ['JP'] },
    };

    const sorted = sortSites(sites, metaWithZ);
    expect(sorted.map(s => s.site)).toEqual(['siteZ', 'bangumi']);
  });

  it('should sort by title alphabetically as tie-breaker', () => {
    // Both are US (Rank 2)
    const sites = [
      { site: 'siteB', regions: ['US'] }, // Title: Banana
      { site: 'siteD', regions: ['US'] }, // Title: undefined -> siteD
    ];

    // Add siteD to meta for clear title comparison
    const metaWithD: SiteMeta = {
        ...siteMeta,
        siteD: { title: 'Date', type: 'info', regions: ['US'] }
    };

    const sorted = sortSites(sites, metaWithD);
    expect(sorted.map(s => s.site)).toEqual(['siteB', 'siteD']);

    // Reverse input order
    const sitesRev = [
        { site: 'siteD', regions: ['US'] },
        { site: 'siteB', regions: ['US'] },
    ];
    const sortedRev = sortSites(sitesRev, metaWithD);
    expect(sortedRev.map(s => s.site)).toEqual(['siteB', 'siteD']);
  });

  it('should handle complex sorting scenarios', () => {
    const sites = [
      { site: 'siteC', regions: ['CN'] },       // Rank 3, Title: Carrot
      { site: 'bangumi', regions: ['JP'] },     // Rank 1, but last in group, Title: Bangumi
      { site: 'siteA', regions: ['JP'] },       // Rank 1, Title: Apple
      { site: 'siteB', regions: ['US'] },       // Rank 2, Title: Banana
      { site: 'siteE', regions: ['TW'] },       // Rank 3, Title: Eggplant (assume)
    ];

    const complexMeta: SiteMeta = {
        ...siteMeta,
        siteE: { title: 'Eggplant', type: 'info', regions: ['TW'] }
    };

    // Expected order:
    // Rank 1: siteA (Apple), bangumi (Last in group)
    // Rank 2: siteB (Banana)
    // Rank 3: siteC (Carrot), siteE (Eggplant) -> Alphabetical: Carrot, Eggplant

    const sorted = sortSites(sites, complexMeta);
    expect(sorted.map(s => s.site)).toEqual(['siteA', 'bangumi', 'siteB', 'siteC', 'siteE']);
  });

  it('should fall back to site key if title is missing', () => {
      const sites = [
          { site: 'zebra', regions: ['US'] },
          { site: 'alpha', regions: ['US'] }
      ];
      // No meta provided
      const sorted = sortSites(sites);
      expect(sorted.map(s => s.site)).toEqual(['alpha', 'zebra']);
  });
});
