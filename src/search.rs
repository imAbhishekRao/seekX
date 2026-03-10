#[derive(Clone, Debug)]
pub struct MatchScore {
    pub score: i64,
    pub start_idx: usize,
}

pub fn score(
    query: &str,
    search_terms: &[String],
    normalized_terms: &[String],
) -> Option<MatchScore> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Some(MatchScore {
            score: 1,
            start_idx: 0,
        });
    }

    let normalized_query = compact_alnum(&q);
    let mut best_score = 0i64;
    let mut best_idx = usize::MAX;

    for (idx, term) in search_terms.iter().enumerate() {
        if let Some(pos) = term.find(&q) {
            let base = if idx == 0 { 4000 } else { 3000 };
            let score = (base as i64) - pos as i64;
            if score > best_score {
                best_score = score;
                best_idx = idx;
            }
        }
    }

    if !normalized_query.is_empty() {
        for (idx, term) in normalized_terms.iter().enumerate() {
            if let Some(pos) = term.find(&normalized_query) {
                let base = if idx == 0 { 2000 } else { 1500 };
                let score = (base as i64) - pos as i64;
                if score > best_score {
                    best_score = score;
                    best_idx = idx;
                }
            }
        }

        for (idx, term) in normalized_terms.iter().enumerate() {
            if is_subsequence(&normalized_query, term) {
                let score = if idx == 0 { 3600 } else { 1700 };
                if (score as i64) > best_score {
                    best_score = score as i64;
                    best_idx = idx;
                }
            }
        }
    }

    if best_score > 0 {
        Some(MatchScore {
            score: best_score,
            start_idx: if best_idx == usize::MAX { 0 } else { best_idx },
        })
    } else {
        None
    }
}

fn compact_alnum(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}

fn is_subsequence(needle: &str, haystack: &str) -> bool {
    if needle.is_empty() || needle.len() > haystack.len() {
        return false;
    }

    let mut pos = 0usize;
    let needle_chars: Vec<char> = needle.chars().collect();
    for ch in haystack.chars() {
        if ch == needle_chars[pos] {
            pos += 1;
            if pos == needle_chars.len() {
                return true;
            }
        }
    }

    false
}
