# 421 - Error Tracking

## Overview

Client-side and server-side error tracking with stack trace parsing, error grouping, and alerting integration.

## Rust Implementation

```rust
// crates/analytics/src/errors.rs

use crate::event_types::{AnalyticsEvent, EventCategory};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sha2::{Sha256, Digest};

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Fatal,
}

impl Default for ErrorSeverity {
    fn default() -> Self {
        Self::Error
    }
}

/// Captured error event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    /// Error type/class name
    pub error_type: String,
    /// Error message
    pub message: String,
    /// Stack trace
    pub stack_trace: Option<StackTrace>,
    /// Severity level
    pub severity: ErrorSeverity,
    /// Error fingerprint for grouping
    pub fingerprint: String,
    /// Page URL where error occurred
    pub url: Option<String>,
    /// User agent
    pub user_agent: Option<String>,
    /// Browser info
    pub browser: Option<BrowserInfo>,
    /// OS info
    pub os: Option<OsInfo>,
    /// Custom tags
    pub tags: HashMap<String, String>,
    /// Additional context
    pub context: HashMap<String, serde_json::Value>,
    /// Breadcrumbs (recent actions before error)
    pub breadcrumbs: Vec<Breadcrumb>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Is handled or unhandled
    pub handled: bool,
    /// Error source (javascript, rust, api, etc.)
    pub source: ErrorSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorSource {
    JavaScript,
    React,
    Api,
    Rust,
    Database,
    Network,
    Unknown,
}

/// Stack trace representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackTrace {
    /// Stack frames
    pub frames: Vec<StackFrame>,
    /// Raw stack string
    pub raw: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    /// Function name
    pub function: Option<String>,
    /// File name
    pub filename: Option<String>,
    /// Line number
    pub lineno: Option<u32>,
    /// Column number
    pub colno: Option<u32>,
    /// Source context (surrounding lines)
    pub context: Option<FrameContext>,
    /// Is in-app frame (not library)
    pub in_app: bool,
    /// Module/package name
    pub module: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameContext {
    /// Lines before
    pub pre_context: Vec<String>,
    /// The offending line
    pub context_line: String,
    /// Lines after
    pub post_context: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsInfo {
    pub name: String,
    pub version: Option<String>,
}

/// Breadcrumb for error context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breadcrumb {
    /// Breadcrumb type
    pub breadcrumb_type: BreadcrumbType,
    /// Category
    pub category: String,
    /// Message
    pub message: Option<String>,
    /// Additional data
    pub data: HashMap<String, serde_json::Value>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Level
    pub level: ErrorSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BreadcrumbType {
    Navigation,
    Http,
    Ui,
    Console,
    Debug,
    Error,
    User,
    Query,
}

impl ErrorEvent {
    /// Generate fingerprint for error grouping
    pub fn generate_fingerprint(&mut self) {
        let mut hasher = Sha256::new();

        // Include error type
        hasher.update(self.error_type.as_bytes());

        // Include normalized message (remove variable parts)
        let normalized_message = normalize_error_message(&self.message);
        hasher.update(normalized_message.as_bytes());

        // Include top stack frame
        if let Some(ref stack) = self.stack_trace {
            if let Some(frame) = stack.frames.iter().find(|f| f.in_app) {
                if let Some(ref filename) = frame.filename {
                    hasher.update(filename.as_bytes());
                }
                if let Some(ref function) = frame.function {
                    hasher.update(function.as_bytes());
                }
                if let Some(lineno) = frame.lineno {
                    hasher.update(lineno.to_string().as_bytes());
                }
            }
        }

        let result = hasher.finalize();
        self.fingerprint = hex::encode(&result[..16]);
    }

    /// Convert to analytics event
    pub fn to_event(&self, distinct_id: &str) -> AnalyticsEvent {
        let mut event = AnalyticsEvent::new("$error", distinct_id, EventCategory::Error);

        event.properties.insert("$error_type".to_string(),
            serde_json::json!(self.error_type));
        event.properties.insert("$error_message".to_string(),
            serde_json::json!(self.message));
        event.properties.insert("$error_fingerprint".to_string(),
            serde_json::json!(self.fingerprint));
        event.properties.insert("$error_severity".to_string(),
            serde_json::json!(format!("{:?}", self.severity).to_lowercase()));
        event.properties.insert("$error_handled".to_string(),
            serde_json::json!(self.handled));
        event.properties.insert("$error_source".to_string(),
            serde_json::json!(format!("{:?}", self.source).to_lowercase()));

        if let Some(ref url) = self.url {
            event.properties.insert("$current_url".to_string(), serde_json::json!(url));
        }

        if let Some(ref browser) = self.browser {
            event.properties.insert("$browser".to_string(), serde_json::json!(browser.name));
            event.properties.insert("$browser_version".to_string(),
                serde_json::json!(browser.version));
        }

        if let Some(ref os) = self.os {
            event.properties.insert("$os".to_string(), serde_json::json!(os.name));
        }

        if let Some(ref stack) = self.stack_trace {
            event.properties.insert("$stack_trace".to_string(),
                serde_json::json!(stack));
        }

        // Include tags
        for (key, value) in &self.tags {
            event.properties.insert(format!("tag_{}", key), serde_json::json!(value));
        }

        event
    }
}

fn normalize_error_message(message: &str) -> String {
    // Remove numbers that might be IDs or timestamps
    let mut normalized = message.to_string();

    // Replace UUIDs
    let uuid_pattern = regex::Regex::new(
        r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}"
    ).unwrap();
    normalized = uuid_pattern.replace_all(&normalized, "<UUID>").to_string();

    // Replace numeric IDs
    let id_pattern = regex::Regex::new(r"\b\d{5,}\b").unwrap();
    normalized = id_pattern.replace_all(&normalized, "<ID>").to_string();

    // Replace quoted strings
    let quoted_pattern = regex::Regex::new(r#""[^"]*""#).unwrap();
    normalized = quoted_pattern.replace_all(&normalized, "<STRING>").to_string();

    normalized
}

/// Stack trace parser
pub struct StackTraceParser;

impl StackTraceParser {
    /// Parse JavaScript stack trace
    pub fn parse_javascript(raw: &str) -> StackTrace {
        let mut frames = Vec::new();

        for line in raw.lines() {
            if let Some(frame) = Self::parse_js_frame(line) {
                frames.push(frame);
            }
        }

        StackTrace {
            frames,
            raw: Some(raw.to_string()),
        }
    }

    fn parse_js_frame(line: &str) -> Option<StackFrame> {
        let line = line.trim();

        // Chrome/V8 format: "at functionName (filename:line:column)"
        // or "at filename:line:column"
        if line.starts_with("at ") {
            let rest = &line[3..];

            // Check for "at functionName (location)"
            if let Some(paren_start) = rest.find('(') {
                let function = rest[..paren_start].trim().to_string();
                let location = &rest[paren_start + 1..rest.len() - 1];

                return Self::parse_location(location).map(|(filename, lineno, colno)| {
                    StackFrame {
                        function: Some(function),
                        filename: Some(filename),
                        lineno: Some(lineno),
                        colno,
                        context: None,
                        in_app: Self::is_in_app(&filename),
                        module: None,
                    }
                });
            }

            // Just location
            return Self::parse_location(rest).map(|(filename, lineno, colno)| {
                StackFrame {
                    function: None,
                    filename: Some(filename),
                    lineno: Some(lineno),
                    colno,
                    context: None,
                    in_app: Self::is_in_app(&filename),
                    module: None,
                }
            });
        }

        // Firefox format: "functionName@filename:line:column"
        if let Some(at_pos) = line.find('@') {
            let function = line[..at_pos].to_string();
            let location = &line[at_pos + 1..];

            return Self::parse_location(location).map(|(filename, lineno, colno)| {
                StackFrame {
                    function: if function.is_empty() { None } else { Some(function) },
                    filename: Some(filename),
                    lineno: Some(lineno),
                    colno,
                    context: None,
                    in_app: Self::is_in_app(&filename),
                    module: None,
                }
            });
        }

        None
    }

    fn parse_location(location: &str) -> Option<(String, u32, Option<u32>)> {
        // Format: "filename:line:column" or "filename:line"
        let parts: Vec<&str> = location.rsplitn(3, ':').collect();

        match parts.len() {
            3 => {
                let filename = parts[2].to_string();
                let lineno = parts[1].parse().ok()?;
                let colno = parts[0].parse().ok();
                Some((filename, lineno, colno))
            }
            2 => {
                let filename = parts[1].to_string();
                let lineno = parts[0].parse().ok()?;
                Some((filename, lineno, None))
            }
            _ => None,
        }
    }

    fn is_in_app(filename: &str) -> bool {
        !filename.contains("node_modules") &&
        !filename.contains("vendor") &&
        !filename.starts_with("http://") &&
        !filename.starts_with("https://")
    }

    /// Parse Rust backtrace
    pub fn parse_rust(raw: &str) -> StackTrace {
        let mut frames = Vec::new();

        for line in raw.lines() {
            // Format: "   0: backtrace::backtrace::trace_unsynchronized"
            // or "   at /path/to/file.rs:123:45"
            let trimmed = line.trim();

            if trimmed.starts_with("at ") {
                // Location line
                if let Some(last_frame) = frames.last_mut() {
                    let location = &trimmed[3..];
                    if let Some((filename, lineno, colno)) = Self::parse_location(location) {
                        last_frame.filename = Some(filename.clone());
                        last_frame.lineno = Some(lineno);
                        last_frame.colno = colno;
                        last_frame.in_app = Self::is_rust_in_app(&filename);
                    }
                }
            } else if let Some(colon_pos) = trimmed.find(':') {
                // Function line
                let function = trimmed[colon_pos + 1..].trim().to_string();
                frames.push(StackFrame {
                    function: Some(function),
                    filename: None,
                    lineno: None,
                    colno: None,
                    context: None,
                    in_app: true,
                    module: None,
                });
            }
        }

        StackTrace {
            frames,
            raw: Some(raw.to_string()),
        }
    }

    fn is_rust_in_app(filename: &str) -> bool {
        !filename.contains(".cargo") &&
        !filename.contains("rustc") &&
        !filename.contains("/std/")
    }
}

/// Error grouping service
pub struct ErrorGrouper {
    /// Known error groups
    groups: std::sync::RwLock<HashMap<String, ErrorGroup>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorGroup {
    pub fingerprint: String,
    pub error_type: String,
    pub message: String,
    pub count: u64,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub affected_users: std::collections::HashSet<String>,
    pub status: ErrorGroupStatus,
    pub assigned_to: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorGroupStatus {
    New,
    Acknowledged,
    Resolved,
    Ignored,
    Regressed,
}

impl ErrorGrouper {
    pub fn new() -> Self {
        Self {
            groups: std::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Process an error event
    pub fn process(&self, error: &ErrorEvent, distinct_id: &str) -> ErrorGroup {
        let mut groups = self.groups.write().unwrap();

        let group = groups.entry(error.fingerprint.clone())
            .or_insert_with(|| ErrorGroup {
                fingerprint: error.fingerprint.clone(),
                error_type: error.error_type.clone(),
                message: error.message.clone(),
                count: 0,
                first_seen: error.timestamp,
                last_seen: error.timestamp,
                affected_users: std::collections::HashSet::new(),
                status: ErrorGroupStatus::New,
                assigned_to: None,
                tags: Vec::new(),
            });

        group.count += 1;
        group.last_seen = error.timestamp;
        group.affected_users.insert(distinct_id.to_string());

        // Check for regression
        if group.status == ErrorGroupStatus::Resolved {
            group.status = ErrorGroupStatus::Regressed;
        }

        group.clone()
    }

    /// Update group status
    pub fn update_status(&self, fingerprint: &str, status: ErrorGroupStatus) {
        let mut groups = self.groups.write().unwrap();
        if let Some(group) = groups.get_mut(fingerprint) {
            group.status = status;
        }
    }

    /// Get group by fingerprint
    pub fn get(&self, fingerprint: &str) -> Option<ErrorGroup> {
        let groups = self.groups.read().unwrap();
        groups.get(fingerprint).cloned()
    }

    /// Get all groups
    pub fn list(&self) -> Vec<ErrorGroup> {
        let groups = self.groups.read().unwrap();
        groups.values().cloned().collect()
    }
}

impl Default for ErrorGrouper {
    fn default() -> Self {
        Self::new()
    }
}

/// Error alerting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorAlertConfig {
    /// Enable alerts
    pub enabled: bool,
    /// Alert on new error groups
    pub alert_on_new: bool,
    /// Alert on regressions
    pub alert_on_regression: bool,
    /// Spike detection threshold (errors per minute)
    pub spike_threshold: u64,
    /// Alert channels
    pub channels: Vec<AlertChannel>,
    /// Error types to ignore
    pub ignore_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AlertChannel {
    Slack { webhook_url: String },
    Email { addresses: Vec<String> },
    Webhook { url: String },
    PagerDuty { routing_key: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_js_stack_parsing() {
        let stack = r#"Error: Something went wrong
    at doSomething (app.js:10:5)
    at handleClick (handlers.js:25:10)
    at HTMLButtonElement.<anonymous> (node_modules/react-dom/client.js:1234:5)"#;

        let parsed = StackTraceParser::parse_javascript(stack);

        assert_eq!(parsed.frames.len(), 3);
        assert_eq!(parsed.frames[0].function, Some("doSomething".to_string()));
        assert_eq!(parsed.frames[0].lineno, Some(10));
        assert!(parsed.frames[0].in_app);
        assert!(!parsed.frames[2].in_app); // node_modules
    }

    #[test]
    fn test_error_fingerprinting() {
        let mut error1 = ErrorEvent {
            error_type: "TypeError".to_string(),
            message: "Cannot read property 'foo' of undefined".to_string(),
            stack_trace: None,
            severity: ErrorSeverity::Error,
            fingerprint: String::new(),
            url: None,
            user_agent: None,
            browser: None,
            os: None,
            tags: HashMap::new(),
            context: HashMap::new(),
            breadcrumbs: Vec::new(),
            timestamp: Utc::now(),
            handled: false,
            source: ErrorSource::JavaScript,
        };

        let mut error2 = error1.clone();

        error1.generate_fingerprint();
        error2.generate_fingerprint();

        assert_eq!(error1.fingerprint, error2.fingerprint);
    }

    #[test]
    fn test_message_normalization() {
        let msg1 = "User 12345 not found";
        let msg2 = "User 67890 not found";

        assert_eq!(
            normalize_error_message(msg1),
            normalize_error_message(msg2)
        );
    }

    #[test]
    fn test_error_grouping() {
        let grouper = ErrorGrouper::new();

        let mut error = ErrorEvent {
            error_type: "TypeError".to_string(),
            message: "test error".to_string(),
            stack_trace: None,
            severity: ErrorSeverity::Error,
            fingerprint: "abc123".to_string(),
            url: None,
            user_agent: None,
            browser: None,
            os: None,
            tags: HashMap::new(),
            context: HashMap::new(),
            breadcrumbs: Vec::new(),
            timestamp: Utc::now(),
            handled: false,
            source: ErrorSource::JavaScript,
        };

        // First occurrence
        let group = grouper.process(&error, "user-1");
        assert_eq!(group.count, 1);
        assert_eq!(group.status, ErrorGroupStatus::New);

        // Second occurrence, different user
        let group = grouper.process(&error, "user-2");
        assert_eq!(group.count, 2);
        assert_eq!(group.affected_users.len(), 2);
    }
}
```

