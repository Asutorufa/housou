import { http, HttpResponse } from 'msw'

const mockConfig = {
  years: [2024, 2025],
  site_meta: {
    bilibili: { title: 'Bilibili', urlTemplate: 'https://www.bilibili.com/bangumi/play/ss{{id}}' },
    netflix: { title: 'Netflix', urlTemplate: 'https://www.netflix.com/title/{{id}}' },
    amazon: { title: 'Prime Video', urlTemplate: 'https://www.amazon.co.jp/dp/{{id}}' },
    abema: { title: 'Abema', urlTemplate: 'https://abema.tv/video/title/{{id}}' },
    crunchyroll: { title: 'Crunchyroll', urlTemplate: 'https://www.crunchyroll.com/series/{{id}}' },
  },
  attribution: {
    tmdb: {
      logo_square: "https://www.themoviedb.org/assets/2/v4/logos/v2/blue_square_2-d537fb228cf3ed904132c3096b9736928c38cfe75196763ebd7e9f22e855d9e5.svg",
      logo_long: "https://www.themoviedb.org/assets/2/v4/logos/v2/blue_short-8e7b30f73a4020692ccca9c88bafe5dcb6f8a62a4c6bc55cd9ba82bb2cd95f6c.svg",
      logo_alt_long: "https://www.themoviedb.org/assets/2/v4/logos/v2/blue_long_2-9665a76b1ae401a510ec1e0ca40ddcb3b0cfe45f1d51b77a308fea0845885648.svg"
    }
  }
}

const mockItems = [
  // Sunday (0)
  {
    title: 'One Piece',
    begin: '2023-10-01T09:30:00Z', // Sunday
    sites: [
      { site: 'crunchyroll', id: 'op', type: 'onair' },
      { site: 'netflix', id: 'op-n', type: 'onair' }
    ],
    titleTranslate: { 'en': ['One Piece'] }
  },
  {
    title: 'Shangri-La Frontier',
    begin: '2023-10-01T17:00:00Z', // Sunday
    sites: [
        { site: 'crunchyroll', id: 'slf', type: 'onair' },
        { site: 'bilibili', id: 'slf-b', type: 'onair' }
    ],
    titleTranslate: { 'en': ['Shangri-La Frontier'] }
  },
  
  // Monday (1)
  {
    title: 'Ron Kamonohashi\'s Forbidden Deductions',
    begin: '2023-10-02T23:00:00Z', // Monday
    sites: [
        { site: 'crunchyroll', id: 'ron', type: 'onair' },
        { site: 'netflix', id: 'ron-n', type: 'onair' }
    ]
  },
  
  // Tuesday (2)
  {
    title: 'Paradox Live THE ANIMATION',
    begin: '2023-10-03T23:00:00Z', // Tuesday
    sites: [
        { site: 'abema', id: 'para', type: 'onair' }
    ]
  },

  // Wednesday (3)
  {
    title: 'Kage no Jitsuryokusha ni Naritakute! 2nd Season',
    begin: '2023-10-04T22:30:00Z', // Wednesday
    sites: [
        { site: 'hidive', id: 'shadow', type: 'onair' },
        { site: 'bilibili', id: 'shadow-b', type: 'onair' }
    ]
  },

  // Thursday (4)
  {
    title: 'Dr. STONE NEW WORLD Part 2',
    begin: '2023-10-12T22:30:00Z', // Thursday
    sites: [
        { site: 'crunchyroll', id: 'stone', type: 'onair' },
        { site: 'netflix', id: 'stone-n', type: 'onair' }
    ]
  },
  
  // Friday (5)
  {
    title: '葬送のフリーレン',
    begin: '2023-09-29T23:00:00Z', // Friday
    sites: [
      { site: 'bilibili', id: '123', type: "onair" },
      { site: 'netflix', id: '456', type: "onair" },
      { site: 'amazon', id: '789', type: "onair" },
      { site: 'abema', id: '101', type: "onair" },
      { site: 'crunchyroll', id: 'CR1', type: "onair" }
    ],
    titleTranslate: {
      'zh-Hans': ['葬送的芙莉莲']
    }
  },
  {
    title: 'Undead Unluck',
    begin: '2023-10-06T01:28:00Z', // Friday
    sites: [
        { site: 'hulu', id: 'uu', type: 'onair' }
    ]
  },
  {
    title: 'Goblin Slayer II',
    begin: '2023-10-06T22:00:00Z', // Friday
    sites: [
        { site: 'crunchyroll', id: 'gs', type: 'onair' },
        { site: 'amazon', id: 'gs-a', type: 'onair' }
    ]
  },

  // Saturday (6)
  {
    title: '薬屋のひとりごと',
    begin: '2023-10-21T01:05:00Z', // Saturday
    sites: [
      { site: 'netflix', id: '789', type: "tv" },
      { site: 'bilibili', id: '2233', type: "onair" },
      { site: 'amazon', id: '999', type: "onair" }
    ],
    titleTranslate: {
      'zh-Hans': ['药屋少女的呢喃']
    }
  },
  {
    title: 'SPY x FAMILY Season 2',
    begin: '2023-10-07T23:00:00Z', // Saturday
    sites: [
        { site: 'crunchyroll', id: 'spy', type: 'onair' },
        { site: 'netflix', id: 'spy-n', type: 'onair' },
        { site: 'bilibili', id: 'spy-b', type: 'onair' }
    ]
  },
  {
    title: 'Ragna Crimson',
    begin: '2023-09-30T01:00:00Z', // Saturday
    sites: [
        { site: 'hidive', id: 'ragna', type: 'onair' }
    ]
  },
  {
    title: 'Kikansha no Mahou wa Tokubetsu desu',
    begin: '2023-10-07T00:00:00Z', // Saturday
    sites: [
        { site: 'crunchyroll', id: 'magic', type: 'onair' }
    ]
  },

  // Others/Old
  {
    title: '呪術廻戦 懐玉・玉折',
    begin: '2023-07-06T23:56:00Z', // Thursday
    sites: [
      { site: 'bilibili', id: 'jjk', type: "onair" },
      { site: 'netflix', id: 'jjk-n', type: "onair" },
      { site: 'abema', id: 'jjk-a', type: "onair" }
    ],
    titleTranslate: {
        'zh-Hans': ['咒术回战 第二季']
    }
  }
]

