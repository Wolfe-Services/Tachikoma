use crate::{ForgeSession, Participant};
use super::orchestrator::RoundType;

pub fn build_prompt(
    round_type: RoundType,
    goal: &str,
    participant: &Participant,
    session: &ForgeSession,
) -> String {
    match round_type {
        RoundType::Draft => {
            let prior_drafts = session
                .rounds
                .iter()
                .filter_map(|r| match r {
                    crate::round::ForgeRound::Draft(draft) => Some(format!(
                        "**{}**:\n{}",
                        draft.drafter.name, draft.content
                    )),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n\n---\n\n");

            format!(
                "You are {}.\n\n\
                Goal: {}\n\n\
                {}\
                Write your proposal. If a transcript is provided, explicitly respond to it:\n\
                - agree/disagree with specific points\n\
                - build on good ideas\n\
                - call out gaps\n\n\
                Your role: {}",
                participant.name,
                goal,
                if prior_drafts.is_empty() {
                    "".to_string()
                } else {
                    format!("Prior drafts so far:\n\n{}\n\n", prior_drafts)
                },
                participant.model_config.model_name
            )
        }
        
        RoundType::Critique => {
            let drafts = session.rounds
                .iter()
                .filter_map(|r| match r {
                    crate::round::ForgeRound::Draft(draft) => Some(format!("**{}**: {}", draft.drafter.name, draft.content)),
                    crate::round::ForgeRound::Synthesis(synthesis) => Some(format!("**{}**: {}", synthesis.synthesizer.name, synthesis.content)),
                    crate::round::ForgeRound::Refinement(refinement) => Some(format!("**{}**: {}", refinement.refiner.name, refinement.content)),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n\n---\n\n");
            
            format!(
                "You are {}.\n\n\
                Review these proposals and provide constructive critique:\n\n{}\n\n\
                Identify strengths, weaknesses, gaps, and potential improvements.",
                participant.name,
                drafts
            )
        }
        
        RoundType::Synthesis => {
            let drafts = session.rounds
                .iter()
                .filter_map(|r| match r {
                    crate::round::ForgeRound::Draft(draft) => Some(format!("**{}**: {}", draft.drafter.name, draft.content)),
                    crate::round::ForgeRound::Synthesis(synthesis) => Some(format!("**{}**: {}", synthesis.synthesizer.name, synthesis.content)),
                    crate::round::ForgeRound::Refinement(refinement) => Some(format!("**{}**: {}", refinement.refiner.name, refinement.content)),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n\n---\n\n");

            let critiques = session.rounds
                .iter()
                .filter_map(|r| match r {
                    crate::round::ForgeRound::Critique(c) => Some(
                        c.critiques
                            .iter()
                            .map(|crit| format!("**{}**: {}", crit.critic.name, crit.raw_content))
                            .collect::<Vec<_>>()
                            .join("\n\n---\n\n"),
                    ),
                    _ => None,
                })
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join("\n\n---\n\n");

            format!(
                "You are {}.\n\n\
                Based on all proposals and critiques so far, synthesize the best elements \
                into a unified solution.\n\n\
                Goal: {}\n\n\
                Proposals:\n\n{}\n\n\
                Critiques:\n\n{}\n\n\
                Create a cohesive approach that addresses the critiques.",
                participant.name,
                goal,
                if drafts.is_empty() { "_No proposals yet._".to_string() } else { drafts },
                if critiques.is_empty() { "_No critiques yet._".to_string() } else { critiques },
            )
        }
        
        RoundType::Convergence => {
            format!(
                "You are {}.\n\n\
                Review the synthesized solution.\n\n\
                Vote: Do you AGREE or DISAGREE that this adequately addresses the goal?\n\n\
                Provide your reasoning. If you disagree, specify what's missing.",
                participant.name
            )
        }
    }
}