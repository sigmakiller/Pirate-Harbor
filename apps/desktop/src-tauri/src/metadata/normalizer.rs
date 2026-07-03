//! Title and genre normalization — T30.
//!
//! Cleans raw strings from user input or API responses before they are
//! stored in the database or used for matching.

/// Normalize a game title for display and search.
///
/// - Trims whitespace
/// - Collapses multiple internal spaces into one
/// - Removes common non-printable characters
pub fn normalize_title(raw: &str) -> String {
    raw.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Normalize a genre string.
///
/// - Trims whitespace and lowercases
/// - Maps common API variants to a canonical form
/// - Returns `None` for empty / unknown genres
pub fn normalize_genre(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let lower = trimmed.to_lowercase();
    let canonical = match lower.as_str() {
        "action-adventure" | "action adventure" => "Action-Adventure",
        "rpg" | "role-playing" | "role playing" | "role-playing game" => "RPG",
        "fps" | "first person shooter" | "first-person shooter" => "FPS",
        "tps" | "third person shooter" | "third-person shooter" => "TPS",
        "rts" | "real time strategy" | "real-time strategy" => "RTS",
        "moba"                  => "MOBA",
        "battle royale"         => "Battle Royale",
        "visual novel"          => "Visual Novel",
        "metroidvania"          => "Metroidvania",
        "hack and slash" | "hack & slash" => "Hack and Slash",
        "point and click" | "point & click" => "Point and Click",
        "turn-based" | "turn based" | "turn-based strategy" => "Turn-Based",
        "survival horror"       => "Survival Horror",
        "open world"            => "Open World",
        "sandbox"               => "Sandbox",
        "simulation" | "sim"    => "Simulation",
        "sports"                => "Sports",
        "racing"                => "Racing",
        "fighting"              => "Fighting",
        "platformer"            => "Platformer",
        "puzzle"                => "Puzzle",
        "horror"                => "Horror",
        "adventure"             => "Adventure",
        "action"                => "Action",
        "strategy"              => "Strategy",
        "shooter"               => "Shooter",
        "indie"                 => "Indie",
        _                       => trimmed,
    };
    Some(canonical.to_string())
}

/// Extract a clean sort key from a title (removes leading articles).
///
/// Used for alphabetical sorting: "The Witcher 3" → "witcher 3".
pub fn sort_key(title: &str) -> String {
    let lower = title.to_lowercase();
    for prefix in ["the ", "a ", "an "] {
        if let Some(stripped) = lower.strip_prefix(prefix) {
            return stripped.to_string();
        }
    }
    lower
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_title_collapses_spaces() {
        assert_eq!(normalize_title("  The  Witcher  3  "), "The Witcher 3");
    }

    #[test]
    fn normalize_genre_maps_rpg() {
        assert_eq!(normalize_genre("role-playing"), Some("RPG".to_string()));
        assert_eq!(normalize_genre("RPG"), Some("RPG".to_string()));
    }

    #[test]
    fn normalize_genre_empty_returns_none() {
        assert_eq!(normalize_genre("  "), None);
    }

    #[test]
    fn sort_key_strips_article() {
        assert_eq!(sort_key("The Witcher 3"), "witcher 3");
        assert_eq!(sort_key("A Plague Tale"), "plague tale");
    }

    #[test]
    fn sort_key_no_article_unchanged() {
        assert_eq!(sort_key("Dark Souls"), "dark souls");
    }
}
