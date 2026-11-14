CREATE TABLE countries (
    country_code CHAR(2) PRIMARY KEY CHECK (country_code ~ '^[A-Z]{2}$'),
    alpha_3 CHAR(3) NOT NULL UNIQUE CHECK (alpha_3 ~ '^[A-Z]{3}$'),
    display_name TEXT NOT NULL CHECK (length(btrim(display_name)) > 0),
    official_name TEXT,
    source TEXT NOT NULL,
    source_version TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE country_name_aliases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    country_code CHAR(2) NOT NULL REFERENCES countries(country_code) ON DELETE RESTRICT,
    display_name TEXT NOT NULL CHECK (length(btrim(display_name)) > 0),
    language_tag TEXT NOT NULL DEFAULT 'und',
    source TEXT NOT NULL,
    source_version TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (country_code, display_name, language_tag)
);
CREATE INDEX country_name_aliases_name_idx ON country_name_aliases (lower(display_name));

ALTER TABLE accounts
    ADD CONSTRAINT accounts_country_code_fk
    FOREIGN KEY (eligibility_country_code) REFERENCES countries(country_code) NOT VALID,
    ADD COLUMN account_status TEXT NOT NULL DEFAULT 'active'
        CHECK (account_status IN ('active', 'suspended', 'pending_deletion', 'deleted'));
ALTER TABLE accounts DROP CONSTRAINT accounts_email_key;
CREATE UNIQUE INDEX accounts_email_lower_unique ON accounts (lower(email));
CREATE FUNCTION maintain_updated_at()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$;
CREATE TRIGGER accounts_maintain_updated_at BEFORE UPDATE ON accounts
FOR EACH ROW EXECUTE FUNCTION maintain_updated_at();

CREATE TABLE academic_levels (
    id TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    source TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    display_order SMALLINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE academic_level_aliases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    academic_level_id TEXT NOT NULL REFERENCES academic_levels(id) ON DELETE CASCADE,
    alias TEXT NOT NULL CHECK (length(btrim(alias)) > 0),
    language_tag TEXT NOT NULL DEFAULT 'und',
    source TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (academic_level_id, alias, language_tag)
);
CREATE UNIQUE INDEX academic_level_aliases_alias_locale_unique
    ON academic_level_aliases (lower(alias), language_tag);
CREATE TABLE academic_disciplines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    description TEXT,
    source TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE academic_subjects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    description TEXT,
    discipline_id UUID REFERENCES academic_disciplines(id) ON DELETE SET NULL,
    source TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX academic_subjects_discipline_idx ON academic_subjects (discipline_id)
    WHERE discipline_id IS NOT NULL;
