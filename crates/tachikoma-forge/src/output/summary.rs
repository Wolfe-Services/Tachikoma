//! Consensus summary generation for forge sessions.

use crate::{ForgeSession, ForgeRound};

/// A human-readable summary of what was decided in a forge session.
#[derive(Debug, Clone)]
pub struct ConsensusSummary {
    pub title: String,
    pub goal: String,
    pub decision: String,
    pub rationale: String,
    pub dissenting_views: Vec<DissentingView>,
    pub next_steps: Vec<String>,
}

/// A dissenting view from a participant.
#[derive(Debug, Clone)]
pub struct DissentingView {
    pub participant: String,
    pub concern: String,
}

impl ConsensusSummary {
    /// Generate a consensus summary from a forge session.
    pub fn generate_summary(session: &ForgeSession) -> Self {
        // Extract the final decision from the latest synthesis or refinement round
        let decision = Self::extract_final_decision(session);
        
        // Extract rationale from synthesis rounds
        let rationale = Self::extract_rationale(session);
        
        // Find dissenting views from convergence rounds
        let dissenting_views = Self::extract_dissenting_views(session);
        
        // Generate next steps from the session context
        let next_steps = Self::generate_next_steps(session);
        
        Self {
            title: format!("Consensus: {}", session.topic.title),
            goal: session.topic.description.clone(),
            decision,
            rationale,
            dissenting_views,
            next_steps,
        }
    }
    
    /// Convert the summary to markdown format (~500 words max).
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        
        // Title and goal
        md.push_str(&format!("# {}\n\n", self.title));
        md.push_str(&format!("**Goal:** {}\n\n", self.goal));
        
        // Decision section
        md.push_str("## Decision\n\n");
        md.push_str(&self.decision);
        md.push_str("\n\n");
        
        // Rationale section
        if !self.rationale.is_empty() {
            md.push_str("## Rationale\n\n");
            md.push_str(&self.rationale);
            md.push_str("\n\n");
        }
        
        // Dissenting views section
        if !self.dissenting_views.is_empty() {
            md.push_str("## Dissenting Views\n\n");
            for dissent in &self.dissenting_views {
                md.push_str(&format!("**{}:** {}\n\n", dissent.participant, dissent.concern));
            }
        }
        
        // Next steps section
        if !self.next_steps.is_empty() {
            md.push_str("## Next Steps\n\n");
            for (i, step) in self.next_steps.iter().enumerate() {
                md.push_str(&format!("{}. {}\n", i + 1, step));
            }
            md.push_str("\n");
        }
        
        // Trim to approximately 500 words
        Self::trim_to_word_limit(&md, 500)
    }
    
    /// Extract the final decision from the session.
    fn extract_final_decision(session: &ForgeSession) -> String {
        // Look for the most recent synthesis or refinement round
        session.rounds
            .iter()
            .rev()
            .find_map(|round| match round {
                ForgeRound::Synthesis(synthesis) => Some(synthesis.content.clone()),
                ForgeRound::Refinement(refinement) => Some(refinement.content.clone()),
                ForgeRound::Draft(draft) => Some(draft.content.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "No final decision reached.".to_string())
    }
    
    /// Extract rationale from synthesis rounds.
    fn extract_rationale(session: &ForgeSession) -> String {
        let rationales: Vec<String> = session.rounds
            .iter()
            .filter_map(|round| match round {
                ForgeRound::Synthesis(synthesis) => {
                    if !synthesis.reasoning.is_empty() {
                        Some(format!("The Architect reasoned: {}", synthesis.reasoning))
                    } else {
                        None
                    }
                },
                ForgeRound::Refinement(refinement) => {
                    Some(format!("The Refiner noted: Focus on {} with depth {}.", 
                               refinement.focus_area, refinement.depth))
                },
                _ => None,
            })
            .collect();
        
        rationales.join(" ")
    }
    
    /// Extract dissenting views from convergence rounds.
    fn extract_dissenting_views(session: &ForgeSession) -> Vec<DissentingView> {
        session.rounds
            .iter()
            .filter_map(|round| match round {
                ForgeRound::Convergence(conv) => Some(conv),
                _ => None,
            })
            .flat_map(|conv| &conv.votes)
            .filter(|vote| !vote.agrees)
            .map(|vote| DissentingView {
                participant: format!("The {}", vote.participant.name),
                concern: vote.reasoning.clone(),
            })
            .collect()
    }
    
    /// Generate next steps based on session context.
    fn generate_next_steps(session: &ForgeSession) -> Vec<String> {
        let mut steps = Vec::new();
        
        // Check if there are remaining issues from convergence
        for round in &session.rounds {
            if let ForgeRound::Convergence(conv) = round {
                for issue in &conv.remaining_issues {
                    steps.push(format!("Address remaining concern: {}", issue));
                }
            }
        }
        
        // Add standard next steps if no specific issues
        if steps.is_empty() {
            steps.push("Implement the agreed solution".to_string());
            steps.push("Monitor for any implementation challenges".to_string());
        }
        
        steps
    }
    
    /// Trim markdown content to approximately the specified word limit.
    fn trim_to_word_limit(content: &str, word_limit: usize) -> String {
        let words: Vec<&str> = content.split_whitespace().collect();
        if words.len() <= word_limit {
            content.to_string()
        } else {
            let trimmed_words = &words[..word_limit];
            let mut result = trimmed_words.join(" ");
            result.push_str("...\n\n*[Summary truncated to stay within word limit]*");
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ForgeSessionConfig, ForgeTopic, Participant};
    use chrono::Utc;

    #[test]
    fn test_generate_summary_basic() {
        let config = ForgeSessionConfig::default();
        let topic = ForgeTopic {
            title: "Test Topic".to_string(),
            description: "A test description".to_string(),
            constraints: vec![],
        };
        let session = ForgeSession::new(config, topic);
        
        let summary = ConsensusSummary::generate_summary(&session);
        
        assert_eq!(summary.title, "Consensus: Test Topic");
        assert_eq!(summary.goal, "A test description");
    }
    
    #[test]
    fn test_to_markdown_format() {
        let summary = ConsensusSummary {
            title: "Test Summary".to_string(),
            goal: "Test goal".to_string(),
            decision: "Test decision".to_string(),
            rationale: "Test rationale".to_string(),
            dissenting_views: vec![DissentingView {
                participant: "The Critic".to_string(),
                concern: "Test concern".to_string(),
            }],
            next_steps: vec!["Step 1".to_string(), "Step 2".to_string()],
        };
        
        let markdown = summary.to_markdown();
        
        assert!(markdown.contains("# Test Summary"));
        assert!(markdown.contains("**Goal:** Test goal"));
        assert!(markdown.contains("## Decision"));
        assert!(markdown.contains("## Rationale"));
        assert!(markdown.contains("## Dissenting Views"));
        assert!(markdown.contains("**The Critic:** Test concern"));
        assert!(markdown.contains("## Next Steps"));
        assert!(markdown.contains("1. Step 1"));
        assert!(markdown.contains("2. Step 2"));
    }
    
    #[test]
    fn test_word_limit_trimming() {
        let long_content = "word ".repeat(600);
        let trimmed = ConsensusSummary::trim_to_word_limit(&long_content, 500);
        
        let word_count = trimmed.split_whitespace().count();
        // Should be approximately 500 words plus the truncation message
        assert!(word_count < 520);
        assert!(trimmed.contains("[Summary truncated"));
    }
}