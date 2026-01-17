//! Output generation for Forge sessions.

use std::collections::HashMap;
use std::path::Path;

use crate::{
    DecisionLog, DissentLog, ForgeResult, ForgeRound, ForgeSession, ForgeSessionStatus,
};

/// Configuration for output generation.
#[derive(Debug, Clone)]
pub struct OutputConfig {
    /// Include metadata section.
    pub include_metadata: bool,
    /// Include round history.
    pub include_history: bool,
    /// Include decision log.
    pub include_decisions: bool,
    /// Include dissent log.
    pub include_dissents: bool,
    /// Include convergence metrics.
    pub include_metrics: bool,
    /// Output format.
    pub format: OutputFormat,
    /// Custom template path.
    pub template: Option<String>,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            include_metadata: true,
            include_history: false,
            include_decisions: false,
            include_dissents: false,
            include_metrics: false,
            format: OutputFormat::Markdown,
            template: None,
        }
    }
}

/// Output format options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Markdown,
    Json,
    Yaml,
    Html,
    Plain,
}

/// Generator for session output.
pub struct OutputGenerator {
    config: OutputConfig,
}

impl OutputGenerator {
    /// Create a new output generator.
    pub fn new(config: OutputConfig) -> Self {
        Self { config }
    }

    /// Generate output from a session.
    pub fn generate(
        &self,
        session: &ForgeSession,
        decision_log: Option<&DecisionLog>,
        dissent_log: Option<&DissentLog>,
    ) -> ForgeResult<String> {
        match self.config.format {
            OutputFormat::Markdown => self.generate_markdown(session, decision_log, dissent_log),
            OutputFormat::Json => self.generate_json(session, decision_log, dissent_log),
            OutputFormat::Yaml => self.generate_yaml(session, decision_log, dissent_log),
            OutputFormat::Html => self.generate_html(session, decision_log, dissent_log),
            OutputFormat::Plain => self.generate_plain(session),
        }
    }

    /// Generate markdown output.
    fn generate_markdown(
        &self,
        session: &ForgeSession,
        decision_log: Option<&DecisionLog>,
        dissent_log: Option<&DissentLog>,
    ) -> ForgeResult<String> {
        let mut output = String::new();

        // Main content
        if let Some(content) = session.latest_draft() {
            output.push_str(content);
            output.push_str("\n\n");
        }

        // Metadata section
        if self.config.include_metadata {
            output.push_str(&self.generate_metadata_section(session));
        }

        // Round history
        if self.config.include_history {
            output.push_str(&self.generate_history_section(session));
        }

        // Metrics
        if self.config.include_metrics {
            output.push_str(&self.generate_metrics_section(session));
        }

        // Decision log
        if self.config.include_decisions {
            if let Some(log) = decision_log {
                output.push_str("\n---\n\n");
                output.push_str("## Appendix A: Decision Log\n\n");
                output.push_str(&log.to_markdown());
            }
        }

        // Dissent log
        if self.config.include_dissents {
            if let Some(log) = dissent_log {
                output.push_str("\n---\n\n");
                output.push_str("## Appendix B: Dissent Log\n\n");
                output.push_str(&log.to_markdown());
            }
        }

        Ok(output)
    }

    /// Generate metadata section.
    fn generate_metadata_section(&self, session: &ForgeSession) -> String {
        format!(
            r#"
---

## Forge Session Metadata

| Property | Value |
|----------|-------|
| Session ID | `{}` |
| Status | {:?} |
| Created | {} |
| Updated | {} |
| Rounds | {} |
| Total Cost | ${:.2} |
| Tokens Used | {} |

### Topic

**Title:** {}

**Description:** {}

{}

---

"#,
            session.id,
            session.status,
            session.created_at,
            session.updated_at,
            session.rounds.len(),
            session.total_cost_usd,
            session.total_tokens.total(),
            session.topic.title,
            session.topic.description,
            if session.topic.constraints.is_empty() {
                String::new()
            } else {
                format!("**Constraints:**\n- {}", session.topic.constraints.join("\n- "))
            }
        )
    }

