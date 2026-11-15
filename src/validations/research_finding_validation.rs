use std::collections::HashSet;

use axum::http::Uri;
use uuid::Uuid;

use crate::{
    constants::research,
    dtos::research_finding::{CreateResearchSourceRequest, ResearchFindingCitationRequest},
    enums::research::{ResearchFindingError, ResearchSourceType},
};

pub struct ValidatedResearchSource {
    pub source_type: ResearchSourceType,
    pub title: String,
    pub doi: Option<String>,
    pub source_url: Option<String>,
    pub authors: Vec<String>,
    pub publication_year: Option<i16>,
    pub provenance: String,
    pub license: Option<String>,
}

pub struct ValidatedResearchFinding {
    pub title: String,
    pub statement: String,
    pub details: String,
    pub citations: Vec<ValidatedResearchCitation>,
}

pub struct ValidatedResearchCitation {
    pub source_id: Uuid,
    pub locator: Option<String>,
    pub context: Option<String>,
}

pub fn validate_research_source(
    request: CreateResearchSourceRequest,
) -> Result<ValidatedResearchSource, ResearchFindingError> {
    let title = validate_required_text(
        &request.title,
        research::MAX_RESEARCH_SOURCE_TITLE_LENGTH,
        ResearchFindingError::SourceTitleInvalid,
    )?;
    let doi = validate_optional_text(
        request.doi.as_deref(),
        research::MAX_RESEARCH_SOURCE_DOI_LENGTH,
        ResearchFindingError::SourceDoiInvalid,
    )?;
    let source_url = validate_optional_url(request.source_url.as_deref())?;
    if request.source_type == ResearchSourceType::Doi
        && !doi.as_deref().is_some_and(is_doi_shaped)
    {
        return Err(ResearchFindingError::SourceDoiInvalid);
    }
    if request.source_type == ResearchSourceType::ExternalUrl && source_url.is_none() {
        return Err(ResearchFindingError::SourceUrlInvalid);
    }
    let authors = validate_source_authors(&request.authors)?;
    let publication_year = request.publication_year;
    if publication_year.is_some_and(|year| !(1000..=9999).contains(&year)) {
        return Err(ResearchFindingError::SourceYearInvalid);
    }
    let provenance = validate_required_text(
        &request.provenance,
        research::MAX_RESEARCH_SOURCE_PROVENANCE_LENGTH,
        ResearchFindingError::SourceProvenanceInvalid,
    )?;
    let license = validate_optional_text(
        request.license.as_deref(),
        research::MAX_RESEARCH_SOURCE_LICENSE_LENGTH,
        ResearchFindingError::SourceLicenseInvalid,
    )?;

    Ok(ValidatedResearchSource {
        source_type: request.source_type,
        title,
        doi,
        source_url,
        authors,
        publication_year,
        provenance,
        license,
    })
}

pub fn validate_research_finding(
    title: &str,
    statement: &str,
    details: &str,
    citations: &[ResearchFindingCitationRequest],
) -> Result<ValidatedResearchFinding, ResearchFindingError> {
    let title = title.trim();
    if !(3..=research::MAX_RESEARCH_FINDING_TITLE_LENGTH).contains(&title.chars().count()) {
        return Err(ResearchFindingError::FindingTitleInvalid);
    }
    let statement = statement.trim();
    if statement.is_empty()
        || statement.chars().count() > research::MAX_RESEARCH_FINDING_STATEMENT_LENGTH
    {
        return Err(ResearchFindingError::FindingStatementInvalid);
    }
    let details = details.trim();
    if details.chars().count() > research::MAX_RESEARCH_FINDING_DETAILS_LENGTH {
        return Err(ResearchFindingError::FindingDetailsInvalid);
    }
    if citations.len() > research::MAX_RESEARCH_FINDING_CITATIONS {
        return Err(ResearchFindingError::TooManyCitations);
    }
    let mut unique_source_ids = HashSet::new();
    let citations = citations
        .iter()
        .map(|citation| {
            if !unique_source_ids.insert(citation.source_id) {
                return Err(ResearchFindingError::DuplicateCitation);
            }
            Ok(ValidatedResearchCitation {
                source_id: citation.source_id,
                locator: validate_optional_text(
                    citation.locator.as_deref(),
                    research::MAX_RESEARCH_CITATION_LOCATOR_LENGTH,
                    ResearchFindingError::CitationLocatorInvalid,
                )?,
                context: validate_optional_text(
                    citation.context.as_deref(),
                    research::MAX_RESEARCH_CITATION_CONTEXT_LENGTH,
                    ResearchFindingError::CitationContextInvalid,
                )?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ValidatedResearchFinding {
        title: title.to_owned(),
        statement: statement.to_owned(),
        details: details.to_owned(),
        citations,
    })
}

pub fn validate_research_finding_review_notes(
    notes: Option<&str>,
) -> Result<Option<String>, ResearchFindingError> {
    validate_optional_text(
        notes,
        research::MAX_RESEARCH_FINDING_REVIEW_NOTES_LENGTH,
        ResearchFindingError::ReviewNotesInvalid,
    )
}

fn validate_source_authors(authors: &[String]) -> Result<Vec<String>, ResearchFindingError> {
    if authors.len() > research::MAX_RESEARCH_SOURCE_AUTHORS {
        return Err(ResearchFindingError::SourceAuthorsInvalid);
    }
    authors
        .iter()
        .map(|author| {
            let author = author.trim();
            if !author.is_empty() && author.chars().count() <= 300 {
                Ok(author.to_owned())
            } else {
                Err(ResearchFindingError::SourceAuthorsInvalid)
            }
        })
        .collect()
}

fn validate_required_text(
    value: &str,
    maximum_length: usize,
    error: ResearchFindingError,
) -> Result<String, ResearchFindingError> {
    let value = value.trim();
    if !value.is_empty() && value.chars().count() <= maximum_length {
        Ok(value.to_owned())
    } else {
        Err(error)
    }
}

fn validate_optional_text(
    value: Option<&str>,
    maximum_length: usize,
    error: ResearchFindingError,
) -> Result<Option<String>, ResearchFindingError> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            if value.chars().count() <= maximum_length {
                Ok(value.to_owned())
            } else {
                Err(error)
            }
        })
        .transpose()
}

fn validate_optional_url(value: Option<&str>) -> Result<Option<String>, ResearchFindingError> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    let uri = value
        .parse::<Uri>()
        .map_err(|_| ResearchFindingError::SourceUrlInvalid)?;
    if value.chars().count() > research::MAX_RESEARCH_SOURCE_URL_LENGTH
        || uri.scheme_str() != Some("https")
        || uri.authority().is_none_or(|authority| {
            authority.host().is_empty() || authority.as_str().contains('@')
        })
    {
        return Err(ResearchFindingError::SourceUrlInvalid);
    }
    Ok(Some(value.to_owned()))
}

fn is_doi_shaped(doi: &str) -> bool {
    let doi = doi.trim();
    doi.starts_with("10.") && doi.contains('/') && !doi.chars().any(char::is_whitespace)
}
