export interface Site {
  site: string;
  id?: string;
  url?: string;
  begin?: string;
  broadcast?: string;
  comment?: string;
  regions?: string[];
}

export interface TitleTranslate {
  [key: string]: string[] | undefined;
}

export interface AnimeItem {
  title: string;
  type: "tv" | "movie" | "ova" | "ona" | "special" | string;
  lang: string;
  officialSite: string;
  begin: string;
  broadcast?: string;
  end: string;
  comment?: string;
  sites?: Site[];
  titleTranslate?: TitleTranslate;
}

export type UserStatus = 0 | 1 | 2 | 3 | 4 | 5;

export const USER_STATUS_LABELS: Record<UserStatus, string> = {
  0: "未登録",
  1: "見てる",
  2: "見終わった",
  3: "保留",
  4: "切った",
  5: "見たい",
};

export interface UserItemSummary {
  status: UserStatus;
  score: number | null;
}

export interface DisplayAnimeItem extends AnimeItem {
  userStatus?: UserStatus;
  userScore?: number;
}

export interface SiteMetaItem {
  title: string;
  urlTemplate?: string;
  type: string;
  regions?: string[];
}

export interface SiteMeta {
  [key: string]: SiteMetaItem | undefined;
}

export interface Config {
  years: number[];
  site_meta: SiteMeta;
  attribution?: {
    tmdb: {
      logo_square: string;
      logo_long: string;
      logo_alt_long: string;
    };
  };
  auth_enabled?: boolean;
  github_enabled?: boolean;
  telegram_bot_name?: string;
}

export interface User {
  id: number;
  email: string;
  username: string;
  avatar_url?: string;
  github_id?: string;
  telegram_id?: string;
  has_password?: boolean;
  created_at: number;
}

// Define strict types for auth payloads
export interface LoginData {
  email: string;
  password: string;
}

export interface RegisterData {
  email: string;
  username: string;
  password: string;
}

export interface TelegramAuthData {
  id: number;
  first_name: string;
  last_name?: string;
  username?: string;
  photo_url?: string;
  auth_date: number;
  hash: string;
}

export interface UniversalTitle {
  romaji?: string;
  english?: string;
  native?: string;
}

export interface UniversalCoverImage {
  large?: string;
  extraLarge?: string;
}

export interface UniversalCharacter {
  name: string;
  voiceActor?: string;
  role?: string;
}

export interface UniversalStaff {
  name: string;
  role: string;
  department?: string;
}

export interface UniversalEpisode {
  number: number;
  title?: string;
  airDate?: string;
  overview?: string;
  runtime?: number;
}

export interface UniversalVideo {
  key?: string;
  site?: string;
  name?: string;
  type?: string;
  size?: number;
}

export interface UnifiedMetadata {
  id: string;
  sourceSite?: string;
  title: UniversalTitle;
  coverImage: UniversalCoverImage;
  averageScore?: number;
  episodes?: number;
  genres: string[];
  description?: string;
  studios: string[];
  characters: UniversalCharacter[];
  staff: UniversalStaff[];
  episodesList: UniversalEpisode[];
  isFinished: boolean;
  totalSeasons?: number;
  currentSeason?: number;
  runtime?: number;
  contentRating?: string;
  videos?: UniversalVideo[];
}

export interface Selections {
  year: string;
  season: string;
  site: string;
  status?: string;
}

export interface MetadataRequest {
  tmdb_id?: string;
  mal_id?: string;
  anilist_id?: string;
  title?: string;
  year?: number;
}
