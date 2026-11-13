use std::collections::HashSet;

use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    enums::{
        community::{CommunityCategory, CommunityMembershipPolicy, CommunityVisibility},
        community_error::CommunityError,
    },
    errors::app_error::AppError,
    repositories::community_repository::{self, NewCommunity},
};

pub struct CreateCommunityDetails {
    pub slug: Option<String>,
    pub display_name: String,
    pub description: String,
    pub category: CommunityCategory,
    pub visibility: CommunityVisibility,
    pub membership_policy: CommunityMembershipPolicy,
    pub primary_language_tag: String,
    pub institution_id: Option<Uuid>,
    pub course_id: Option<Uuid>,
    pub discipline_id: Option<Uuid>,
    pub subject_ids: Vec<Uuid>,
}

pub async fn create_community(
    pool: &PgPool,
    creator_account_id: Uuid,
    details: CreateCommunityDetails,
) -> Result<crate::models::community::Community, AppError> {
    let display_name = details.display_name.trim();
    if !(3..=80).contains(&display_name.chars().count()) {
        return Err(CommunityError::NameInvalid.into());
    }
    if details.description.chars().count() > 2000 {
        return Err(CommunityError::DescriptionTooLong.into());
    }
    if details.subject_ids.len() > 20 {
        return Err(CommunityError::SubjectListTooLarge.into());
    }
    if details.subject_ids.iter().collect::<HashSet<_>>().len() != details.subject_ids.len() {
        return Err(CommunityError::DuplicateSubject.into());
    }
    validate_category_references(&details)?;
    validate_language_tag(&details.primary_language_tag)?;

    let slug = normalize_community_slug(details.slug.as_deref(), display_name)?;
    let membership_policy = if matches!(details.visibility, CommunityVisibility::Private) {
        CommunityMembershipPolicy::ApprovalRequired
    } else {
        details.membership_policy
    };
    let description = details.description.trim();

    community_repository::create_community(
        pool,
        NewCommunity {
            slug: &slug,
            display_name,
            description,
            category: details.category.as_database_value(),
            visibility: details.visibility.as_database_value(),
            membership_policy: membership_policy.as_database_value(),
            primary_language_tag: &details.primary_language_tag,
            institution_id: details.institution_id,
            course_id: details.course_id,
            discipline_id: details.discipline_id,
            subject_ids: &details.subject_ids,
            creator_account_id,
        },
    )
    .await
}

fn validate_category_references(details: &CreateCommunityDetails) -> Result<(), AppError> {
    let category_has_required_reference = match details.category {
        CommunityCategory::AcademicSubject => !details.subject_ids.is_empty(),
        CommunityCategory::Institution => details.institution_id.is_some(),
        CommunityCategory::InstitutionCourse => details.course_id.is_some(),
        CommunityCategory::AcademicDiscipline => details.discipline_id.is_some(),
        _ => true,
    };
    if !category_has_required_reference {
        return Err(CommunityError::InvalidCategoryReferences.into());
    }
    if details.course_id.is_some() && details.institution_id.is_none() {
        return Err(CommunityError::InvalidCategoryReferences.into());
    }
    Ok(())
}

fn validate_language_tag(language_tag: &str) -> Result<(), AppError> {
    if !crate::utils::language_tag::is_valid_language_tag(language_tag, 35) {
        return Err(CommunityError::LanguageTagInvalid.into());
    }
    Ok(())
}

fn normalize_community_slug(
    requested_slug: Option<&str>,
    display_name: &str,
) -> Result<String, AppError> {
    let source = requested_slug
        .unwrap_or(display_name)
        .trim()
        .to_ascii_lowercase();
    let mut slug = String::new();
    let mut previous_was_separator = true;
    for character in source.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            previous_was_separator = false;
        } else if matches!(character, ' ' | '-' | '_') {
            if !previous_was_separator {
                slug.push('-');
                previous_was_separator = true;
            }
        } else if !character.is_ascii() && requested_slug.is_some() {
            return Err(CommunityError::SlugInvalid.into());
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }

    let is_valid = (3..=50).contains(&slug.len())
        && !matches!(
            slug.as_str(),
            "admin" | "api" | "help" | "login" | "peerfora" | "signup" | "support"
        );
    if !is_valid {
        return Err(CommunityError::SlugInvalid.into());
    }
    Ok(slug)
}

#[cfg(test)]
mod tests {
    use super::normalize_community_slug;

    #[test]
    fn generated_slug_normalizes_spacing_and_case() {
        assert_eq!(
            normalize_community_slug(None, "  Data   Structures ").unwrap(),
            "data-structures"
        );
    }

    #[test]
    fn reserved_slug_is_rejected() {
        assert!(normalize_community_slug(Some("admin"), "Administrators").is_err());
    }
}
