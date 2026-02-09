use super::MetadataProvider;
use crate::model;
use regex::Regex;
use std::sync::OnceLock;
use tmdb_client::async_apis::AsyncAPIClient;
use tmdb_client::models;
use worker::*;

pub struct TmdbProvider<'a> {
    env: &'a Env,
}

struct SyncApiClient(AsyncAPIClient);

// SAFETY: AsyncAPIClient uses trait objects that are not Send/Sync, preventing it from being Sync.
// However, Cloudflare Workers (wasm32-unknown-unknown) is a single-threaded environment.
// We are using this wrapper to satisfy the static OnceLock requirement while guaranteeing
// no actual concurrent access will occur in this environment.
unsafe impl Send for SyncApiClient {}
unsafe impl Sync for SyncApiClient {}

static TMDB_CLIENT: OnceLock<Option<SyncApiClient>> = OnceLock::new();

impl<'a> TmdbProvider<'a> {
    pub fn new(env: &'a Env) -> Self {
        Self { env }
    }

    fn get_client(&self) -> Result<&AsyncAPIClient> {
        let client_opt = TMDB_CLIENT.get_or_init(|| {
            let api_token = self
                .env
                .secret("TMDB_TOKEN")
                .map(|s| s.to_string())
                .or_else(|_| self.env.var("TMDB_TOKEN").map(|s| s.to_string()))
                .ok();

            api_token.map(|t| SyncApiClient(AsyncAPIClient::new_with_api_key(t)))
        });

        client_opt
            .as_ref()
            .map(|w| &w.0)
            .ok_or_else(|| Error::RustError("TMDB_TOKEN not set".into()))
    }
}

impl<'a> MetadataProvider for TmdbProvider<'a> {
    async fn fetch(
        &self,
        id: Option<&str>,
        title: Option<&str>,
        year: Option<i32>,
    ) -> Result<model::UnifiedMetadata> {
        let client = self.get_client()?;

        // 1. Resolve ID (Search if needed)
        let (media_id, media_type) = if let Some(id) = id {
            parse_tmdb_id(id)?
        } else if let Some(search_title) = title {
            search_media(client, search_title, year).await?
        } else {
            return Err(Error::RustError("ID or Title required".into()));
        };

        // 2. Fetch Details based on type
        match media_type {
            MediaType::Movie => get_movie_details(client, media_id).await,
            MediaType::Tv { show_id, season } => get_tv_details(client, show_id, season).await,
        }
    }
}

#[derive(Debug, PartialEq)]
enum MediaType {
    Movie,
    Tv { show_id: String, season: i32 },
}

fn parse_tmdb_id(id: &str) -> Result<(String, MediaType)> {
    let id = id.trim_start_matches('/');
    // Strip episode part if present
    let id = id.split("/episode/").next().unwrap_or(id);
    let parts: Vec<&str> = id.split('/').collect();

    if parts.is_empty() || (parts.len() == 1 && parts[0].is_empty()) {
        return Err(Error::RustError("Empty ID".into()));
    }

    if parts.first() == Some(&"tv") {
        if parts.len() < 2 {
            return Err(Error::RustError("Invalid TV ID format: missing ID".into()));
        }
        let show_id = parts[1].to_string();
        let season = if parts.len() >= 4 && parts[2] == "season" {
            parts[3]
                .parse()
                .map_err(|_| Error::RustError("Invalid season number".into()))?
        } else {
            1
        };
        Ok((show_id.clone(), MediaType::Tv { show_id, season }))
    } else if parts.first() == Some(&"movie") {
        if parts.len() < 2 {
            return Err(Error::RustError(
                "Invalid Movie ID format: missing ID".into(),
            ));
        }
        let movie_id = parts[1].to_string();
        Ok((movie_id, MediaType::Movie))
    } else if parts.len() == 1 {
        // Assume movie if ID is just a number/slug
        let movie_id = parts[0].to_string();
        Ok((movie_id, MediaType::Movie))
    } else {
        Err(Error::RustError("Unknown media type or format".into()))
    }
}