    /// Generate round history section.
    fn generate_history_section(&self, session: &ForgeSession) -> String {
        let mut output = String::from("\n## Round History\n\n");

        for (i, round) in session.rounds.iter().enumerate() {
            output.push_str(&format!("### Round {} - ", i + 1));

            match round {
                ForgeRound::Draft(d) => {
                    output.push_str(&format!(
                        "Draft\n\n\
                         - **Drafter:** {}\n\
                         - **Duration:** {}ms\n\
                         - **Tokens:** {} in / {} out\n\n",
                        d.drafter.name,
                        d.duration_ms,
                        d.tokens.input,
                        d.tokens.output
                    ));
                }
                ForgeRound::Critique(c) => {
                    output.push_str("Critique\n\n");
                    output.push_str(&format!("**{} critiques received:**\n\n", c.critiques.len()));

                    for critique in &c.critiques {
                        output.push_str(&format!(
                            "- **{}:** Score {}/100\n\
                             - Strengths: {}\n\
                             - Weaknesses: {}\n\
                             - Suggestions: {}\n\n",
                            critique.critic.name,
                            critique.score,
                            critique.strengths.len(),
                            critique.weaknesses.len(),
                            critique.suggestions.len()
                        ));
                    }
                }
                ForgeRound::Synthesis(s) => {
                    output.push_str(&format!(
                        "Synthesis\n\n\
                         - **Synthesizer:** {}\n\
                         - **Conflicts Resolved:** {}\n\
                         - **Changes Made:** {}\n\
                         - **Duration:** {}ms\n\n",
                        s.synthesizer.name,
                        s.resolved_conflicts.len(),
                        s.changes.len(),
                        s.duration_ms
                    ));

                    if !s.resolved_conflicts.is_empty() {
                        output.push_str("**Conflicts Resolved:**\n");
                        for conflict in &s.resolved_conflicts {
                            output.push_str(&format!(
                                "- {}: {}\n",
                                conflict.issue,
                                conflict.resolution
                            ));
                        }
                        output.push('\n');
                    }
                }
                ForgeRound::Refinement(r) => {
                    output.push_str(&format!(
                        "Refinement\n\n\
                         - **Refiner:** {}\n\
                         - **Focus Area:** {}\n\
                         - **Depth:** {}\n\
                         - **Duration:** {}ms\n\n",
                        r.refiner.name,
                        r.focus_area,
                        r.depth,
                        r.duration_ms
                    ));
                }
                ForgeRound::Convergence(c) => {
                    output.push_str(&format!(
                        "Convergence Check\n\n\
                         - **Score:** {:.2}\n\
                         - **Converged:** {}\n\
                         - **Votes:** {} agree, {} disagree\n\n",
                        c.score,
                        c.converged,
                        c.votes.iter().filter(|v| v.agrees).count(),
                        c.votes.iter().filter(|v| !v.agrees).count()
                    ));

                    if !c.remaining_issues.is_empty() {
                        output.push_str("**Remaining Issues:**\n");
                        for issue in &c.remaining_issues {
                            output.push_str(&format!("- {}\n", issue));
                        }
                        output.push('\n');
                    }
                }
            }
        }

        output
    }

