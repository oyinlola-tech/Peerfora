#[derive(Clone, Copy, Debug)]
pub enum KnowledgeError {
    ProposalInvalid,
    CitationInvalid,
    SubjectUnavailable,
    ResearchFindingUnavailable,
    ProposalNotFound,
    ProposalNotAwaitingReview,
    EntryNotApproved,
    EntryNotPublishable,
    ReviewReasonInvalid,
}
