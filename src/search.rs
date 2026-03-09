#[derive(Clone, Debug)]
pub struct MatchScore {
    pub score: i64,
    pub start_idx: usize,
}

pub fn score(query: &str, haystack: &str) -> Option<MatchScore> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Some(MatchScore {
            score: 1,
            start_idx: 0,
        });
    }

    let h = haystack.to_lowercase();

    if let Some(idx) = h.find(&q) {
        let positional_bonus = 1_000_i64.saturating_sub(idx as i64 * 4);
        let length_bonus = 400_i64.saturating_sub((h.len() as i64 - q.len() as i64).max(0));
        return Some(MatchScore {
            score: 10_000 + positional_bonus + length_bonus,
            start_idx: idx,
        });
    }

    // Compact normalization helps match queries like "local send" to "LocalSend"
    // without re-introducing very loose fuzzy matching.
    let q_compact = compact_alnum(&q);
    let h_compact = compact_alnum(&h);
    if !q_compact.is_empty() {
        if let Some(idx) = h_compact.find(&q_compact) {
            let positional_bonus = 700_i64.saturating_sub(idx as i64 * 3);
            let length_bonus =
                250_i64.saturating_sub((h_compact.len() as i64 - q_compact.len() as i64).max(0));
            return Some(MatchScore {
                score: 8_000 + positional_bonus + length_bonus,
                start_idx: idx,
            });
        }
    }

    None
}

fn compact_alnum(input: &str) -> String {
    input.chars().filter(|c| c.is_ascii_alphanumeric()).collect()
}