    /// Generate metrics section.
    fn generate_metrics_section(&self, session: &ForgeSession) -> String {
        let mut output = String::from("\n## Convergence Metrics\n\n");

        // Collect metrics from convergence rounds
        let convergence_rounds: Vec<_> = session.rounds.iter()
            .filter_map(|r| match r {
                ForgeRound::Convergence(c) => Some(c),
                _ => None,
            })
            .collect();

        if convergence_rounds.is_empty() {
            output.push_str("No convergence checks performed yet.\n");
            return output;
        }

        output.push_str("| Round | Score | Converged | Agreeing | Issues |\n");
        output.push_str("|-------|-------|-----------|----------|--------|\n");

        for (i, c) in convergence_rounds.iter().enumerate() {
            output.push_str(&format!(
                "| {} | {:.2} | {} | {}/{} | {} |\n",
                i + 1,
                c.score,
                if c.converged { "Yes" } else { "No" },
                c.votes.iter().filter(|v| v.agrees).count(),
                c.votes.len(),
                c.remaining_issues.len()
            ));
        }

        // Score trend
        if convergence_rounds.len() >= 2 {
            let first_score = convergence_rounds.first().unwrap().score;
            let last_score = convergence_rounds.last().unwrap().score;
            let trend = last_score - first_score;

            output.push_str(&format!(
                "\n**Score Trend:** {} ({:+.2})\n",
                if trend > 0.05 { "Improving" } else if trend < -0.05 { "Declining" } else { "Stable" },
                trend
            ));
        }

        output
    }

    /// Generate JSON output.
    fn generate_json(
        &self,
        session: &ForgeSession,
        decision_log: Option<&DecisionLog>,
        dissent_log: Option<&DissentLog>,
    ) -> ForgeResult<String> {
        #[derive(serde::Serialize)]
        struct JsonOutput<'a> {
            session: &'a ForgeSession,
            #[serde(skip_serializing_if = "Option::is_none")]
            decision_log: Option<&'a DecisionLog>,
            #[serde(skip_serializing_if = "Option::is_none")]
            dissent_log: Option<&'a DissentLog>,
        }

        let output = JsonOutput {
            session,
            decision_log: if self.config.include_decisions { decision_log } else { None },
            dissent_log: if self.config.include_dissents { dissent_log } else { None },
        };