async fn search_media(
    client: &AsyncAPIClient,
    title: &str,
    year: Option<i32>,
) -> Result<(String, MediaType)> {
    // Try normalized title search
    let normalized = normalize_title(title);

    let results = client
        .search_api()
        .get_search_multi_paginated(&normalized, Some("ja-JP"), Some(1), Some(false), None)
        .await
        .map_err(|e| Error::RustError(format!("TMDb search failed: {}", e)))?;

    // Filter and find best match
    if let Some(results_vec) = results.results {
        for res in results_vec {
            let media_type = res.get("media_type").and_then(|v| v.as_str());

            match media_type {
                Some("movie") => {
                    let id = res.get("id").and_then(|v| v.as_i64());
                    let release_date = res.get("release_date").and_then(|v| v.as_str());

                    if let Some(id) = id {
                        let id_str = id.to_string();
                        // Check year if provided
                        if let Some(y) = year {
                            if let Some(date_str) = release_date
                                && date_str.starts_with(&y.to_string())
                            {
                                return Ok((id_str, MediaType::Movie));
                            }
                        } else {
                            return Ok((id_str, MediaType::Movie));
                        }
                    }
                }
                Some("tv") => {
                    let id = res.get("id").and_then(|v| v.as_i64());
                    let first_air_date = res.get("first_air_date").and_then(|v| v.as_str());

                    if let Some(id) = id {
                        let id_str = id.to_string();
                        // Check year if provided
                        if let Some(y) = year {
                            if let Some(date_str) = first_air_date
                                && date_str.starts_with(&y.to_string())
                            {
                                return Ok((
                                    id_str.clone(),
                                    MediaType::Tv {
                                        show_id: id_str,
                                        season: 1,
                                    },
                                ));
                            }
                        } else {
                            return Ok((
                                id_str.clone(),
                                MediaType::Tv {
                                    show_id: id_str,
                                    season: 1,
                                },
                            ));
                        }
                    }
                }
                _ => continue,
            }
        }
    }

    Err(Error::RustError("No suitable match found".into()))
}

static TITLE_NORMALIZE_REGEX: OnceLock<Regex> = OnceLock::new();

fn normalize_title(title: &str) -> String {
    let normalized = title.replace("-", " - ");

    let re = TITLE_NORMALIZE_REGEX.get_or_init(|| {
        Regex::new(r"(?i)(\s*第\d+期|\s*第\d+クール|\s*Season\s*\d+|\s*\d+(st|nd|rd|th)\s*Season|\s*[ⅡⅢⅣⅤⅥⅦⅧⅨⅩ]+\s*|\s*\(\d{4}\)\s*)+$")
            .expect("Invalid Title Normalize Regex")
    });

    let stripped = re.replace(&normalized, "");

    let mut parts = stripped.split_whitespace();
    let mut result = String::with_capacity(stripped.len());

    if let Some(first) = parts.next() {
        result.push_str(first);
        for part in parts {
            result.push(' ');
            result.push_str(part);
        }
    }

    while result.ends_with(|c: char| c.is_whitespace() || c == '-') {
        result.pop();
    }
    result
}

async fn get_movie_details(
    client: &AsyncAPIClient,
    movie_id: String,
) -> Result<model::UnifiedMetadata> {
    // Extract ID if it contains a slug (fallback to parsing the whole string if no slug)
    let id: i32 = movie_id
        .split('-')
        .next()
        .unwrap_or(&movie_id)
        .parse()
        .map_err(|_| Error::RustError("Invalid movie ID format".into()))?;

    let movie = client
        .movies_api()
        .get_movie_details(id, Some("ja-JP"), None, Some("release_dates,credits"))
        .await
        .map_err(|e| Error::RustError(format!("Failed to fetch movie details: {}", e)))?;

    Ok(movie_to_unified(movie))
}

async fn get_tv_details(
    client: &AsyncAPIClient,
    show_id: String,
    season_number: i32,
) -> Result<model::UnifiedMetadata> {
    // Extract ID if it contains a slug
    let id: i32 = show_id
        .split('-')
        .next()
        .unwrap_or(&show_id)
        .parse()
        .map_err(|_| Error::RustError("Invalid show ID format".into()))?;

    let show = client
        .tv_api()
        .get_tv_details(id, Some("ja-JP"), None, Some("content_ratings,credits"))
        .await
        .map_err(|e| Error::RustError(format!("Failed to fetch TV details: {}", e)))?;

    let season = client
        .tv_seasons_api()
        .get_tv_season_details(id, season_number, Some("ja-JP"), None, Some("credits"))
        .await
        .map_err(|e| Error::RustError(format!("Failed to fetch Season details: {}", e)))?;

    Ok(tv_to_unified(show, season))
}

