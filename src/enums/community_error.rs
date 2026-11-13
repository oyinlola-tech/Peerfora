#[derive(Clone, Copy, Debug)]
pub enum CommunityError {
    NameInvalid,
    SlugInvalid,
    LanguageTagInvalid,
    DescriptionTooLong,
    InvalidCategoryReferences,
    DuplicateSubject,
    SubjectListTooLarge,
    SubjectUnavailable,
    InstitutionUnavailable,
    CourseUnavailable,
    CommunityNotFound,
    CommunitySlugAlreadyUsed,
    MembershipNotAllowed,
    MembershipBanned,
    OwnerCannotLeave,
}
