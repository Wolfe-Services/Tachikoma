# Spec 579: Consensus Summary Output

**Priority:** P0  
**Status:** planned  
**Depends on:** 577  
**Estimated Effort:** 2 hours  
**Target Files:**
- `crates/tachikoma-forge/src/output/summary.rs` (new)
- `crates/tachikoma-forge/src/output/mod.rs` (update)

---

## Overview

After deliberation converges, generate a human-readable consensus summary. This is what the user sees - a concise document explaining what was decided and why.

**Critical Rule**: The summary is for HUMANS only. It does NOT replace the structured task breakdown (see Spec 580).

---

## Acceptance Criteria

- [ ] Create `crates/tachikoma-forge/src/output/summary.rs`
- [ ] Define `ConsensusSummary` struct with: title, goal, decision, rationale, dissenting_views, next_steps
- [ ] Implement `generate_summary(session: &ForgeSession) -> ConsensusSummary`
- [ ] Add `to_markdown(&self) -> String` method that produces ~500 words max
- [ ] Include participant attribution: "The Architect proposed...", "The Critic noted..."
- [ ] Highlight any unresolved concerns in a "Dissent" section
- [ ] Export from `output/mod.rs`
- [ ] Verify `cargo check -p tachikoma-forge` passes

---

## Implementation

```rust
// crates/tachikoma-forge/src/output/summary.rs

use crate::ForgeSession;

#[derive(Debug, Clone)]
pub struct ConsensusSummary {
    pub title: String,
    pub goal: String,
    pub decision: String,
    pub rationale: String,
    pub key_points: Vec<String>,
    pub dissenting_views: Vec<DissentingView>,
    pub next_steps: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DissentingView {
    pub participant: String,
    pub concern: String,
}

impl ConsensusSummary {
    pub fn generate(session: &ForgeSession) -> Self {
        // Extract from convergence round
        let decision = session.rounds
            .iter()
            .filter(|r| r.round_type == "synthesis")
            .last()
            .map(|r| r.contributions.first())
            .flatten()
            .map(|c| c.content.clone())
            .unwrap_or_default();
        
        // Extract key points from critiques
        let key_points = session.rounds
            .iter()
            .filter(|r| r.round_type == "critique")
            .flat_map(|r| &r.contributions)
            .take(5)
            .map(|c| extract_main_point(&c.content))
            .collect();
        
        // Find dissent from convergence round
        let dissenting_views = session.rounds
            .iter()
            .filter(|r| r.round_type == "convergence")
            .flat_map(|r| &r.contributions)
            .filter(|c| c.content.to_lowercase().contains("disagree"))
            .map(|c| DissentingView {
                participant: c.participant_name.clone(),
                concern: extract_concern(&c.content),
            })
            .collect();
        
        Self {
            title: format!("Consensus: {}", session.name),
            goal: session.goal.clone(),
            decision,
            rationale: String::new(), // Extracted from synthesis
            key_points,
            dissenting_views,
            next_steps: Vec::new(), // Populated by beadifier
        }
    }
    
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        
        md.push_str(&format!("# {}\n\n", self.title));
        md.push_str(&format!("**Goal:** {}\n\n", self.goal));
        md.push_str("## Decision\n\n");
        md.push_str(&self.decision);
        md.push_str("\n\n");
        
        if !self.key_points.is_empty() {
            md.push_str("## Key Points\n\n");
            for point in &self.key_points {
                md.push_str(&format!("- {}\n", point));
            }
            md.push_str("\n");
        }
        
        if !self.dissenting_views.is_empty() {
            md.push_str("## Dissenting Views\n\n");
            for dissent in &self.dissenting_views {
                md.push_str(&format!("**{}:** {}\n\n", dissent.participant, dissent.concern));
            }
        }
        
        md
    }
}

fn extract_main_point(content: &str) -> String {
    // Take first sentence or first 100 chars
    content.split('.').next()
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| content.chars().take(100).collect())
}

fn extract_concern(content: &str) -> String {
    // Find the reasoning after "disagree"
    content.to_lowercase()
        .find("disagree")
        .map(|idx| content[idx..].chars().skip(8).take(200).collect())
        .unwrap_or_default()
}
```
