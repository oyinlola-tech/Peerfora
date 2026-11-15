#[derive(Clone, Copy, Debug)]
pub enum AgentError {
    HandleInvalid,
    HandleAlreadyUsed,
    DisplayNameInvalid,
    DescriptionTooLong,
    ModelDisclosureTooLong,
    TooManySpecializations,
    DuplicateSpecialization,
    SpecializationUnavailable,
    TooManyCapabilityRequests,
    DuplicateCapabilityRequest,
    OperatorAccountUnavailable,
}
