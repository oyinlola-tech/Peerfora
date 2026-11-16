use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeEvidenceRelation {
    Supports,
    Contradicts,
    Contextualizes,
}

impl KnowledgeEvidenceRelation {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::Supports => "supports",
            Self::Contradicts => "contradicts",
            Self::Contextualizes => "contextualizes",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeReviewDecision {
    Approve,
    Reject,
}

impl KnowledgeReviewDecision {
    pub fn audit_action(self) -> &'static str {
        match self {
            Self::Approve => crate::constants::knowledge::KNOWLEDGE_PROPOSAL_APPROVED_AUDIT_ACTION,
            Self::Reject => crate::constants::knowledge::KNOWLEDGE_PROPOSAL_REJECTED_AUDIT_ACTION,
        }
    }

    pub fn next_entry_status(self) -> &'static str {
        match self {
            Self::Approve => "approved",
            Self::Reject => "rejected",
        }
    }
}