fn movie_to_unified(movie: models::MovieDetails) -> model::UnifiedMetadata {
    use model::*;

    let title = UniversalTitle {
        romaji: None,
        english: None,
        native: movie.title.clone(),
    };

    let cover_image = UniversalCoverImage {
        large: movie
            .poster_path
            .as_ref()
            .map(|p| format!("https://image.tmdb.org/t/p/w500{}", p)),
        extra_large: movie
            .poster_path
            .as_ref()
            .map(|p| format!("https://image.tmdb.org/t/p/original{}", p)),
    };

    let genres = movie
        .genres
        .unwrap_or_default()
        .into_iter()
        .filter_map(|g| g.name)
        .collect();
    let studios = movie
        .production_companies
        .unwrap_or_default()
        .into_iter()
        .filter_map(|s| s.name)
        .collect();

    let mut characters = Vec::new();
    let mut staff = Vec::new();

    if let Some(credits) = movie.credits {
        if let Some(cast) = credits.cast {
            for member in cast.into_iter().take(6) {
                characters.push(UniversalCharacter {
                    name: member.character.unwrap_or_default(),
                    voice_actor: member.name,
                    role: Some("Cast".to_string()),
                });
            }
        }
        if let Some(crew) = credits.crew {
            for member in crew.into_iter().take(10) {
                staff.push(UniversalStaff {
                    name: member.name.unwrap_or_default(),
                    role: member.job.unwrap_or_default(),
                    department: member.department,
                });
            }
        }
    }

    // Content Ratings (Release Dates for Movies)
    let content_rating = movie.release_dates.and_then(|dates| {
        dates.results.and_then(|results| {
            find_best_rating(
                &results,
                |r| r.iso_3166_1.as_deref(),
                |r| {
                    r.release_dates
                        .as_ref()
                        .and_then(|d| {
                            d.iter()
                                .find(|x| x.certification.as_deref().is_some_and(|c| !c.is_empty()))
                        })
                        .and_then(|x| x.certification.clone())
                },
            )
        })
    });

    UnifiedMetadata {
        id: format!("movie/{}", movie.id.unwrap_or(0)),
        title,
        cover_image,
        average_score: movie.vote_average.map(|v| (v * 10.0) as i32),
        episodes: None,
        genres,
        description: movie.overview,
        studios,
        characters,
        staff,
        episodes_list: Vec::new(),
        is_finished: movie.status.as_deref() == Some("Released"),
        total_seasons: None,
        current_season: None,
        runtime: movie.runtime,
        content_rating: content_rating.or_else(|| {
            if movie.adult == Some(true) {
                Some("R18".to_string())
            } else {
                None
            }
        }),
    }
}

