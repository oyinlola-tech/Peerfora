#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProfileError {
    BiographyTooLong,
    InvalidProfileVisibility,
    ProfileNotFound,
    DisplayNameTooLong,
    InvalidProfileReference,
    ProfileReferenceTooLong,
    InvalidPreferenceCountryCode,
    InvalidLanguageTag,
}
