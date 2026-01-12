# 420 - Action Tracking

## Overview

User action tracking for clicks, form submissions, and custom interactions with automatic element identification.

## Rust Implementation

```rust
// crates/analytics/src/actions.rs

use crate::event_types::{AnalyticsEvent, EventCategory, ActionEvent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// User action types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    Click,
    Submit,
    Change,
    Focus,
    Blur,
    Scroll,
    Copy,
    Download,
    Custom,
}

/// Captured user action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAction {
    /// Action type
    pub action_type: ActionType,
    /// Action name (custom or derived)
    pub name: String,
    /// Target element info
    pub element: Option<ElementInfo>,
    /// Page URL where action occurred
    pub page_url: String,
    /// Page path
    pub page_path: String,
    /// Position on page
    pub position: Option<Position>,
    /// Value (for forms)
    pub value: Option<serde_json::Value>,
    /// Custom properties
    pub properties: HashMap<String, serde_json::Value>,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Element information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementInfo {
    /// HTML tag name
    pub tag: String,
    /// Element ID
    pub id: Option<String>,
    /// CSS classes
    pub classes: Vec<String>,
    /// Element text content
    pub text: Option<String>,
    /// Element href (for links)
    pub href: Option<String>,
    /// Element name attribute
    pub name: Option<String>,
    /// CSS selector path
    pub selector: Option<String>,
    /// nth-child index
    pub nth_child: Option<u32>,
    /// nth-of-type index
    pub nth_of_type: Option<u32>,
    /// Parent element info (limited depth)
    pub parent: Option<Box<ElementInfo>>,
    /// Data attributes
    pub data_attributes: HashMap<String, String>,
}

impl ElementInfo {
    /// Generate a unique selector for this element
    pub fn unique_selector(&self) -> String {
        let mut parts = Vec::new();

        // Start with tag
        parts.push(self.tag.clone());

        // Add ID if available (most specific)
        if let Some(ref id) = self.id {
            return format!("{}#{}", self.tag, id);
        }

        // Add relevant classes
        for class in &self.classes {
            if !is_dynamic_class(class) {
                parts.push(format!(".{}", class));
            }
        }

        // Add data attributes
        for (key, value) in &self.data_attributes {
            if key.starts_with("data-test") || key.starts_with("data-analytics") {
                parts.push(format!("[{}=\"{}\"]", key, value));
            }
        }

        // Add nth-child if needed for uniqueness
        if parts.len() == 1 {
            if let Some(nth) = self.nth_of_type {
                parts.push(format!(":nth-of-type({})", nth));
            }
        }

        parts.join("")
    }

    /// Get display text for the element
    pub fn display_text(&self) -> Option<String> {
        self.text.clone()
            .or_else(|| self.data_attributes.get("data-label").cloned())
            .or_else(|| self.name.clone())
            .or_else(|| self.id.clone())
    }
}

fn is_dynamic_class(class: &str) -> bool {
    // Common patterns for dynamically generated class names
    class.contains("__") ||  // BEM modifiers
    class.chars().any(|c| c.is_numeric() && class.len() > 10) || // Hash classes
    class.starts_with("css-") ||  // CSS-in-JS
    class.starts_with("sc-") ||   // Styled components
    class.starts_with("emotion-") // Emotion
}

/// Position on page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub viewport_x: Option<i32>,
    pub viewport_y: Option<i32>,
}

impl UserAction {
    /// Convert to analytics event
    pub fn to_event(&self, distinct_id: &str) -> AnalyticsEvent {
        let event_name = match self.action_type {
            ActionType::Click => "$click",
            ActionType::Submit => "$submit",
            ActionType::Change => "$change",
            _ => &self.name,
        };

        let mut event = AnalyticsEvent::new(event_name, distinct_id, EventCategory::Action);

        event.properties.insert("$action_type".to_string(),
            serde_json::json!(format!("{:?}", self.action_type).to_lowercase()));
        event.properties.insert("$current_url".to_string(), serde_json::json!(self.page_url));
        event.properties.insert("$pathname".to_string(), serde_json::json!(self.page_path));

        if let Some(ref element) = self.element {
            event.properties.insert("$element_tag".to_string(), serde_json::json!(element.tag));

            if let Some(ref id) = element.id {
                event.properties.insert("$element_id".to_string(), serde_json::json!(id));
            }

            if !element.classes.is_empty() {
                event.properties.insert("$element_classes".to_string(),
                    serde_json::json!(element.classes));
            }

            if let Some(ref text) = element.text {
                event.properties.insert("$element_text".to_string(),
                    serde_json::json!(truncate_text(text, 100)));
            }

            if let Some(ref href) = element.href {
                event.properties.insert("$element_href".to_string(), serde_json::json!(href));
            }

            if let Some(ref selector) = element.selector {
                event.properties.insert("$element_selector".to_string(), serde_json::json!(selector));
            }

            // Include data-analytics attributes
            for (key, value) in &element.data_attributes {
                if key.starts_with("data-analytics") {
                    let prop_key = key.replace("data-analytics-", "");
                    event.properties.insert(prop_key, serde_json::json!(value));
                }
            }
        }

        if let Some(ref pos) = self.position {
            event.properties.insert("$click_x".to_string(), serde_json::json!(pos.x));
            event.properties.insert("$click_y".to_string(), serde_json::json!(pos.y));
        }

        // Merge custom properties
        for (key, value) in &self.properties {
            event.properties.insert(key.clone(), value.clone());
        }

        event
    }
}

fn truncate_text(text: &str, max_len: usize) -> String {
    let trimmed = text.trim();
    if trimmed.len() <= max_len {
        trimmed.to_string()
    } else {
        format!("{}...", &trimmed[..max_len])
    }
}

/// Autocapture configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutocaptureConfig {
    /// Enable autocapture
    pub enabled: bool,
    /// DOM events to capture
    pub events: Vec<DomEventConfig>,
    /// CSS selectors to ignore
    pub ignore_selectors: Vec<String>,
    /// CSS selectors to always capture
    pub capture_selectors: Vec<String>,
    /// Capture text content
    pub capture_text: bool,
    /// Max text length
    pub max_text_length: usize,
    /// Sensitive fields to mask
    pub sensitive_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomEventConfig {
    /// DOM event type
    pub event_type: String,
    /// Element selectors to track
    pub selectors: Vec<String>,
    /// Capture value for this event
    pub capture_value: bool,
}

impl Default for AutocaptureConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            events: vec![
                DomEventConfig {
                    event_type: "click".to_string(),
                    selectors: vec![
                        "a".to_string(),
                        "button".to_string(),
                        "[role=\"button\"]".to_string(),
                        "input[type=\"submit\"]".to_string(),
                        "input[type=\"button\"]".to_string(),
                    ],
                    capture_value: false,
                },
                DomEventConfig {
                    event_type: "submit".to_string(),
                    selectors: vec!["form".to_string()],
                    capture_value: false,
                },
                DomEventConfig {
                    event_type: "change".to_string(),
                    selectors: vec!["select".to_string()],
                    capture_value: true,
                },
            ],
            ignore_selectors: vec![
                "[data-analytics-ignore]".to_string(),
                ".analytics-ignore".to_string(),
            ],
            capture_selectors: vec![
                "[data-analytics-capture]".to_string(),
            ],
            capture_text: true,
            max_text_length: 100,
            sensitive_fields: vec![
                "password".to_string(),
                "credit-card".to_string(),
                "ssn".to_string(),
                "secret".to_string(),
            ],
        }
    }
}

/// Action definitions for named actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDefinition {
    /// Action identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Matching rules
    pub rules: Vec<ActionRule>,
    /// Tags for organization
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRule {
    /// Rule type
    pub rule_type: ActionRuleType,
    /// Value to match
    pub value: String,
    /// Match type
    pub match_type: MatchType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionRuleType {
    Url,
    UrlPath,
    Selector,
    ElementId,
    ElementClass,
    ElementText,
    ElementHref,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchType {
    Exact,
    Contains,
    StartsWith,
    EndsWith,
    Regex,
}

impl ActionDefinition {
    /// Check if an action matches this definition
    pub fn matches(&self, action: &UserAction) -> bool {
        self.rules.iter().all(|rule| rule.matches(action))
    }
}

impl ActionRule {
    pub fn matches(&self, action: &UserAction) -> bool {
        let value_to_check = match self.rule_type {
            ActionRuleType::Url => Some(action.page_url.as_str()),
            ActionRuleType::UrlPath => Some(action.page_path.as_str()),
            ActionRuleType::Selector => action.element.as_ref()
                .and_then(|e| e.selector.as_deref()),
            ActionRuleType::ElementId => action.element.as_ref()
                .and_then(|e| e.id.as_deref()),
            ActionRuleType::ElementClass => None, // Need special handling
            ActionRuleType::ElementText => action.element.as_ref()
                .and_then(|e| e.text.as_deref()),
            ActionRuleType::ElementHref => action.element.as_ref()
                .and_then(|e| e.href.as_deref()),
        };

        if let Some(value) = value_to_check {
            self.match_value(value)
        } else if self.rule_type == ActionRuleType::ElementClass {
            // Check if any class matches
            if let Some(ref element) = action.element {
                element.classes.iter().any(|c| self.match_value(c))
            } else {
                false
            }
        } else {
            false
        }
    }

    fn match_value(&self, value: &str) -> bool {
        match self.match_type {
            MatchType::Exact => value == self.value,
            MatchType::Contains => value.contains(&self.value),
            MatchType::StartsWith => value.starts_with(&self.value),
            MatchType::EndsWith => value.ends_with(&self.value),
            MatchType::Regex => {
                regex::Regex::new(&self.value)
                    .map(|re| re.is_match(value))
                    .unwrap_or(false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_unique_selector() {
        let element = ElementInfo {
            tag: "button".to_string(),
            id: Some("signup-btn".to_string()),
            classes: vec!["primary".to_string()],
            text: Some("Sign Up".to_string()),
            href: None,
            name: None,
            selector: None,
            nth_child: None,
            nth_of_type: None,
            parent: None,
            data_attributes: HashMap::new(),
        };

        assert_eq!(element.unique_selector(), "button#signup-btn");
    }

    #[test]
    fn test_element_selector_without_id() {
        let element = ElementInfo {
            tag: "a".to_string(),
            id: None,
            classes: vec!["nav-link".to_string(), "css-abc123".to_string()],
            text: Some("Home".to_string()),
            href: Some("/".to_string()),
            name: None,
            selector: None,
            nth_child: None,
            nth_of_type: Some(2),
            parent: None,
            data_attributes: HashMap::new(),
        };

        // Should exclude dynamic class
        assert!(element.unique_selector().contains(".nav-link"));
        assert!(!element.unique_selector().contains("css-abc123"));
    }

    #[test]
    fn test_action_rule_matching() {
        let rule = ActionRule {
            rule_type: ActionRuleType::UrlPath,
            value: "/pricing".to_string(),
            match_type: MatchType::Exact,
        };

        let action = UserAction {
            action_type: ActionType::Click,
            name: "click".to_string(),
            element: None,
            page_url: "https://example.com/pricing".to_string(),
            page_path: "/pricing".to_string(),
            position: None,
            value: None,
            properties: HashMap::new(),
            timestamp: chrono::Utc::now(),
        };

        assert!(rule.matches(&action));
    }
}
```

## TypeScript Autocapture

```typescript
// Browser autocapture implementation
class Autocapture {
  private config: AutocaptureConfig;

  constructor(config: AutocaptureConfig) {
    this.config = config;
    this.setupListeners();
  }

  private setupListeners(): void {
    document.addEventListener('click', this.handleClick.bind(this), true);
    document.addEventListener('submit', this.handleSubmit.bind(this), true);
  }

  private handleClick(event: MouseEvent): void {
    const target = event.target as HTMLElement;

    if (this.shouldCapture(target)) {
      const action = this.captureElement(target, 'click', event);
      analytics.capture('$click', action);
    }
  }

  private captureElement(element: HTMLElement, actionType: string, event: Event): object {
    return {
      $element_tag: element.tagName.toLowerCase(),
      $element_id: element.id || undefined,
      $element_classes: Array.from(element.classList),
      $element_text: this.getElementText(element),
      $element_href: (element as HTMLAnchorElement).href,
    };
  }
}
```

## Related Specs

- 419-pageview-tracking.md - Page context
- 421-error-tracking.md - Error events
- 412-event-schema.md - Event validation
