#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKind {
    Tv,
    Movie,
    Ova,
    Ona,
    Special,
    Other,
}

impl MediaKind {
    pub fn from_request(value: Option<&str>) -> Option<Self> {
        match value?.trim().to_ascii_lowercase().as_str() {
            "tv" => Some(Self::Tv),
            "movie" => Some(Self::Movie),
            "ova" => Some(Self::Ova),
            "ona" | "web" => Some(Self::Ona),
            "special" => Some(Self::Special),
            _ => None,
        }
    }
}

pub fn title_candidates(primary: Option<&str>, aliases: &[String], limit: usize) -> Vec<String> {
    let mut values = Vec::new();

    for value in primary
        .into_iter()
        .chain(aliases.iter().map(String::as_str))
    {
        let value = value.trim();
        if value.is_empty() {
            continue;
        }

        let key = canonical_title(value);
        if key.is_empty()
            || values
                .iter()
                .any(|existing: &String| canonical_title(existing) == key)
        {
            continue;
        }

        values.push(value.to_string());
        if values.len() == limit {
            break;
        }
    }

    values
}

pub fn canonical_title(value: &str) -> String {
    value
        .chars()
        .flat_map(char::to_lowercase)
        .filter(|ch| ch.is_alphanumeric())
        .collect()
}

pub fn title_score<'a>(
    expected: &[String],
    candidate_titles: impl IntoIterator<Item = &'a str>,
) -> i32 {
    let expected: Vec<String> = expected
        .iter()
        .map(|title| canonical_title(title))
        .filter(|title| !title.is_empty())
        .collect();

    let mut best = 0;
    for candidate in candidate_titles {
        let candidate = canonical_title(candidate);
        if candidate.is_empty() {
            continue;
        }

        for expected in &expected {
            let score = if candidate == *expected {
                100
            } else if candidate.len().min(expected.len()) >= 4
                && (candidate.contains(expected) || expected.contains(&candidate))
            {
                55
            } else {
                0
            };
            best = best.max(score);
        }
    }

    best
}

pub fn year_score(expected: Option<i32>, actual: Option<i32>) -> i32 {
    match (expected, actual) {
        (Some(expected), Some(actual)) if expected == actual => 30,
        (Some(expected), Some(actual)) if (expected - actual).abs() == 1 => 12,
        (Some(_), Some(_)) => -30,
        _ => 0,
    }
}

pub fn score_thresholds(year: Option<i32>, kind: Option<MediaKind>) -> (i32, i32) {
    if year.is_none() && kind.is_none() {
        (90, 100)
    } else {
        (105, 130)
    }
}

pub fn media_kind_score(expected: Option<MediaKind>, actual: MediaKind) -> i32 {
    let Some(expected) = expected else {
        return 0;
    };

    if expected == actual {
        return 30;
    }

    match (expected, actual) {
        (MediaKind::Tv | MediaKind::Ova | MediaKind::Ona | MediaKind::Special, MediaKind::Tv) => 18,
        (MediaKind::Tv, MediaKind::Ova | MediaKind::Ona | MediaKind::Special) => 12,
        (
            MediaKind::Ova | MediaKind::Ona | MediaKind::Special,
            MediaKind::Ova | MediaKind::Ona | MediaKind::Special,
        ) => 8,
        (MediaKind::Movie, _) | (_, MediaKind::Movie) => -30,
        _ => -10,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_candidates_deduplicate_punctuation_and_case() {
        let aliases = vec![
            "Test Anime".to_string(),
            "テストアニメ".to_string(),
            "test-anime".to_string(),
        ];
        assert_eq!(
            title_candidates(Some("Test Anime"), &aliases, 6),
            vec!["Test Anime", "テストアニメ"]
        );
    }

    #[test]
    fn title_score_prefers_exact_normalized_match() {
        let expected = vec!["SPY×FAMILY".to_string(), "Spy Family".to_string()];
        assert_eq!(title_score(&expected, ["spy family"]), 100);
        assert_eq!(title_score(&expected, ["Spy Family Part 2"]), 55);
        assert_eq!(title_score(&expected, ["Completely Different"]), 0);
    }

    #[test]
    fn year_score_rejects_distant_remakes() {
        assert_eq!(year_score(Some(2026), Some(2026)), 30);
        assert_eq!(year_score(Some(2026), Some(2025)), 12);
        assert_eq!(year_score(Some(2026), Some(2020)), -30);
    }

    #[test]
    fn movie_mismatch_is_strongly_penalized() {
        assert_eq!(media_kind_score(Some(MediaKind::Movie), MediaKind::Tv), -30);
        assert!(media_kind_score(Some(MediaKind::Ona), MediaKind::Tv) > 0);
    }
}
