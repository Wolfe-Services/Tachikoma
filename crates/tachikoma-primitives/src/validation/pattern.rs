//! Regex pattern validation.

use super::{ValidationError, ValidationErrors};
use regex::Regex;

/// Pattern validator for regex.
pub struct PatternValidator {
    /// Maximum pattern length.
    max_length: usize,
    /// Disallowed patterns (e.g., catastrophic backtracking).
    disallowed: Vec<String>,
    /// Maximum compile time (to detect slow patterns).
    max_compile_time_ms: u64,
}

impl Default for PatternValidator {
    fn default() -> Self {
        Self {
            max_length: 1000,
            disallowed: vec![
                // Patterns that can cause catastrophic backtracking
                r"(a+)+".to_string(),
                r"(a*)*".to_string(),
                r"(a|a)+".to_string(),
                r"(a+)+$".to_string(),
                r"(a|a)*".to_string(),
                // Nested quantifiers
                r"(.*)*".to_string(),
                r"(.+)+".to_string(),
                r"(.?)+".to_string(),
                // Alternative catastrophic patterns
                r"(x+x+)+y".to_string(),
                r"(a*a*)+".to_string(),
                r"([a-z]*)*".to_string(),
            ],
            max_compile_time_ms: 100,
        }
    }
}

impl PatternValidator {
    /// Create a new pattern validator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum pattern length.
    pub fn max_length(mut self, max: usize) -> Self {
        self.max_length = max;
        self
    }

    /// Add a disallowed pattern.
    pub fn disallow_pattern(mut self, pattern: &str) -> Self {
        self.disallowed.push(pattern.to_string());
        self
    }

    /// Set maximum compile time in milliseconds.
    pub fn max_compile_time_ms(mut self, ms: u64) -> Self {
        self.max_compile_time_ms = ms;
        self
    }

    /// Validate a regex pattern.
    pub fn validate(&self, pattern: &str) -> ValidationErrors {
        let mut errors = ValidationErrors::new();

        // Check length
        if pattern.len() > self.max_length {
            errors.add(ValidationError::new(
                "pattern",
                &format!("exceeds maximum length of {}", self.max_length),
                "max_length",
            ));
            return errors;
        }

        // Check if pattern is empty
        if pattern.trim().is_empty() {
            errors.add(ValidationError::new(
                "pattern",
                "pattern cannot be empty",
                "not_empty",
            ));
            return errors;
        }

        // Check for potentially dangerous patterns
        for disallowed in &self.disallowed {
            if pattern.contains(disallowed) {
                errors.add(ValidationError::new(
                    "pattern",
                    "pattern may cause performance issues (catastrophic backtracking)",
                    "safe_pattern",
                ).with_suggestion("Avoid nested repetition operators like (a+)+, (a*)*, etc."));
            }
        }

        // Check if it compiles and measure compilation time
        let start = std::time::Instant::now();
        match Regex::new(pattern) {
            Ok(_) => {
                let compile_time = start.elapsed();
                if compile_time.as_millis() as u64 > self.max_compile_time_ms {
                    errors.add(ValidationError::new(
                        "pattern",
                        &format!(
                            "pattern takes too long to compile ({}ms > {}ms)",
                            compile_time.as_millis(),
                            self.max_compile_time_ms
                        ),
                        "compile_time",
                    ).with_suggestion("Simplify the regex pattern to improve performance"));
                }
            }
            Err(e) => {
                errors.add(ValidationError::new(
                    "pattern",
                    &format!("invalid regex: {}", e),
                    "valid_regex",
                ));
                return errors;
            }
        }

        // Additional safety checks
        if self.has_excessive_alternation(pattern) {
            errors.add(ValidationError::new(
                "pattern",
                "pattern has excessive alternation which may impact performance",
                "excessive_alternation",
            ).with_suggestion("Consider simplifying alternation groups"));
        }

        if self.has_deep_nesting(pattern) {
            errors.add(ValidationError::new(
                "pattern",
                "pattern has deep nesting which may impact performance",
                "deep_nesting",
            ).with_suggestion("Reduce nesting levels in the regex"));
        }

        errors
    }

    /// Validate and compile a pattern.
    pub fn validate_and_compile(&self, pattern: &str) -> Result<Regex, ValidationErrors> {
        let errors = self.validate(pattern);
        if !errors.is_empty() {
            return Err(errors);
        }

        Regex::new(pattern).map_err(|e| {
            let mut errors = ValidationErrors::new();
            errors.add(ValidationError::new(
                "pattern",
                &format!("failed to compile: {}", e),
                "compile",
            ));
            errors
        })
    }

    /// Check for excessive alternation (too many | operators).
    fn has_excessive_alternation(&self, pattern: &str) -> bool {
        pattern.matches('|').count() > 20
    }

    /// Check for deep nesting (too many nested groups).
    fn has_deep_nesting(&self, pattern: &str) -> bool {
        let mut depth = 0;
        let mut max_depth = 0;
        
        for ch in pattern.chars() {
            match ch {
                '(' => {
                    depth += 1;
                    max_depth = max_depth.max(depth);
                }
                ')' => {
                    depth = depth.saturating_sub(1);
                }
                _ => {}
            }
        }
        
        max_depth > 10
    }

