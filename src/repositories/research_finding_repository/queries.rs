use sqlx::{PgPool, query_as};
use uuid::Uuid;

use crate::{
    enums::research::ResearchFindingError,
    errors::app_error::AppError,
    models::research_finding::{ResearchFinding, ResearchFindingRevision},
};

const FINDING_FIELDS: &str = "SELECT finding.id, finding.project_id,
    creator.username AS created_by_username, author.username AS authored_by_username,
    CASE WHEN member.actor_id IS NULL THEN 'published' ELSE finding.finding_status END
        AS finding_status,
    revision.revision_number, finding.published_revision_number,
    revision.title, revision.statement, revision.details,
    COALESCE((
        SELECT jsonb_agg(jsonb_build_object(
            'sourceId', source.id, 'sourceType', source.source_type, 'title', source.title,
            'doi', source.doi, 'sourceUrl', source.source_url, 'authors', source.authors,
            'publicationYear', source.publication_year, 'provenance', source.provenance,
            'license', source.license, 'locator', citation.citation_locator,
            'context', citation.citation_context
        ) ORDER BY source.title)
        FROM research_finding_revision_sources citation
        JOIN research_sources source ON source.id = citation.source_id
        WHERE citation.finding_id = finding.id
            AND citation.revision_number = revision.revision_number
    ), '[]'::jsonb) AS citations,
    finding.created_at, finding.updated_at
    FROM research_findings finding
    JOIN research_projects project ON project.id = finding.project_id
        AND project.project_status = 'active'
    LEFT JOIN human_actor_identities member_identity ON member_identity.account_id = $2
    LEFT JOIN research_project_members member ON member.project_id = finding.project_id
        AND member.actor_id = member_identity.actor_id AND member.membership_status = 'active'
    JOIN research_finding_revisions revision ON revision.finding_id = finding.id
        AND revision.revision_number = CASE WHEN member.actor_id IS NULL
            THEN finding.published_revision_number ELSE finding.latest_revision_number END
    JOIN human_actor_identities creator_identity
        ON creator_identity.actor_id = finding.created_by_actor_id
    JOIN accounts creator ON creator.id = creator_identity.account_id
    JOIN human_actor_identities author_identity
        ON author_identity.actor_id = revision.authored_by_actor_id
    JOIN accounts author ON author.id = author_identity.account_id";

pub async fn find_research_finding(
    pool: &PgPool,
    finding_id: Uuid,
    account_id: Option<Uuid>,
) -> Result<ResearchFinding, AppError> {
    let query = format!(
        "{FINDING_FIELDS}
        WHERE finding.id = $1 AND finding.finding_status <> 'retracted'
            AND (member.actor_id IS NOT NULL OR
                (project.visibility = 'public' AND finding.published_revision_number IS NOT NULL))"
    );
    query_as::<_, ResearchFinding>(&query)
        .bind(finding_id)
        .bind(account_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| AppError::Internal)?
        .ok_or_else(|| ResearchFindingError::FindingNotFound.into())
}

pub async fn list_research_findings(
    pool: &PgPool,
    project_id: Uuid,
    account_id: Uuid,
    limit: i64,
) -> Result<Vec<ResearchFinding>, AppError> {
    let mut transaction = pool.begin().await.map_err(|_| AppError::Internal)?;
    let actor_id = super::access::find_active_human_actor(&mut transaction, account_id).await?;
    super::access::require_active_project(&mut transaction, project_id).await?;
    super::access::require_project_member(&mut transaction, project_id, actor_id).await?;
    let query = format!(
        "{FINDING_FIELDS}
        WHERE finding.project_id = $1 AND member.actor_id IS NOT NULL
            AND finding.finding_status <> 'retracted'
        ORDER BY finding.updated_at DESC, finding.id LIMIT $3"
    );
    let findings = query_as::<_, ResearchFinding>(&query)
        .bind(project_id)
        .bind(Some(account_id))
        .bind(limit)
        .fetch_all(&mut *transaction)
        .await
        .map_err(|_| AppError::Internal)?;
    transaction.commit().await.map_err(|_| AppError::Internal)?;
    Ok(findings)
}

pub async fn list_research_finding_revisions(
    pool: &PgPool,
    finding_id: Uuid,
    account_id: Uuid,
) -> Result<Vec<ResearchFindingRevision>, AppError> {
    let revisions = query_as::<_, ResearchFindingRevision>(
        "SELECT revision.finding_id, revision.project_id, revision.revision_number,
            revision.title, revision.statement, revision.details,
            author.username AS authored_by_username, revision.review_status,
            reviewer.username AS reviewed_by_username, revision.reviewer_notes,
            COALESCE((
                SELECT jsonb_agg(jsonb_build_object(
                    'sourceId', source.id, 'sourceType', source.source_type,
                    'title', source.title, 'doi', source.doi, 'sourceUrl', source.source_url,
                    'authors', source.authors, 'publicationYear', source.publication_year,
                    'provenance', source.provenance, 'license', source.license,
                    'locator', citation.citation_locator, 'context', citation.citation_context
                ) ORDER BY source.title)
                FROM research_finding_revision_sources citation
                JOIN research_sources source ON source.id = citation.source_id
                WHERE citation.finding_id = revision.finding_id
                    AND citation.revision_number = revision.revision_number
            ), '[]'::jsonb) AS citations,
            revision.created_at, revision.reviewed_at
        FROM research_finding_revisions revision
        JOIN research_findings finding ON finding.id = revision.finding_id
        JOIN research_projects project ON project.id = revision.project_id
            AND project.project_status = 'active'
        JOIN human_actor_identities member_identity ON member_identity.account_id = $2
        JOIN research_project_members member ON member.project_id = revision.project_id
            AND member.actor_id = member_identity.actor_id AND member.membership_status = 'active'
        JOIN human_actor_identities author_identity
            ON author_identity.actor_id = revision.authored_by_actor_id
        JOIN accounts author ON author.id = author_identity.account_id
        LEFT JOIN human_actor_identities reviewer_identity
            ON reviewer_identity.actor_id = revision.reviewed_by_actor_id
        LEFT JOIN accounts reviewer ON reviewer.id = reviewer_identity.account_id
        WHERE revision.finding_id = $1
        ORDER BY revision.revision_number DESC",
    )
    .bind(finding_id)
    .bind(account_id)
    .fetch_all(pool)
    .await
    .map_err(|_| AppError::Internal)?;
    if revisions.is_empty() {
        return Err(ResearchFindingError::FindingNotFound.into());
    }
    Ok(revisions)
}