## TypeScript Client

```typescript
// Browser error tracking
class ErrorTracker {
  private breadcrumbs: Breadcrumb[] = [];
  private maxBreadcrumbs = 50;

  constructor() {
    this.setupGlobalHandlers();
  }

  private setupGlobalHandlers(): void {
    // Unhandled errors
    window.onerror = (message, source, lineno, colno, error) => {
      this.captureError(error || new Error(String(message)), {
        handled: false,
        source: 'javascript',
      });
    };

    // Unhandled promise rejections
    window.onunhandledrejection = (event) => {
      this.captureError(event.reason, {
        handled: false,
        source: 'javascript',
      });
    };

    // Console errors
    const originalError = console.error;
    console.error = (...args) => {
      this.addBreadcrumb({
        type: 'console',
        category: 'console.error',
        message: args.map(String).join(' '),
        level: 'error',
      });
      originalError.apply(console, args);
    };
  }

  captureError(error: Error, options: CaptureOptions = {}): void {
    const errorEvent = {
      $error_type: error.name,
      $error_message: error.message,
      $stack_trace: this.parseStackTrace(error.stack),
      $breadcrumbs: this.breadcrumbs.slice(-20),
      $handled: options.handled ?? true,
      $error_source: options.source || 'javascript',
    };

    analytics.capture('$error', errorEvent);
  }

  addBreadcrumb(crumb: Partial<Breadcrumb>): void {
    this.breadcrumbs.push({
      type: crumb.type || 'debug',
      category: crumb.category || 'custom',
      message: crumb.message,
      data: crumb.data || {},
      timestamp: new Date().toISOString(),
      level: crumb.level || 'info',
    });

    if (this.breadcrumbs.length > this.maxBreadcrumbs) {
      this.breadcrumbs.shift();
    }
  }

  private parseStackTrace(stack?: string): StackFrame[] {
    if (!stack) return [];

    return stack.split('\n').slice(1).map(line => {
      const match = line.match(/at (\S+) \((.+):(\d+):(\d+)\)/);
      if (match) {
        return {
          function: match[1],
          filename: match[2],
          lineno: parseInt(match[3]),
          colno: parseInt(match[4]),
        };
      }
      return null;
    }).filter(Boolean);
  }
}

// React Error Boundary integration
class AnalyticsErrorBoundary extends React.Component {
  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    errorTracker.captureError(error, {
      handled: true,
      source: 'react',
      extra: {
        componentStack: errorInfo.componentStack,
      },
    });
  }
}
```

## Related Specs

- 420-action-tracking.md - Breadcrumb sources
- 422-performance-tracking.md - Performance errors
- 429-analytics-webhooks.md - Error alerts
