use crate::{ForgeSession, Participant};
use super::orchestrator::RoundType;

pub fn build_prompt(
    round_type: RoundType,
    goal: &str,
    participant: &Participant,
    session: &ForgeSession,
) -> String {
    match round_type {
        RoundType::Draft => format!(
            "You are {}.\n\n\
            Goal: {}\n\n\
            Propose a solution or approach to this goal. Be specific and actionable.\n\n\
            Your role: {}",
            participant.name,
            goal,
            participant.model_config.model_name
        ),
        
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
            format!(
                "You are {}.\n\n\
                Based on all proposals and critiques so far, synthesize the best elements \
                into a unified solution.\n\n\
                Goal: {}\n\n\
                Create a cohesive approach that addresses the critiques.",
                participant.name,
                goal
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