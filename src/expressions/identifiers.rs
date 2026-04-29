//! Identifier extraction from expression formulas.
//!
//! Extracts column identifiers from formula strings by finding
//! word-boundary alphanumeric tokens and excluding known function names.

use regex::Regex;
use lazy_static::lazy_static;
use std::collections::HashSet;

use super::types::KNOWN_FUNCTIONS;

lazy_static! {
    /// Pattern to extract identifiers: letter/underscore followed by alphanumerics.
    static ref IDENTIFIER_PATTERN: Regex = Regex::new(r"\b([A-Za-z_][A-Za-z0-9_]*)\b").unwrap();
}

/// Extract column identifiers from a formula string.
///
/// Finds all identifier tokens (letter/underscore followed by alphanumerics)
/// and excludes known function names. Returns a deduplicated list preserving
/// order of first appearance.
pub fn extract_identifiers(formula: &str) -> Vec<String> {
    let known: HashSet<&str> = KNOWN_FUNCTIONS.iter().copied().collect();
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for cap in IDENTIFIER_PATTERN.captures_iter(formula) {
        let ident = cap.get(1).unwrap().as_str();
        if !known.contains(ident) && seen.insert(ident.to_string()) {
            result.push(ident.to_string());
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_identifiers() {
        let ids = extract_identifiers("weight / (height ** 2)");
        assert_eq!(ids, vec!["weight", "height"]);
    }

    #[test]
    fn test_excludes_known_functions() {
        let ids = extract_identifiers("if_else(bmi < 18.5, 'underweight', 'normal')");
        assert_eq!(ids, vec!["bmi", "underweight", "normal"]);
    }

    #[test]
    fn test_deduplication_preserves_order() {
        let ids = extract_identifiers("a + b + a + c + b");
        assert_eq!(ids, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_excludes_all_known_functions() {
        let ids = extract_identifiers("abs(x) + min(y, z) + sqrt(w)");
        assert_eq!(ids, vec!["x", "y", "z", "w"]);
    }

    #[test]
    fn test_empty_formula() {
        let ids = extract_identifiers("");
        assert!(ids.is_empty());
    }
}