fn tv_to_unified(show: models::TvDetails, season: models::SeasonDetails) -> model::UnifiedMetadata {
    use model::*;

    let title = UniversalTitle {
        romaji: None,
        english: None,
        native: Some(format!(
            "{} : {}",
            show.name.clone().unwrap_or_default(),
            season.name.clone().unwrap_or_default()
        )),
    };

    let poster_path = season.poster_path.or(show.poster_path.clone());
    let cover_image = UniversalCoverImage {
        large: poster_path
            .as_ref()
            .map(|p| format!("https://image.tmdb.org/t/p/w500{}", p)),
        extra_large: poster_path
            .as_ref()
            .map(|p| format!("https://image.tmdb.org/t/p/original{}", p)),
    };

    let genres = show
        .genres
        .unwrap_or_default()
        .into_iter()
        .filter_map(|g| g.name)
        .collect();
    let studios = show
        .production_companies
        .unwrap_or_default()
        .into_iter()
        .filter_map(|s| s.name)
        .collect();

    let mut characters = Vec::new();
    let mut staff = Vec::new();

    // Prefer season credits, fall back to show credits
    let credits = season.credits.or(show.credits);

    if let Some(credits) = credits {
        if let Some(cast) = credits.cast {
            for member in cast.into_iter().take(6) {
                characters.push(UniversalCharacter {
                    name: member.character.unwrap_or_default(),
                    voice_actor: member.name,
                    role: Some("Cast".to_string()),
                });
            }
        }
        if let Some(crew) = credits.crew {
            for member in crew.into_iter().take(10) {
                staff.push(UniversalStaff {
                    name: member.name.unwrap_or_default(),
                    role: member.job.unwrap_or_default(),
                    department: member.department,
                });
            }
        }
    }

    let episodes_list: Vec<_> = season
        .episodes
        .unwrap_or_default()
        .into_iter()
        .map(|e| UniversalEpisode {
            number: e.episode_number.unwrap_or(0),
            title: e.name,
            air_date: e.air_date,
            overview: e.overview,
            runtime: None,
        })
        .collect();

    // Content Ratings
    let content_rating = show.content_ratings.and_then(|ratings| {
        ratings.results.and_then(|results| {
            find_best_rating(
                &results,
                |r| r.iso_3166_1.as_deref(),
                |r| r.rating.as_ref().filter(|s| !s.is_empty()).cloned(),
            )
        })
    });

    let is_finished =
        show.status.as_deref() == Some("Ended") || show.status.as_deref() == Some("Canceled");

    // Runtime logic
    let season_episodes_len = episodes_list.len();
    let first_ep_runtime = episodes_list.first().and_then(|e| e.runtime);

    // show.episode_run_time is Option<Vec<i32>>
    let show_runtime = show
        .episode_run_time
        .as_ref()
        .and_then(|v| v.first().copied());

    let final_runtime = first_ep_runtime.or(show_runtime);

    let show_id_val = show.id.unwrap_or(0);
    let season_num_val = season.season_number.unwrap_or(1);

    UnifiedMetadata {
        id: format!("tv/{}/season/{}", show_id_val, season_num_val),
        title,
        cover_image,
        average_score: show.vote_average.map(|v| (v * 10.0) as i32),
        episodes: Some(season_episodes_len as i32),
        genres,
        description: season.overview.filter(|s| !s.is_empty()).or(show.overview),
        studios,
        characters,
        staff,
        episodes_list,
        is_finished,
        total_seasons: show.number_of_seasons,
        current_season: Some(season_num_val),
        runtime: final_runtime,
        content_rating: content_rating.or_else(|| {
            if show.adult == Some(true) {
                Some("R18".to_string())
            } else {
                None
            }
        }),
    }
}

