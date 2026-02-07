# Housou (放送)

Anime broadcast schedule viewer based on [bangumi-data](https://github.com/bangumi-data/bangumi-data).

## Features

- 📅 Weekly schedule view with day-of-week tabs
- 🔍 Filter by year, season, and streaming site
- 🌙 Automatic dark mode
- ⚡ Cloudflare Workers edge deployment
- 🗄️ Auto-caching (1 day TTL)

## Deploy

[![Deploy to Cloudflare Workers](https://deploy.workers.cloudflare.com/button)](https://deploy.workers.cloudflare.com/?url=https://github.com/YOUR_USERNAME/housou)

## Local Development

```bash
# Install dependencies
cargo install worker-build

# Start dev server
npx wrangler dev
```

## Manual Deploy

```bash
npx wrangler deploy
```

## Project Structure

```
├── public/           # Static assets (assets binding)
│   └── index.html   # Frontend
├── src/
│   ├── lib.rs       # Worker entry + API routes
│   └── model.rs     # Data models
├── wrangler.toml    # Cloudflare config
└── Cargo.toml       # Rust dependencies
```

## API

- `GET /api/config` - Get config (site metadata, available years)
- `GET /api/items?year=2025&season=Winter` - Get anime list

## License

MIT
