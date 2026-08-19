//! Rust port of `goto.js` `fuzzy` / `scoreEntry`.

pub struct ScoreFields<'a> {
    pub title: &'a str,
    pub path: &'a str,
    pub description: Option<&'a str>,
    pub url: Option<&'a str>,
}

pub fn fuzzy(query: &str, text: &str) -> f64 {
    let q = query.to_lowercase();
    let t = text.to_lowercase();
    if q.is_empty() {
        return 0.0;
    }
    if t.is_empty() {
        return -1.0;
    }
    if let Some(exact) = t.find(&q) {
        return 1200.0 - exact as f64 + q.len().min(80) as f64;
    }
    let q_chars: Vec<char> = q.chars().collect();
    let t_chars: Vec<char> = t.chars().collect();
    let mut ti = 0usize;
    let mut score = 0.0;
    let mut run = 0.0;
    for &qc in &q_chars {
        let found = match t_chars[ti..].iter().position(|&tc| tc == qc) {
            Some(offset) => ti + offset,
            None => return -1.0,
        };
        run = if found == ti { run + 1.0 } else { 1.0 };
        score += run * 5.0 - (found as f64 - ti as f64);
        ti = found + 1;
    }
    score
}

pub fn score_entry(query: &str, entry: ScoreFields<'_>) -> f64 {
    if query.is_empty() {
        return 1.0;
    }
    let parts: [(&str, f64); 4] = [
        (entry.title, 1.0),
        (entry.path, 0.85),
        (entry.description.unwrap_or(""), 0.4),
        (entry.url.unwrap_or(""), 0.7),
    ];
    let mut best: f64 = -1.0;
    for (text, weight) in parts {
        let next = fuzzy(query, text);
        if next >= 0.0 {
            best = best.max(next * weight);
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_query_scores_one() {
        let score = score_entry(
            "",
            ScoreFields {
                title: "CLI entry points",
                path: "plans/cli-entry-points.md",
                description: None,
                url: Some("/plans/cli-entry-points/"),
            },
        );
        assert_eq!(score, 1.0);
    }

    #[test]
    fn substring_matches_goto_js() {
        assert_eq!(fuzzy("cli", "cli entry points"), 1203.0);
        assert_eq!(fuzzy("cli", "plans/cli-entry-points.md"), 1197.0);
        let score = score_entry(
            "cli",
            ScoreFields {
                title: "CLI entry points",
                path: "plans/cli-entry-points.md",
                description: None,
                url: None,
            },
        );
        assert_eq!(score, 1203.0);
    }

    #[test]
    fn subsequence_matches_goto_js() {
        assert_eq!(fuzzy("abc", "a-b-c"), 13.0);
        assert_eq!(fuzzy("xyz", "abc"), -1.0);
        assert_eq!(fuzzy("q", ""), -1.0);
    }
}