        serde_json::to_string_pretty(&output)
            .map_err(|e| crate::ForgeError::Serialization(e.to_string()))
    }

    /// Generate YAML output.
    fn generate_yaml(
        &self,
        session: &ForgeSession,
        decision_log: Option<&DecisionLog>,
        dissent_log: Option<&DissentLog>,
    ) -> ForgeResult<String> {
        #[derive(serde::Serialize)]
        struct YamlOutput<'a> {
            session: &'a ForgeSession,
            #[serde(skip_serializing_if = "Option::is_none")]
            decision_log: Option<&'a DecisionLog>,
            #[serde(skip_serializing_if = "Option::is_none")]
            dissent_log: Option<&'a DissentLog>,
        }

        let output = YamlOutput {
            session,
            decision_log: if self.config.include_decisions { decision_log } else { None },
            dissent_log: if self.config.include_dissents { dissent_log } else { None },
        };

        serde_yaml::to_string(&output)
            .map_err(|e| crate::ForgeError::Serialization(e.to_string()))
    }

    /// Generate HTML output.
    fn generate_html(
        &self,
        session: &ForgeSession,
        decision_log: Option<&DecisionLog>,
        dissent_log: Option<&DissentLog>,
    ) -> ForgeResult<String> {
        // First generate markdown
        let markdown = self.generate_markdown(session, decision_log, dissent_log)?;

        // Convert to HTML
        let parser = pulldown_cmark::Parser::new(&markdown);
        let mut html_output = String::new();
        pulldown_cmark::html::push_html(&mut html_output, parser);

        // Wrap in HTML document
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{} - Forge Output</title>
    <style>
        body {{ font-family: system-ui, sans-serif; max-width: 900px; margin: 0 auto; padding: 2rem; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #f5f5f5; }}
        code {{ background-color: #f5f5f5; padding: 2px 4px; border-radius: 3px; }}
        pre {{ background-color: #f5f5f5; padding: 1rem; overflow-x: auto; }}
        blockquote {{ border-left: 3px solid #ddd; margin-left: 0; padding-left: 1rem; color: #666; }}
    </style>
</head>
<body>
{}
</body>
</html>"#,
            session.topic.title,
            html_output
        );

        Ok(html)
    }

    /// Generate plain text output.
    fn generate_plain(&self, session: &ForgeSession) -> ForgeResult<String> {
        Ok(session.latest_draft().unwrap_or_default().to_string())
    }
}

/// Builder for output configuration.
pub struct OutputConfigBuilder {
    config: OutputConfig,
}

impl OutputConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: OutputConfig::default(),
        }
    }

    pub fn with_metadata(mut self) -> Self {
        self.config.include_metadata = true;
        self
    }

    pub fn with_history(mut self) -> Self {
        self.config.include_history = true;
        self
    }

    pub fn with_decisions(mut self) -> Self {
        self.config.include_decisions = true;
        self
    }

    pub fn with_dissents(mut self) -> Self {
        self.config.include_dissents = true;
        self
    }

    pub fn with_metrics(mut self) -> Self {
        self.config.include_metrics = true;
        self
    }

    pub fn format(mut self, format: OutputFormat) -> Self {
        self.config.format = format;
        self
    }

    pub fn template(mut self, template: String) -> Self {
        self.config.template = Some(template);
        self
    }

    pub fn build(self) -> OutputConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ForgeSession, ForgeSessionConfig, ForgeTopic, ForgeSessionStatus};

    fn create_test_session() -> ForgeSession {
        let config = ForgeSessionConfig::default();
        let topic = ForgeTopic {
            title: "Test Topic".to_string(),
            description: "A test topic for output generation".to_string(),
            constraints: vec!["Must be concise".to_string()],
        };
        ForgeSession::new(config, topic)
    }

    #[test]
    fn test_output_config_builder() {
        let config = OutputConfigBuilder::new()
            .with_metadata()
            .with_history()
            .format(OutputFormat::Html)
            .build();

        assert!(config.include_metadata);
        assert!(config.include_history);
        assert_eq!(config.format, OutputFormat::Html);
    }

    #[test]
    fn test_plain_text_generation() {
        let session = create_test_session();
        let generator = OutputGenerator::new(OutputConfig {
            format: OutputFormat::Plain,
            ..OutputConfig::default()
        });

        let result = generator.generate(&session, None, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ""); // No draft yet
    }

    #[test]
    fn test_markdown_generation_with_metadata() {
        let session = create_test_session();
        let generator = OutputGenerator::new(OutputConfig {
            include_metadata: true,
            format: OutputFormat::Markdown,
            ..OutputConfig::default()
        });

        let result = generator.generate(&session, None, None);
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(output.contains("## Forge Session Metadata"));
        assert!(output.contains("Test Topic"));
        assert!(output.contains("Must be concise"));
    }

    #[test]
    fn test_json_generation() {
        let session = create_test_session();
        let generator = OutputGenerator::new(OutputConfig {
            format: OutputFormat::Json,
            ..OutputConfig::default()
        });

        let result = generator.generate(&session, None, None);
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(output.contains("\"session\""));
        // Verify it's valid JSON
        assert!(serde_json::from_str::<serde_json::Value>(&output).is_ok());
    }

    #[test]
    fn test_yaml_generation() {
        let session = create_test_session();
        let generator = OutputGenerator::new(OutputConfig {
            format: OutputFormat::Yaml,
            ..OutputConfig::default()
        });

        let result = generator.generate(&session, None, None);
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(output.contains("session:"));
    }

    #[test]
    fn test_html_generation() {
        let session = create_test_session();
        let generator = OutputGenerator::new(OutputConfig {
            include_metadata: true,
            format: OutputFormat::Html,
            ..OutputConfig::default()
        });

        let result = generator.generate(&session, None, None);
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(output.contains("<!DOCTYPE html>"));
        assert!(output.contains("<title>Test Topic - Forge Output</title>"));
        assert!(output.contains("</html>"));
    }
}