//! File suggestion utilities for error messages.

use std::path::Path;

/// Find similar file names in the same directory.
pub fn find_similar_files(path: &Path, max_suggestions: usize) -> Vec<String> {
    let file_name = match path.file_name().and_then(|n| n.to_str()) {
        Some(name) => name,
        None => return Vec::new(),
    };

    let parent = match path.parent() {
        Some(p) => p,
        None => return Vec::new(),
    };

    let entries = match std::fs::read_dir(parent) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut candidates: Vec<(String, usize)> = entries
        .filter_map(|e| e.ok())
        .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
        .filter(|name| name != file_name)
        .map(|name| {
            let distance = levenshtein_distance(file_name, &name);
            (name, distance)
        })
        .filter(|(_, distance)| *distance <= 3) // Only suggest if close enough
        .collect();

    candidates.sort_by_key(|(_, d)| *d);
    candidates.truncate(max_suggestions);

    candidates.into_iter().map(|(name, _)| name).collect()
}

/// Calculate Levenshtein distance between two strings.
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut matrix = vec![vec![0usize; b_len + 1]; a_len + 1];

    for i in 0..=a_len {
        matrix[i][0] = i;
    }
    for j in 0..=b_len {
        matrix[0][j] = j;
    }

    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }

    matrix[a_len][b_len]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("main.rs", "mian.rs"), 2);
        assert_eq!(levenshtein_distance("test", "test"), 0);
    }
}