CREATE TABLE academic_subject_aliases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_id UUID NOT NULL REFERENCES academic_subjects(id) ON DELETE CASCADE,
    alias TEXT NOT NULL CHECK (length(btrim(alias)) > 0),
    language_tag TEXT NOT NULL DEFAULT 'und',
    source TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (subject_id, alias, language_tag)
);
CREATE INDEX academic_subject_aliases_alias_lower_idx ON academic_subject_aliases (lower(alias));
CREATE TABLE academic_qualification_types (
    id TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    academic_level_id TEXT NOT NULL REFERENCES academic_levels(id) ON DELETE RESTRICT,
    source TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE institution_types (
    id TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    source TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE institutions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    identifier_namespace TEXT,
    official_identifier TEXT,
    country_code CHAR(2) REFERENCES countries(country_code) ON DELETE RESTRICT,
    region_name TEXT,
    institution_type TEXT NOT NULL REFERENCES institution_types(id) ON DELETE RESTRICT,
    official_website TEXT,
    verification_status TEXT NOT NULL DEFAULT 'unverified'
        CHECK (verification_status IN ('unverified', 'pending', 'verified', 'rejected', 'inactive')),
    record_source TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((official_identifier IS NULL) = (identifier_namespace IS NULL))
);
CREATE UNIQUE INDEX institutions_namespace_identifier_unique
    ON institutions (COALESCE(country_code::text, ''), identifier_namespace, official_identifier)
    WHERE official_identifier IS NOT NULL;
CREATE INDEX institutions_country_type_idx ON institutions (country_code, institution_type)
    WHERE is_active;
CREATE TABLE institution_aliases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    institution_id UUID NOT NULL REFERENCES institutions(id) ON DELETE CASCADE,
    alias TEXT NOT NULL CHECK (length(btrim(alias)) > 0),
    record_source TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (institution_id, alias)
);
CREATE INDEX institution_aliases_alias_lower_idx ON institution_aliases (lower(alias));
CREATE TABLE directory_suggestions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    suggestion_type TEXT NOT NULL CHECK (suggestion_type IN (
        'academic_level', 'discipline', 'subject', 'qualification_type',
        'institution', 'institution_type', 'other'
    )),
    proposed_name TEXT NOT NULL CHECK (length(btrim(proposed_name)) BETWEEN 2 AND 120),
    parent_name TEXT,
    country_code CHAR(2) REFERENCES countries(country_code) ON DELETE RESTRICT,
    official_website TEXT,
    details TEXT CHECK (details IS NULL OR length(details) <= 2000),
    submitted_by UUID NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    review_status TEXT NOT NULL DEFAULT 'pending'
        CHECK (review_status IN ('pending', 'accepted', 'rejected', 'duplicate')),
    reviewed_by UUID REFERENCES accounts(id) ON DELETE RESTRICT,
    reviewed_at TIMESTAMPTZ,
    review_note TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (suggestion_type <> 'institution' OR country_code IS NOT NULL),
    CHECK ((review_status = 'pending' AND reviewed_by IS NULL AND reviewed_at IS NULL)
        OR (review_status <> 'pending' AND reviewed_by IS NOT NULL AND reviewed_at IS NOT NULL))
);
CREATE INDEX directory_suggestions_pending_created_idx
    ON directory_suggestions (created_at DESC) WHERE review_status = 'pending';
CREATE TABLE institution_courses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    institution_id UUID NOT NULL REFERENCES institutions(id) ON DELETE RESTRICT,
    course_code TEXT NOT NULL CHECK (length(btrim(course_code)) > 0),
    display_name TEXT NOT NULL,
    description TEXT,
    department_name TEXT,
    academic_level_id TEXT REFERENCES academic_levels(id) ON DELETE SET NULL,
    credit_value NUMERIC(6, 2) CHECK (credit_value IS NULL OR credit_value >= 0),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    record_source TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (institution_id, course_code),
    UNIQUE (id, institution_id)
);
CREATE TABLE course_subjects (
    course_id UUID NOT NULL REFERENCES institution_courses(id) ON DELETE CASCADE,
    subject_id UUID NOT NULL REFERENCES academic_subjects(id) ON DELETE RESTRICT,
    mapping_source TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (course_id, subject_id)
);
CREATE INDEX course_subjects_subject_idx ON course_subjects (subject_id, course_id);

INSERT INTO academic_levels (id, display_name, source, display_order) VALUES
    ('secondary_school', 'Secondary School', 'peerfora_starter_directory', 10),
    ('foundation', 'Foundation', 'peerfora_starter_directory', 20),
    ('vocational', 'Vocational', 'peerfora_starter_directory', 30),
    ('technical', 'Technical', 'peerfora_starter_directory', 40),
    ('diploma', 'Diploma', 'peerfora_starter_directory', 50),
    ('associate_degree', 'Associate Degree', 'peerfora_starter_directory', 60),
    ('undergraduate', 'Undergraduate', 'peerfora_starter_directory', 70),
    ('masters', 'Master''s', 'peerfora_starter_directory', 80),
    ('doctoral', 'Doctoral', 'peerfora_starter_directory', 90),
    ('postdoctoral', 'Postdoctoral', 'peerfora_starter_directory', 100),
    ('professional_qualification', 'Professional Qualification', 'peerfora_starter_directory', 110),
    ('independent_learner', 'Independent Learner', 'peerfora_starter_directory', 120);