    /// Validate a replacement pattern for use in substitutions.
    pub fn validate_replacement(&self, replacement: &str) -> ValidationErrors {
        let mut errors = ValidationErrors::new();

        // Check length
        if replacement.len() > self.max_length {
            errors.add(ValidationError::new(
                "replacement",
                &format!("exceeds maximum length of {}", self.max_length),
                "max_length",
            ));
        }

        // Check for potentially dangerous replacement patterns
        if replacement.contains("${") || replacement.contains("$(") {
            errors.add(ValidationError::new(
                "replacement",
                "replacement contains potentially dangerous substitution patterns",
                "safe_replacement",
            ).with_suggestion("Avoid ${} and $() patterns in replacements"));
        }

        errors
    }
}

/// Check if a pattern might cause catastrophic backtracking.
pub fn is_potentially_catastrophic(pattern: &str) -> bool {
    let dangerous_patterns = [
        r"(.*)*",
        r"(.+)+", 
        r"(a+)+",
        r"(a*)*",
        r"(a|a)+",
        r"(a|a)*",
        r"([a-z]*)*",
        r"(.?)+",
    ];
    
    dangerous_patterns.iter().any(|p| pattern.contains(p))
}

/// Estimate the complexity of a regex pattern.
pub fn estimate_complexity(pattern: &str) -> u32 {
    let mut complexity = 0;
    
    // Count quantifiers
    complexity += pattern.matches(['*', '+', '?']).count() as u32 * 2;
    
    // Count alternations
    complexity += pattern.matches('|').count() as u32;
    
    // Count character classes
    complexity += pattern.matches('[').count() as u32;
    
    // Count groups
    complexity += pattern.matches('(').count() as u32;
    
    // Bonus for nested structures
    let mut paren_depth = 0;
    let mut max_depth = 0;
    for ch in pattern.chars() {
        match ch {
            '(' => {
                paren_depth += 1;
                max_depth = max_depth.max(paren_depth);
            }
            ')' => paren_depth = paren_depth.saturating_sub(1),
            _ => {}
        }
    }
    complexity += max_depth * max_depth; // Quadratic penalty for depth
    
    complexity
}

/// Suggest a safer alternative for common problematic patterns.
pub fn suggest_safer_alternative(pattern: &str) -> Option<String> {
    match pattern {
        p if p.contains("(.*)*") => Some("Use .* instead of (.*)* ".to_string()),
        p if p.contains("(.+)+") => Some("Use .+ instead of (.+)+".to_string()),
        p if p.contains("(a+)+") => Some("Use a+ instead of (a+)+".to_string()),
        p if p.contains("(a*)*") => Some("Use a* instead of (a*)*".to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_pattern() {
        let validator = PatternValidator::new();
        let errors = validator.validate(r"fn\s+\w+");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_invalid_pattern() {
        let validator = PatternValidator::new();
        let errors = validator.validate(r"[invalid");
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_pattern_too_long() {
        let validator = PatternValidator::new();
        let long_pattern = "a".repeat(2000);
        let errors = validator.validate(&long_pattern);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_empty_pattern() {
        let validator = PatternValidator::new();
        let errors = validator.validate("");
        assert!(!errors.is_empty());
        
        let errors = validator.validate("   ");
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_catastrophic_backtracking() {
        let validator = PatternValidator::new();
        
        let errors = validator.validate("(a+)+");
        assert!(!errors.is_empty());
        
        let errors = validator.validate("(.*)*");
        assert!(!errors.is_empty());
        
        let errors = validator.validate("(.+)+");
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_excessive_alternation() {
        let validator = PatternValidator::new();
        let pattern_with_many_alternations = (0..25)
            .map(|i| format!("option{}", i))
            .collect::<Vec<_>>()
            .join("|");
        let errors = validator.validate(&pattern_with_many_alternations);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_deep_nesting() {
        let validator = PatternValidator::new();
        let deeply_nested = "(".repeat(15) + "a" + &")".repeat(15);
        let errors = validator.validate(&deeply_nested);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_replacement_validation() {
        let validator = PatternValidator::new();
        
        let errors = validator.validate_replacement("$1");
        assert!(errors.is_empty());
        
        let errors = validator.validate_replacement("${dangerous}");
        assert!(!errors.is_empty());
        
        let errors = validator.validate_replacement("$(command)");
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_catastrophic_detection() {
        assert!(is_potentially_catastrophic("(a+)+"));
        assert!(is_potentially_catastrophic("(.*)*"));
        assert!(!is_potentially_catastrophic("a+"));
        assert!(!is_potentially_catastrophic(".*"));
    }

    #[test]
    fn test_complexity_estimation() {
        assert!(estimate_complexity("a") < estimate_complexity("a+"));
        assert!(estimate_complexity("a+") < estimate_complexity("(a+)+"));
        assert!(estimate_complexity("a|b") < estimate_complexity("a|b|c|d|e"));
    }

    #[test]
    fn test_safer_alternatives() {
        assert_eq!(
            suggest_safer_alternative("(.*)*"),
            Some("Use .* instead of (.*)* ".to_string())
        );
        assert_eq!(
            suggest_safer_alternative("(.+)+"),
            Some("Use .+ instead of (.+)+".to_string())
        );
        assert_eq!(suggest_safer_alternative("simple"), None);
    }

    #[test]
    fn test_validate_and_compile() {
        let validator = PatternValidator::new();
        
        let result = validator.validate_and_compile(r"\d+");
        assert!(result.is_ok());
        
        let result = validator.validate_and_compile(r"[invalid");
        assert!(result.is_err());
    }
}