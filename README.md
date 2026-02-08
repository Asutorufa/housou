# Housou (放送)

Anime broadcast schedule viewer based on [bangumi-data](https://github.com/bangumi-data/bangumi-data).

## Features

- 📅 Weekly schedule view with day-of-week tabs
- 🔍 Filter by year, season, and streaming site
- � Details modal with cast, staff, episodes, streaming links
- 📊 Metadata from TMDb and AniList (auto-selected)
- �🌙 Automatic dark mode
- ⚡ Cloudflare Workers edge deployment
- 🗄️ Dynamic caching (7 days for airing, 30 days for finished)

## Screenshots

| Home Page | Details Modal |
| :---: | :---: |
| ![Home](docs/images/home.png) | ![Details](docs/images/details.png) |

## Deploy

[![Deploy to Cloudflare Workers](https://deploy.workers.cloudflare.com/button)](https://deploy.workers.cloudflare.com/?url=https://github.com/Asutorufa/housou)

## Environment Variables

Create `.dev.vars` for local development:

```
TMDB_TOKEN=your_tmdb_api_token
```

For production, set the secret via:

```bash
npx wrangler secret put TMDB_TOKEN
```

## Local Development

```bash
# Install web dependencies
cd web && npm install && cd ..

# Start dev server
npx wrangler dev
```

## Manual Deploy

```bash
cd web && npm run build && cd ..
npx wrangler deploy
```

## Project Structure

```
├── src/
│   ├── lib.rs           # Worker entry + API routes
│   ├── model.rs         # Data models
│   ├── provider.rs      # Provider router
│   └── provider/
│       ├── tmdb.rs      # TMDb metadata provider
│       └── anilist.rs   # AniList metadata provider
├── web/
│   ├── src/
│   │   ├── App.tsx              # Main app
│   │   └── components/
│   │       ├── AnimeCard.tsx    # Anime card with poster
│   │       ├── DetailsModal.tsx # Details popup
│   │       ├── Header.tsx       # Header with filters
│   │       └── TabbedGrid.tsx   # Day-of-week tabs + grid
│   └── public/
│       └── favicon.svg          # Rainbow broadcast icon
├── wrangler.toml        # Cloudflare config
└── Cargo.toml           # Rust dependencies
```

## API

- `GET /api/items` - Get anime list from bangumi-data
- `GET /api/metadata?id=xxx&title=xxx&year=2025` - Get metadata (auto-selects TMDb or AniList)

## License

MIT
