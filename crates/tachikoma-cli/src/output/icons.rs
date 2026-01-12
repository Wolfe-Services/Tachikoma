//! Unicode icons for CLI output.

use std::env;

/// Status icons
pub struct Icons;

impl Icons {
    pub const CHECK: &'static str = "✓";
    pub const CROSS: &'static str = "✗";
    pub const WARNING: &'static str = "⚠";
    pub const INFO: &'static str = "ℹ";
    pub const ARROW: &'static str = "→";
    pub const BULLET: &'static str = "•";
    pub const SPINNER: [&'static str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

    /// Fallback ASCII versions
    pub const CHECK_ASCII: &'static str = "[ok]";
    pub const CROSS_ASCII: &'static str = "[err]";
    pub const WARNING_ASCII: &'static str = "[warn]";
    pub const INFO_ASCII: &'static str = "[info]";
    pub const ARROW_ASCII: &'static str = "->";
    pub const BULLET_ASCII: &'static str = "*";
}

/// Icon context that handles unicode support detection
pub struct IconContext {
    unicode: bool,
}

impl IconContext {
    pub fn new() -> Self {
        Self {
            unicode: detect_unicode_support(),
        }
    }

    pub fn check(&self) -> &'static str {
        if self.unicode { Icons::CHECK } else { Icons::CHECK_ASCII }
    }

    pub fn cross(&self) -> &'static str {
        if self.unicode { Icons::CROSS } else { Icons::CROSS_ASCII }
    }

    pub fn warning(&self) -> &'static str {
        if self.unicode { Icons::WARNING } else { Icons::WARNING_ASCII }
    }

    pub fn info(&self) -> &'static str {
        if self.unicode { Icons::INFO } else { Icons::INFO_ASCII }
    }

    pub fn arrow(&self) -> &'static str {
        if self.unicode { Icons::ARROW } else { Icons::ARROW_ASCII }
    }

    pub fn bullet(&self) -> &'static str {
        if self.unicode { Icons::BULLET } else { Icons::BULLET_ASCII }
    }
}

impl Default for IconContext {
    fn default() -> Self {
        Self::new()
    }
}

fn detect_unicode_support() -> bool {
    env::var("TERM")
        .map(|t| !t.contains("linux"))
        .unwrap_or(true)
        && env::var("LANG")
            .map(|l| l.to_uppercase().contains("UTF"))
            .unwrap_or(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_constants() {
        assert_eq!(Icons::CHECK, "✓");
        assert_eq!(Icons::CROSS, "✗");
        assert_eq!(Icons::WARNING, "⚠");
        assert_eq!(Icons::INFO, "ℹ");
        assert_eq!(Icons::ARROW, "→");
        assert_eq!(Icons::BULLET, "•");
    }

    #[test]
    fn test_icon_context() {
        let ctx = IconContext::new();
        // These tests will depend on the environment, but ensure they return valid strings
        assert!(!ctx.check().is_empty());
        assert!(!ctx.cross().is_empty());
        assert!(!ctx.warning().is_empty());
        assert!(!ctx.info().is_empty());
    }
}