export const handlers = [
  http.get('/api/config', () => {
    return HttpResponse.json(mockConfig)
  }),

  http.get('/api/items', () => {
    return HttpResponse.json(mockItems)
  }),

  http.get('/api/anilist', ({ request }) => {
    const url = new URL(request.url)
    const title = url.searchParams.get('title')

    return HttpResponse.json({
      data: {
        Media: {
          title: {
            native: title,
            romaji: 'Frieren: Beyond Journey\'s End',
            english: 'Frieren: Beyond Journey\'s End'
          },
          coverImage: {
            extraLarge: 'https://image.tmdb.org/t/p/original/dhzbCznEzU67RXWYb53fyPe9Keb.jpg',
            large: 'https://image.tmdb.org/t/p/original/dhzbCznEzU67RXWYb53fyPe9Keb.jpg'
          },
          description: 'The adventure is over but life goes on for an elf mage beginning to learn what love means to the people who are long dead.',
          averageScore: 92,
          episodes: 28,
          genres: ['Adventure', 'Drama', 'Fantasy'],
          studios: {
            nodes: [{ name: 'Madhouse' }]
          },
          characters: {
            edges: [
              {
                node: { name: { full: 'Frieren' } },
                voiceActors: [{ name: { full: 'Atsumi Tanezaki' } }]
              }
            ]
          }
        }
      }
    })
  }),

  http.get('/api/metadata', ({ request }) => {
    const url = new URL(request.url)
    const title = url.searchParams.get('title')

    return HttpResponse.json({
      id: '12345',
      title: {
        native: title,
        romaji: 'Frieren: Beyond Journey\'s End',
        english: 'Frieren: Beyond Journey\'s End'
      },
      coverImage: {
        extraLarge: 'https://image.tmdb.org/t/p/original/dhzbCznEzU67RXWYb53fyPe9Keb.jpg',
        large: 'https://image.tmdb.org/t/p/original/dhzbCznEzU67RXWYb53fyPe9Keb.jpg'
      },
      description: 'The adventure is over but life goes on for an elf mage beginning to learn what love means to the people who are long dead.',
      averageScore: 92,
      episodes: 28,
      genres: ['Adventure', 'Drama', 'Fantasy'],
      studios: ['Madhouse'],
      characters: [
          { name: 'Frieren', voiceActor: 'Atsumi Tanezaki', role: 'Main' },
          { name: 'Fern', voiceActor: 'Kana Ichinose', role: 'Main' },
          { name: 'Stark', voiceActor: 'Chiaki Kobayashi', role: 'Main' },
          { name: 'Himmel', voiceActor: 'Nobuhiko Okamoto', role: 'Supporting' },
          { name: 'Heiter', voiceActor: 'Hiroki Touchi', role: 'Supporting' },
          { name: 'Eisen', voiceActor: 'Yoji Ueda', role: 'Supporting' }
      ],
      staff: [
          { name: 'Keiichirou Saitou', role: 'Director', department: 'Directing' },
          { name: 'Tomohiro Suzuki', role: 'Series Composition', department: 'Writing' },
          { name: 'Evan Call', role: 'Music', department: 'Sound' },
          { name: 'Reiko Nagasawa', role: 'Character Design', department: 'Art' }
      ],
      episodesList: Array.from({ length: 24 }, (_, i) => ({
          number: i + 1,
          title: `Episode ${i + 1}: The Journey Begins`,
          airDate: '2023-09-29',
          runtime: 24,
          overview: 'After defeating the Demon King, the hero party returns to the capital essentially disbanding. Frieren, an elf mage, begins a new journey to learn about humans.'
      })),
      isFinished: true,
      totalSeasons: 1,
      currentSeason: 1,
      runtime: 24,
      contentRating: 'TV-14'
    })
  })
]
