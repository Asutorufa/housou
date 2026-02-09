
// Base URL for Bangumi Data
pub const BASE_DATA_URL: &str =
    "https://raw.githubusercontent.com/bangumi-data/bangumi-data/master/data/";

// Cache TTLs
const ONE_MINUTE: i32 = 60;
const ONE_HOUR: i32 = 60 * ONE_MINUTE;
const ONE_DAY: i32 = 24 * ONE_HOUR;

pub const CACHE_TTL_SECONDS: i32 = 6 * ONE_HOUR; // 6 hours cache
pub const CACHE_TTL_404: i32 = ONE_HOUR; // 1 hour for 404s
pub const CACHE_TTL_CONFIG: i32 = ONE_MINUTE; // 1 minute for config
pub const CACHE_TTL_API: i32 = ONE_DAY; // 24 hours for API responses
pub const CACHE_TTL_FINISHED: i32 = 30 * ONE_DAY; // 30 days for finished titles
pub const CACHE_TTL_ONGOING: i32 = 7 * ONE_DAY; // 1 week for ongoing titles

// Cache Version
pub const CACHE_VERSION: &str = "v3";

// Configuration
pub const START_YEAR: i32 = 1943;

// TMDB Attribution URLs
pub const TMDB_LOGO_SQUARE: &str = "https://www.themoviedb.org/assets/2/v4/logos/v2/blue_square_2-d537fb228cf3ed904132c3096b9736928c38cfe75196763ebd7e9f22e855d9e5.svg";
pub const TMDB_LOGO_LONG: &str = "https://www.themoviedb.org/assets/2/v4/logos/v2/blue_short-8e7b30f73a4020692ccca9c88bafe5dcb6f8a62a4c6bc55cd9ba82bb2cd95f6c.svg";
pub const TMDB_LOGO_ALT_LONG: &str = "https://www.themoviedb.org/assets/2/v4/logos/v2/blue_long_2-9665a76b1ae401a510ec1e0ca40ddcb3b0cfe45f1d51b77a308fea0845885648.svg";