fn find_best_rating<T, FCountry, FRating>(
    results: &[T],
    get_country: FCountry,
    get_rating: FRating,
) -> Option<String>
where
    FCountry: Fn(&T) -> Option<&str>,
    FRating: Fn(&T) -> Option<String>,
{
    let find_by_country = |country: &str| {
        results
            .iter()
            .find(|r| get_country(r) == Some(country))
            .and_then(&get_rating)
    };

    find_by_country("JP")
        .or_else(|| find_by_country("US"))
        .or_else(|| results.iter().filter_map(&get_rating).next())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tmdb_id() {
        // TV Show Cases
        assert_eq!(
            parse_tmdb_id("tv/123").unwrap(),
            (
                "123".to_string(),
                MediaType::Tv {
                    show_id: "123".to_string(),
                    season: 1
                }
            )
        );
        assert_eq!(
            parse_tmdb_id("tv/123/season/2").unwrap(),
            (
                "123".to_string(),
                MediaType::Tv {
                    show_id: "123".to_string(),
                    season: 2
                }
            )
        );
        assert_eq!(
            parse_tmdb_id("/tv/123/season/2").unwrap(),
            (
                "123".to_string(),
                MediaType::Tv {
                    show_id: "123".to_string(),
                    season: 2
                }
            )
        );
        assert_eq!(
            parse_tmdb_id("tv/123/season/2/episode/5").unwrap(),
            (
                "123".to_string(),
                MediaType::Tv {
                    show_id: "123".to_string(),
                    season: 2
                }
            )
        );

        // Movie Cases
        assert_eq!(
            parse_tmdb_id("movie/456").unwrap(),
            ("456".to_string(), MediaType::Movie)
        );
        assert_eq!(
            parse_tmdb_id("456").unwrap(),
            ("456".to_string(), MediaType::Movie)
        );
        assert_eq!(
            parse_tmdb_id("/movie/456").unwrap(),
            ("456".to_string(), MediaType::Movie)
        );

        // Slugged ID Cases
        assert_eq!(
            parse_tmdb_id("tv/123-show-name").unwrap(),
            (
                "123-show-name".to_string(),
                MediaType::Tv {
                    show_id: "123-show-name".to_string(),
                    season: 1
                }
            )
        );
        assert_eq!(
            parse_tmdb_id("tv/123-show-name/season/2").unwrap(),
            (
                "123-show-name".to_string(),
                MediaType::Tv {
                    show_id: "123-show-name".to_string(),
                    season: 2
                }
            )
        );
        assert_eq!(
            parse_tmdb_id("movie/456-movie-name").unwrap(),
            ("456-movie-name".to_string(), MediaType::Movie)
        );
        assert_eq!(
            parse_tmdb_id("456-movie-name").unwrap(),
            ("456-movie-name".to_string(), MediaType::Movie)
        );

        // Edge Cases
        assert!(parse_tmdb_id("tv").is_err());
        // tv/abc is now valid because we don't extract int. "abc" is a valid ID string.
        // But "tv/123/season/abc" should still fail because season must be int.
        assert_eq!(
            parse_tmdb_id("tv/abc").unwrap(),
            (
                "abc".to_string(),
                MediaType::Tv {
                    show_id: "abc".to_string(),
                    season: 1
                }
            )
        );
        assert!(parse_tmdb_id("tv/123/season/abc").is_err());
        assert!(parse_tmdb_id("").is_err());
        // foo/bar returns Unknown media type
        assert!(parse_tmdb_id("foo/bar").is_err());
    }

    #[test]
    fn test_normalize_title() {
        let cases = vec![
            // Basic cases
            ("One Piece", "One Piece"),
            ("Title-Something", "Title - Something"),
            // Japanese season formats
            ("Title 第1期", "Title"),
            ("Title 第2クール", "Title"),
            // English season formats
            ("Title Season 1", "Title"),
            ("Title 2nd Season", "Title"),
            ("Title Season 1 (2023)", "Title"),
            ("Title - Season 1", "Title"),
            // Roman numerals
            ("Title Ⅱ", "Title"),
            ("Sword Art Online Ⅱ", "Sword Art Online"),
            // Year suffix
            ("Title (2023)", "Title"),
            ("Title (2023) Season 1", "Title"),
            ("Steins;Gate (2011)", "Steins;Gate"),
            // Spacing normalization
            ("Title  Season  1", "Title"),
            ("  Test  Title  ", "Test Title"),
            ("Title   With    Many   Spaces", "Title With Many Spaces"),
            // Real-world titles
            ("Attack on Titan Season 3", "Attack on Titan"),
            ("My Hero Academia 第2期", "My Hero Academia"),
            (
                "Fullmetal Alchemist: Brotherhood",
                "Fullmetal Alchemist: Brotherhood",
            ),
            ("One Piece - 1000", "One Piece - 1000"),
            (
                "Demon Slayer: Kimetsu no Yaiba Season 2",
                "Demon Slayer: Kimetsu no Yaiba",
            ),
        ];

        for (input, expected) in cases {
            assert_eq!(
                normalize_title(input),
                expected,
                "Failed for input: {}",
                input
            );
        }
    }

    #[test]
    fn test_movie_to_unified() {
        // We need to import the models from the tmdb_client crate to construct the input
        use tmdb_client::models::{
            Cast, Credits, Crew, Genre, MovieDetails, ReleaseDate, ReleaseDatesList,
            ReleasedateslistResults,
        };

        let movie = MovieDetails {
            id: Some(12345),
            title: Some("Test Movie".to_string()),
            poster_path: Some("/path/to/poster.jpg".to_string()),
            genres: Some(vec![
                Genre {
                    id: Some(1),
                    name: Some("Action".to_string()),
                    ..Default::default()
                },
                Genre {
                    id: Some(2),
                    name: Some("Adventure".to_string()),
                    ..Default::default()
                },
            ]),
            overview: Some("This is a test movie description.".to_string()),
            release_dates: Some(ReleaseDatesList {
                results: Some(vec![
                    ReleasedateslistResults {
                        iso_3166_1: Some("US".to_string()),
                        release_dates: Some(vec![ReleaseDate {
                            certification: Some("PG-13".to_string()),
                            ..Default::default()
                        }]),
                        ..Default::default()
                    },
                    ReleasedateslistResults {
                        iso_3166_1: Some("JP".to_string()),
                        release_dates: Some(vec![ReleaseDate {
                            certification: Some("G".to_string()),
                            ..Default::default()
                        }]),
                        ..Default::default()
                    },
                ]),
                ..Default::default()
            }),
            credits: Some(Credits {
                cast: Some(vec![Cast {
                    name: Some("Actor 1".to_string()),
                    character: Some("Character 1".to_string()),
                    ..Default::default()
                }]),
                crew: Some(vec![Crew {
                    name: Some("Director 1".to_string()),
                    job: Some("Director".to_string()),
                    department: Some("Directing".to_string()),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            production_companies: None,
            vote_average: Some(8.5),
            status: Some("Released".to_string()),
            runtime: Some(120),
            adult: Some(false),
            ..Default::default()
        };

        let unified = movie_to_unified(movie);

        let expected = model::UnifiedMetadata {
            id: "movie/12345".into(),
            title: model::UniversalTitle {
                native: Some("Test Movie".into()),
                ..Default::default()
            },
            cover_image: model::UniversalCoverImage {
                large: Some("https://image.tmdb.org/t/p/w500/path/to/poster.jpg".into()),
                extra_large: Some("https://image.tmdb.org/t/p/original/path/to/poster.jpg".into()),
            },
            average_score: Some(85),
            genres: vec!["Action".into(), "Adventure".into()],
            description: Some("This is a test movie description.".into()),
            studios: vec![],
            characters: vec![model::UniversalCharacter {
                name: "Character 1".into(),
                voice_actor: Some("Actor 1".into()),
                role: Some("Cast".into()),
            }],
            staff: vec![model::UniversalStaff {
                name: "Director 1".into(),
                role: "Director".into(),
                department: Some("Directing".into()),
            }],
            is_finished: true,
            runtime: Some(120),
            // Content rating should prioritize JP, then US, then first available
            content_rating: Some("G".into()),
            ..Default::default()
        };

        assert_eq!(unified, expected);
    }

    #[test]
    fn test_find_best_rating() {
        struct MockRating {
            country: Option<String>,
            rating: Option<String>,
        }

        let cases = vec![
            (
                vec![
                    MockRating {
                        country: Some("US".into()),
                        rating: Some("PG".into()),
                    },
                    MockRating {
                        country: Some("JP".into()),
                        rating: Some("G".into()),
                    },
                ],
                Some("G".to_string()),
            ),
            (
                vec![
                    MockRating {
                        country: Some("UK".into()),
                        rating: Some("12".into()),
                    },
                    MockRating {
                        country: Some("US".into()),
                        rating: Some("PG-13".into()),
                    },
                ],
                Some("PG-13".to_string()),
            ),
            (
                vec![
                    MockRating {
                        country: Some("UK".into()),
                        rating: Some("15".into()),
                    },
                    MockRating {
                        country: Some("FR".into()),
                        rating: Some("12".into()),
                    },
                ],
                Some("15".to_string()),
            ),
            (vec![], None),
            (
                vec![
                    MockRating {
                        country: Some("JP".into()),
                        rating: None,
                    },
                    MockRating {
                        country: Some("US".into()),
                        rating: Some("R".into()),
                    },
                ],
                Some("R".to_string()),
            ),
            (
                vec![
                    MockRating {
                        country: Some("JP".into()),
                        rating: None,
                    },
                    MockRating {
                        country: Some("US".into()),
                        rating: None,
                    },
                    MockRating {
                        country: Some("UK".into()),
                        rating: Some("18".into()),
                    },
                ],
                Some("18".to_string()),
            ),
        ];

        for (input, expected) in cases {
            let result = find_best_rating(&input, |r| r.country.as_deref(), |r| r.rating.clone());
            assert_eq!(result, expected);
        }
    }
}

#[cfg(test)]
mod tests_tv_transformation {
    use super::*;
    use tmdb_client::models;

    fn create_credits(cast_name: Option<&str>, crew_name: Option<&str>) -> models::Credits {
        models::Credits {
            cast: cast_name.map(|n| {
                vec![models::Cast {
                    name: Some(n.to_string()),
                    character: Some("Character".to_string()),
                    ..Default::default()
                }]
            }),
            crew: crew_name.map(|n| {
                vec![models::Crew {
                    name: Some(n.to_string()),
                    job: Some("Director".to_string()),
                    department: Some("Directing".to_string()),
                    ..Default::default()
                }]
            }),
            guest_stars: None,
            id: None,
        }
    }

    #[test]
    fn test_tv_to_unified_full_data() {
        let show = models::TvDetails {
            name: Some("Show Title".to_string()),
            poster_path: Some("/show_poster.jpg".to_string()),
            genres: Some(vec![models::Genre {
                id: Some(1),
                name: Some("Action".to_string()),
            }]),
            production_companies: Some(vec![models::CompanyObject {
                id: Some(1),
                name: Some("Studio A".to_string()),
                logo_path: None,
            }]),
            credits: Some(create_credits(Some("Show Actor"), Some("Show Director"))),
            content_ratings: None,
            status: Some("Ended".to_string()),
            episode_run_time: Some(vec![24]),
            id: Some(100),
            overview: Some("Show Overview".to_string()),
            number_of_seasons: Some(2),
            adult: Some(false),
            ..Default::default()
        };

        let season = models::SeasonDetails {
            name: Some("Season 1".to_string()),
            poster_path: Some("/season_poster.jpg".to_string()),
            episodes: Some(vec![models::EpisodeDetails {
                episode_number: Some(1),
                name: Some("Ep 1".to_string()),
                air_date: Some("2023-01-01".to_string()),
                overview: Some("Ep Overview".to_string()),
                ..Default::default()
            }]),
            season_number: Some(1),
            overview: Some("Season Overview".to_string()),
            credits: Some(create_credits(
                Some("Season Actor"),
                Some("Season Director"),
            )),
            ..Default::default()
        };

        let result = tv_to_unified(show, season);

        assert_eq!(result.id, "tv/100/season/1");
        assert_eq!(
            result.title.native,
            Some("Show Title : Season 1".to_string())
        );
        // Season poster should take precedence
        assert!(
            result
                .cover_image
                .large
                .unwrap()
                .contains("/season_poster.jpg")
        );
        assert_eq!(result.genres, vec!["Action".to_string()]);
        assert_eq!(result.studios, vec!["Studio A".to_string()]);
        // Season credits should be used
        assert_eq!(
            result.characters[0].voice_actor,
            Some("Season Actor".to_string())
        );
        assert_eq!(result.staff[0].name, "Season Director".to_string());
        assert_eq!(result.description, Some("Season Overview".to_string()));
        assert_eq!(result.runtime, Some(24)); // Show runtime
        assert!(result.is_finished);
    }

    #[test]
    fn test_tv_to_unified_credits_fallback() {
        // Case 1: Season has credits -> expect Season credits
        let show_with_credits = models::TvDetails {
            credits: Some(create_credits(Some("Show Actor"), None)),
            ..Default::default()
        };
        let season_with_credits = models::SeasonDetails {
            credits: Some(create_credits(Some("Season Actor"), None)),
            ..Default::default()
        };
        let result1 = tv_to_unified(show_with_credits.clone(), season_with_credits);
        assert_eq!(
            result1.characters[0].voice_actor,
            Some("Season Actor".to_string())
        );

        // Case 2: Season has NO credits -> expect Show credits
        let season_no_credits = models::SeasonDetails {
            credits: None,
            ..Default::default()
        };
        let result2 = tv_to_unified(show_with_credits, season_no_credits);
        assert_eq!(
            result2.characters[0].voice_actor,
            Some("Show Actor".to_string())
        );
    }

    #[test]
    fn test_tv_to_unified_poster_fallback() {
        // Case 1: Season has poster -> expect Season poster
        let show = models::TvDetails {
            poster_path: Some("/show.jpg".to_string()),
            ..Default::default()
        };
        let season = models::SeasonDetails {
            poster_path: Some("/season.jpg".to_string()),
            ..Default::default()
        };
        let result = tv_to_unified(show.clone(), season);
        assert!(result.cover_image.large.unwrap().contains("/season.jpg"));

        // Case 2: Season has NO poster -> expect Show poster
        let season_no_poster = models::SeasonDetails {
            poster_path: None,
            ..Default::default()
        };
        let result2 = tv_to_unified(show, season_no_poster);
        assert!(result2.cover_image.large.unwrap().contains("/show.jpg"));
    }

    #[test]
    fn test_tv_to_unified_runtime_logic() {
        // Case 1: Show has runtime -> use it
        let show = models::TvDetails {
            episode_run_time: Some(vec![30]),
            ..Default::default()
        };
        let season = models::SeasonDetails {
            episodes: Some(vec![models::EpisodeDetails {
                ..Default::default()
            }]),
            ..Default::default()
        };
        let result = tv_to_unified(show.clone(), season.clone());
        assert_eq!(result.runtime, Some(30));

        // Case 2: Show has NO runtime -> None
        let show_no_runtime = models::TvDetails {
            episode_run_time: None,
            ..Default::default()
        };
        let result2 = tv_to_unified(show_no_runtime, season);
        assert_eq!(result2.runtime, None);
    }
}
