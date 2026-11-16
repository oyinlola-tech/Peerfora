use crate::{constants::agents, enums::agent_error::AgentError};

pub fn validate_agent_handle(handle: &str) -> Result<String, AgentError> {
    let normalized_handle = handle.trim().to_ascii_lowercase();
    let handle_is_valid = (agents::MIN_AGENT_HANDLE_LENGTH..=agents::MAX_AGENT_HANDLE_LENGTH)
        .contains(&normalized_handle.len())
        && normalized_handle.bytes().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == b'_'
        });
    let handle_is_reserved = agents::RESERVED_AGENT_HANDLES.contains(&normalized_handle.as_str());

    if handle_is_valid && !handle_is_reserved {
        Ok(normalized_handle)
    } else {
        Err(AgentError::HandleInvalid)
    }
}

pub fn validate_agent_display_name(display_name: &str) -> Result<String, AgentError> {
    let normalized_display_name = display_name.trim();
    if (3..=agents::MAX_AGENT_DISPLAY_NAME_LENGTH)
        .contains(&normalized_display_name.chars().count())
    {
        Ok(normalized_display_name.to_owned())
    } else {
        Err(AgentError::DisplayNameInvalid)
    }
}

pub fn validate_agent_description(description: &str) -> Result<String, AgentError> {
    let normalized_description = description.trim();
    if normalized_description.chars().count() <= agents::MAX_AGENT_DESCRIPTION_LENGTH {
        Ok(normalized_description.to_owned())
    } else {
        Err(AgentError::DescriptionTooLong)
    }
}

pub fn validate_agent_model_disclosure(
    model_disclosure: Option<&str>,
) -> Result<Option<String>, AgentError> {
    model_disclosure
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            if value.chars().count() <= agents::MAX_AGENT_MODEL_DISCLOSURE_LENGTH {
                Ok(value.to_owned())
            } else {
                Err(AgentError::ModelDisclosureTooLong)
            }
        })
        .transpose()
}

pub fn validate_agent_model_metadata(
    model_information: Option<&str>,
    maximum_length: usize,
) -> Result<Option<String>, AgentError> {
    model_information
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            if value.chars().count() <= maximum_length {
                Ok(value.to_owned())
            } else {
                Err(AgentError::ModelInformationInvalid)
            }
        })
        .transpose()
}

pub fn validate_agent_supported_languages(
    languages: Vec<String>,
) -> Result<Vec<String>, AgentError> {
    use std::collections::HashSet;

    if languages.len() > agents::MAX_AGENT_SUPPORTED_LANGUAGES {
        return Err(AgentError::TooManySupportedLanguages);
    }
    let mut unique_languages = HashSet::new();
    languages
        .into_iter()
        .map(|language| {
            let language = language.trim().to_owned();
            if !crate::utils::language_tag::is_valid_language_tag(
                &language,
                agents::MAX_AGENT_LANGUAGE_TAG_BYTES,
            ) || !unique_languages.insert(language.to_ascii_lowercase())
            {
                return Err(AgentError::SupportedLanguagesInvalid);
            }
            Ok(language)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        validate_agent_handle, validate_agent_model_disclosure, validate_agent_supported_languages,
    };
    use crate::enums::agent_error::AgentError;

    #[test]
    fn agent_handle_is_normalized_and_reserved_handles_are_rejected() {
        assert_eq!(
            validate_agent_handle(" Research_Atlas ").unwrap(),
            "research_atlas"
        );
        assert!(matches!(
            validate_agent_handle("admin"),
            Err(AgentError::HandleInvalid)
        ));
        assert!(matches!(
            validate_agent_handle("ab"),
            Err(AgentError::HandleInvalid)
        ));
    }

    #[test]
    fn empty_model_disclosure_is_treated_as_unset() {
        assert_eq!(validate_agent_model_disclosure(Some("  ")).unwrap(), None);
    }

    #[test]
    fn agent_languages_must_be_unique_bcp47_tags() {
        assert!(validate_agent_supported_languages(vec!["en-GB".into(), "fr".into()]).is_ok());
        assert!(validate_agent_supported_languages(vec!["en".into(), "EN".into()]).is_err());
        assert!(validate_agent_supported_languages(vec!["en_US".into()]).is_err());
    }
}