INSERT INTO academic_level_aliases (academic_level_id, alias, source)
VALUES ('secondary_school', 'high_school', 'peerfora_starter_directory');
INSERT INTO academic_disciplines (slug, display_name, source) VALUES
    ('computer-science', 'Computer Science', 'peerfora_starter_directory'),
    ('engineering', 'Engineering', 'peerfora_starter_directory'),
    ('medicine', 'Medicine', 'peerfora_starter_directory'),
    ('law', 'Law', 'peerfora_starter_directory'),
    ('mathematics', 'Mathematics', 'peerfora_starter_directory'),
    ('business', 'Business', 'peerfora_starter_directory'),
    ('arts', 'Arts', 'peerfora_starter_directory'),
    ('social-sciences', 'Social Sciences', 'peerfora_starter_directory'),
    ('natural-sciences', 'Natural Sciences', 'peerfora_starter_directory');
INSERT INTO academic_subjects (slug, display_name, discipline_id, source)
SELECT values_row.slug, values_row.display_name, discipline.id, 'peerfora_starter_directory'
FROM (VALUES
    ('data-structures', 'Data Structures', 'computer-science'),
    ('algorithms', 'Algorithms', 'computer-science'),
    ('machine-learning', 'Machine Learning', 'computer-science'),
    ('organic-chemistry', 'Organic Chemistry', 'natural-sciences'),
    ('calculus', 'Calculus', 'mathematics'),
    ('linear-algebra', 'Linear Algebra', 'mathematics'),
    ('human-anatomy', 'Human Anatomy', 'medicine'),
    ('constitutional-law', 'Constitutional Law', 'law'),
    ('research-methodology', 'Research Methodology', 'social-sciences')
) AS values_row(slug, display_name, discipline_slug)
JOIN academic_disciplines discipline ON discipline.slug = values_row.discipline_slug;
INSERT INTO academic_qualification_types (id, display_name, academic_level_id, source) VALUES
    ('bachelor_of_arts', 'Bachelor of Arts', 'undergraduate', 'peerfora_starter_directory'),
    ('bachelor_of_science', 'Bachelor of Science', 'undergraduate', 'peerfora_starter_directory'),
    ('higher_national_diploma', 'Higher National Diploma', 'diploma', 'peerfora_starter_directory'),
    ('master_of_arts', 'Master of Arts', 'masters', 'peerfora_starter_directory'),
    ('master_of_science', 'Master of Science', 'masters', 'peerfora_starter_directory'),
    ('master_of_business_administration', 'Master of Business Administration', 'masters', 'peerfora_starter_directory'),
    ('doctor_of_philosophy', 'Doctor of Philosophy', 'doctoral', 'peerfora_starter_directory'),
    ('doctor_of_education', 'Doctor of Education', 'doctoral', 'peerfora_starter_directory');
INSERT INTO institution_types (id, display_name, source) VALUES
    ('secondary_school', 'Secondary School', 'peerfora_starter_directory'),
    ('university', 'University', 'peerfora_starter_directory'),
    ('college', 'College', 'peerfora_starter_directory'),
    ('community_college', 'Community College', 'peerfora_starter_directory'),
    ('polytechnic', 'Polytechnic', 'peerfora_starter_directory'),
    ('technical', 'Technical Institution', 'peerfora_starter_directory'),
    ('vocational', 'Vocational Institution', 'peerfora_starter_directory'),
    ('online', 'Online Institution', 'peerfora_starter_directory'),
    ('research', 'Research Institution', 'peerfora_starter_directory'),
    ('professional_training', 'Professional Training', 'peerfora_starter_directory'),
    ('other', 'Other', 'peerfora_starter_directory');
