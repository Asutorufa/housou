import { http, HttpResponse } from 'msw'

const mockConfig = {
  years: [2024, 2025],
  site_meta: {
    bilibili: { title: 'Bilibili', urlTemplate: 'https://www.bilibili.com/bangumi/play/ss{{id}}' },
    netflix: { title: 'Netflix', urlTemplate: 'https://www.netflix.com/title/{{id}}' },
    amazon: { title: 'Prime Video', urlTemplate: 'https://www.amazon.co.jp/dp/{{id}}' },
    abema: { title: 'Abema', urlTemplate: 'https://abema.tv/video/title/{{id}}' },
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
  {
    title: '葬送のフリーレン',
    begin: '2023-09-29T12:00:00Z',
    sites: [
      { site: 'bilibili', id: '123' },
      { site: 'netflix', id: '456' }
    ],
    titleTranslate: {
      'zh-Hans': ['葬送的芙莉莲']
    }
  },
  {
    title: '薬屋のひとりごと',
    begin: '2023-10-21T16:00:00Z',
    sites: [
      { site: 'netflix', id: '789' }
    ],
    titleTranslate: {
      'zh-Hans': ['药屋少女的呢喃']
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
            extraLarge: 'https://s4.anilist.co/file/anilistcdn/media/anime/cover/large/bx154587-nBySp9Yp93Xz.jpg',
            large: 'https://s4.anilist.co/file/anilistcdn/media/anime/cover/medium/bx154587-nBySp9Yp93Xz.jpg'
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
  })
]
