//! Example property tests demonstrating common patterns.

use proptest::prelude::*;
use crate::strategies::*;

// ============================================
// Pattern: Roundtrip Property
// ============================================

fn encode(s: &str) -> String {
    base64::encode(s)
}

fn decode(s: &str) -> Result<String, base64::DecodeError> {
    base64::decode(s).map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
}

proptest! {
    /// Property: encode then decode returns original string
    #[test]
    fn test_encode_decode_roundtrip(s in "\\PC*") {
        let encoded = encode(&s);
        let decoded = decode(&encoded).expect("decode failed");
        prop_assert_eq!(s, decoded);
    }
}

// ============================================
// Pattern: Invariant Property
// ============================================

fn sort_and_dedupe(mut items: Vec<i32>) -> Vec<i32> {
    items.sort();
    items.dedup();
    items
}

proptest! {
    /// Property: sorted list is always sorted
    #[test]
    fn test_sort_produces_sorted_list(items in prop::collection::vec(any::<i32>(), 0..100)) {
        let result = sort_and_dedupe(items);

        // Check sorted invariant
        for window in result.windows(2) {
            prop_assert!(window[0] <= window[1]);
        }
    }

    /// Property: deduped list has no consecutive duplicates
    #[test]
    fn test_dedupe_removes_duplicates(items in prop::collection::vec(any::<i32>(), 0..100)) {
        let result = sort_and_dedupe(items);

        // Check no consecutive duplicates
        for window in result.windows(2) {
            prop_assert_ne!(window[0], window[1]);
        }
    }
}

// ============================================
// Pattern: Oracle Property (compare implementations)
// ============================================

fn my_max(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}

proptest! {
    /// Property: our max matches std max
    #[test]
    fn test_max_matches_std(a in any::<i32>(), b in any::<i32>()) {
        prop_assert_eq!(my_max(a, b), std::cmp::max(a, b));
    }
}

// ============================================
// Pattern: Metamorphic Property
// ============================================

fn search(haystack: &str, needle: &str) -> bool {
    haystack.contains(needle)
}

proptest! {
    /// Metamorphic: if needle found in part, found in whole
    #[test]
    fn test_search_metamorphic(
        prefix in "[a-z]{0,10}",
        needle in "[a-z]{1,5}",
        suffix in "[a-z]{0,10}"
    ) {
        let haystack = format!("{}{}{}", prefix, needle, suffix);
        prop_assert!(search(&haystack, &needle));
    }
}

// ============================================
// Pattern: Domain-Specific Properties
// ============================================

proptest! {
    /// Property: file paths don't contain double slashes after normalization
    #[test]
    fn test_path_normalization(path in valid_file_path()) {
        // Simulate path normalization
        let normalized = path.replace("//", "/");
        prop_assert!(!normalized.contains("//"));
    }

    /// Property: valid identifiers match expected pattern
    #[test]
    fn test_identifier_format(id in valid_identifier()) {
        prop_assert!(id.chars().next().unwrap().is_ascii_lowercase());
        prop_assert!(id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'));
        prop_assert!(id.len() <= 32);
    }
}