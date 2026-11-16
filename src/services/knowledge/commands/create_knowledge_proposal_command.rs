use std::collections::HashSet;

use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::knowledge,
    dtos::knowledge::{CreateKnowledgeProposalRequest, KnowledgeProposalCreatedResponse},
    enums::knowledge::KnowledgeEvidenceRelation,
    enums::knowledge_error::KnowledgeError,
    errors::app_error::AppError,
    models::knowledge::KnowledgeCitationInputRecord,
    repositories::knowledge_repository::proposals::{
        NewKnowledgeProposal, create_knowledge_proposal,
    },
};

pub async fn submit_knowledge_proposal(
    pool: &PgPool,
    contributor_account_id: Uuid,
    request: CreateKnowledgeProposalRequest,
) -> Result<KnowledgeProposalCreatedResponse, AppError> {
    let title = validate_required_text(
        &request.title,
        knowledge::KNOWLEDGE_TITLE_MIN_LENGTH,
        knowledge::KNOWLEDGE_TITLE_MAX_LENGTH,
    )?;
    let summary = validate_required_text(
        &request.summary,
        knowledge::KNOWLEDGE_SUMMARY_MIN_LENGTH,
        knowledge::KNOWLEDGE_SUMMARY_MAX_LENGTH,
    )?;
    let citations = validate_citations(request.citations)?;
    let research_findings = validate_finding_references(request.research_findings)?;
    let citation_records = citations
        .iter()
        .map(|citation| KnowledgeCitationInputRecord {
            title: &citation.title,
            doi: citation.doi.as_deref(),
            external_url: citation.external_url.as_deref(),
            authors: &citation.authors,
            publication_year: citation.publication_year,
            license: citation.license.as_deref(),
        })
        .collect::<Vec<_>>();
    let entry = create_knowledge_proposal(
        pool,
        contributor_account_id,
        NewKnowledgeProposal {
            entry_id: Uuid::new_v4(),
            academic_subject_id: request.academic_subject_id,
            title: &title,
            summary: &summary,
            citations: &citation_records,
            research_findings: &research_findings,
        },
    )
    .await?;
    Ok(entry.into())
}

fn validate_required_text(value: &str, minimum: usize, maximum: usize) -> Result<String, AppError> {
    let value = value.trim();
    if !(minimum..=maximum).contains(&value.chars().count()) {
        return Err(KnowledgeError::ProposalInvalid.into());
    }
    Ok(value.to_owned())
}

struct ValidatedCitation {
    title: String,
    doi: Option<String>,
    external_url: Option<String>,
    authors: Vec<String>,
    publication_year: Option<i16>,
    license: Option<String>,
}

fn validate_citations(
    citations: Vec<crate::dtos::knowledge::KnowledgeCitationInput>,
) -> Result<Vec<ValidatedCitation>, AppError> {
    if citations.is_empty() || citations.len() > knowledge::KNOWLEDGE_MAX_CITATIONS {
        return Err(KnowledgeError::CitationInvalid.into());
    }
    let mut citation_keys = HashSet::with_capacity(citations.len());
    citations
        .into_iter()
        .map(|citation| {
            let title = citation.title.trim();
            let doi = citation
                .doi
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty());
            let external_url = citation
                .external_url
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty());
            if title.is_empty()
                || title.chars().count() > knowledge::KNOWLEDGE_CITATION_TITLE_MAX_LENGTH
                || (doi.is_none() && external_url.is_none())
            {
                return Err(KnowledgeError::CitationInvalid.into());
            }
            if let Some(doi) = &doi
                && (!is_supported_doi(doi)
                    || doi.len() > knowledge::KNOWLEDGE_CITATION_DOI_MAX_LENGTH
                    || !citation_keys.insert(format!("doi:{}", doi.to_lowercase())))
            {
                return Err(KnowledgeError::CitationInvalid.into());
            }
            let external_url = if let Some(url) = &external_url {
                let normalized_url = validate_https_url(url)?;
                if normalized_url.len() > knowledge::KNOWLEDGE_CITATION_URL_MAX_LENGTH
                    || !citation_keys.insert(format!("url:{normalized_url}"))
                {
                    return Err(KnowledgeError::CitationInvalid.into());
                }
                Some(normalized_url)
            } else {
                None
            };
            let authors = validate_authors(citation.authors)?;
            let license =
                validate_optional_text(citation.license, knowledge::KNOWLEDGE_LICENSE_MAX_LENGTH)?;
            if citation
                .publication_year
                .is_some_and(|year| !(1000..=2200).contains(&year))
            {
                return Err(KnowledgeError::CitationInvalid.into());
            }
            Ok(ValidatedCitation {
                title: title.to_owned(),
                doi,
                external_url,
                authors,
                publication_year: citation.publication_year,
                license,
            })
        })
        .collect()
}

fn is_supported_doi(doi: &str) -> bool {
    doi.split_once('/').is_some_and(|(prefix, suffix)| {
        let has_registrant = prefix.strip_prefix("10.").is_some_and(|registrant| {
            !registrant.is_empty()
                && registrant
                    .chars()
                    .all(|character| character.is_ascii_digit())
        });
        has_registrant && !suffix.trim().is_empty()
    })
}

fn validate_https_url(value: &str) -> Result<String, AppError> {
    let url = url::Url::parse(value).map_err(|_| KnowledgeError::CitationInvalid)?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(KnowledgeError::CitationInvalid.into());
    }
    Ok(url.to_string())
}

fn validate_authors(authors: Vec<String>) -> Result<Vec<String>, AppError> {
    if authors.len() > knowledge::KNOWLEDGE_MAX_CITATION_AUTHORS {
        return Err(KnowledgeError::CitationInvalid.into());
    }
    authors
        .into_iter()
        .map(|author| {
            let author = author.trim();
            if author.is_empty()
                || author.chars().count() > knowledge::KNOWLEDGE_CITATION_AUTHOR_MAX_LENGTH
            {
                return Err(KnowledgeError::CitationInvalid.into());
            }
            Ok(author.to_owned())
        })
        .collect()
}

fn validate_optional_text(
    value: Option<String>,
    maximum: usize,
) -> Result<Option<String>, AppError> {
    value
        .map(|value| {
            let value = value.trim();
            if value.is_empty() || value.chars().count() > maximum {
                return Err(KnowledgeError::CitationInvalid.into());
            }
            Ok(value.to_owned())
        })
        .transpose()
}

fn validate_finding_references(
    findings: Vec<crate::dtos::knowledge::KnowledgeFindingInput>,
) -> Result<Vec<(Uuid, KnowledgeEvidenceRelation)>, AppError> {
    if findings.len() > knowledge::KNOWLEDGE_MAX_RESEARCH_FINDINGS {
        return Err(KnowledgeError::ProposalInvalid.into());
    }
    let mut finding_ids = HashSet::with_capacity(findings.len());
    findings
        .into_iter()
        .map(|finding| {
            if !finding_ids.insert(finding.finding_id) {
                return Err(KnowledgeError::ProposalInvalid.into());
            }
            Ok((finding.finding_id, finding.relation))
        })
        .collect()
}
