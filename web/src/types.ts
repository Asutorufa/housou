export interface Site {
    site: string
    id?: string
    url?: string
    begin?: string
    broadcast?: string
    comment?: string
    regions?: string[]
    type?: string
}

export interface TitleTranslate {
    [key: string]: string[] | undefined
}

export interface AnimeItem {
    title: string
    type: 'tv' | 'movie' | 'ova' | 'ona' | 'special' | string
    lang: string
    officialSite: string
    begin: string
    broadcast?: string
    end: string
    comment?: string
    sites?: Site[]
    titleTranslate?: TitleTranslate
}

export interface SiteMetaItem {
    title: string
    urlTemplate?: string
    type: string
    regions?: string[]
}

export interface SiteMeta {
    [key: string]: SiteMetaItem | undefined
}

export interface Config {
    years: number[]
    site_meta: SiteMeta
    attribution?: {
        tmdb: {
            logo_square: string
            logo_long: string
            logo_alt_long: string
        }
    }
}

export interface UniversalTitle {
    romaji?: string
    english?: string
    native?: string
}

export interface UniversalCoverImage {
    large?: string
    extraLarge?: string
}

export interface UniversalCharacter {
    name: string
    voiceActor?: string
    role?: string
}

export interface UniversalStaff {
    name: string
    role: string
    department?: string
}

export interface UniversalEpisode {
    number: number
    title?: string
    airDate?: string
    overview?: string
    runtime?: number
}

export interface UnifiedMetadata {
    id: string
    title: UniversalTitle
    coverImage: UniversalCoverImage
    averageScore?: number
    episodes?: number
    genres: string[]
    description?: string
    studios: string[]
    characters: UniversalCharacter[]
    staff: UniversalStaff[]
    episodesList: UniversalEpisode[]
    isFinished: boolean
    totalSeasons?: number
    currentSeason?: number
    runtime?: number
    contentRating?: string
}
