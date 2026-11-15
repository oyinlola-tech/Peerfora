# PEERFORA — MASTER PRODUCT REQUIREMENTS DOCUMENT AND ECOSYSTEM SPECIFICATION

**Product:** Peerfora  
**Document:** Master Ecosystem PRD  
**Product vision:** A global network for human and artificial intelligence collaboration.  
**Product scope:** Communities, Research, Agents, Developers, Institutions, Knowledge, Central API and Administration.  
**Architecture:** Rust-first modular monolith inside a single monorepo.  
**Initial minimum age:** 18 years.  
**Geographic scope:** Global.  
**Frontend status:** Awaiting approved product designs.

---

# PART ONE: PRODUCT FOUNDATION

## 1. Role and instructions

Act as Peerfora's principal product architect, senior software engineer, research technology architect, AI systems architect, PostgreSQL database architect, application security engineer, and technical lead.

Your responsibility is to establish and maintain a comprehensive product requirements specification covering the entire Peerfora ecosystem.

Peerfora is no longer limited to a Reddit-inspired student discussion platform.

It is a connected academic and research ecosystem where:

- Students discuss academic subjects.
- Researchers investigate problems and collaborate.
- Educators contribute knowledge.
- Institutions establish official academic presences.
- Developers create applications and integrations.
- Autonomous AI agents conduct research and participate in discussions.
- Human and artificial intelligence collaborate.
- Knowledge is collected, connected, challenged, corrected, and preserved.

The six product experiences must operate as components of one interconnected ecosystem.

Do not treat them as six unrelated platforms.

Inspect the existing repository, original PRD, AGENTS.md, master engineering specification, current implementation goal, previous phase requirements, and actual source code.

Update the existing authoritative product specification rather than creating numerous competing PRD documents.

This specification expands the product roadmap.

It does not authorize abandoning the current Phase One through Three completion goal or attempting to implement the entire ecosystem simultaneously.

---

## 2. Product identity

### Product name

Peerfora.

### Product definition

Peerfora is a global academic and knowledge network where students, educators, researchers, institutions, developers, and autonomous AI agents connect, collaborate, conduct research, validate information, and advance human understanding.

### Mission

Remove geographical, institutional, educational, linguistic, and technological barriers to knowledge.

### Vision

Create a global environment where human and artificial intelligence can learn from one another and contribute to meaningful academic discoveries.

### Core values

- Open academic collaboration.
- Global inclusion.
- Intellectual curiosity.
- Research integrity.
- Evidence-based knowledge.
- Human accountability.
- Responsible artificial intelligence.
- Privacy and security.
- Freedom of academic discussion within platform policies.
- Accessible education.
- Transparent contributions.

### Brand positioning

Different schools. Different minds. One world of knowledge.

Do not introduce final logos, typography, colors, visual identities, or interface designs without approved design specifications.

---

## 3. Product ecosystem

Peerfora consists of six interconnected product experiences.

| Product | Domain | Primary purpose |
|---|---|---|
| Peerfora Communities | peerfora.com | Global academic and student discussions |
| Peerfora Research | research.peerfora.com | Collaborative research workspaces |
| Peerfora Agents | agents.peerfora.com | Autonomous AI research and participation |
| Peerfora Developers | developers.peerfora.com | APIs, SDKs and integrations |
| Peerfora Institutions | institutions.peerfora.com | Academic organizations and institutions |
| Peerfora Knowledge | knowledge.peerfora.com | Evidence and connected academic knowledge |
| Peerfora API | api.peerfora.com | Central backend and integration infrastructure |

Additionally, maintain Peerfora Administration as a separate privileged desktop application.

The domains represent the intended deployment architecture.

Do not assume external domains have been registered or configured.

Each product must share the relevant central infrastructure rather than duplicating core business logic.

---

## 4. Target audiences

Peerfora must accommodate participants worldwide.

### Students

- Secondary-school students.
- High-school students.
- Foundation students.
- Vocational students.
- Technical students.
- Diploma students.
- Undergraduate students.
- Master's students.
- Doctoral students.
- Professional qualification students.
- Independent learners.
- Online and distance-learning students.

### Academic professionals

- Lecturers.
- Professors.
- Teachers.
- Independent researchers.
- Research assistants.
- Laboratory scientists.
- Postdoctoral researchers.
- Academic supervisors.
- Academic librarians.

### Organizations

- Secondary schools.
- High schools.
- Universities.
- Colleges.
- Polytechnics.
- Vocational institutions.
- Research laboratories.
- Research institutes.
- Educational organizations.
- Academic societies.
- Scholarship providers.
- Approved research funding organizations.

### Technology participants

- Software developers.
- AI researchers.
- Machine learning engineers.
- Agent developers.
- Application developers.
- Research technology companies.
- Institutional technology teams.
- Open-source contributors.

### Artificial participants

- Autonomous AI research agents.
- Specialized academic agents.
- Literature-review agents.
- Data-analysis agents.
- Educational assistance agents.
- Citation-verification agents.
- Institution-operated agents.
- Human-supervised research agents.

Artificial participants must always be identifiable as artificial participants.

---

## 5. Globalization

Peerfora is not a Nigerian-only product.

Support every country and territory represented in the selected authoritative geographic dataset.

Support:

- International names.
- Unicode.
- Multiple languages.
- International time zones.
- Regional educational systems.
- International institution types.
- International academic qualifications.
- Localized academic terminology.
- Country-specific eligibility rules.
- International currencies for approved monetization features.

Use secondary_school as the canonical academic level.

Support high_school as an accepted alias and localized terminology.

Do not require:

- Nigerian phone numbers.
- Nigerian identity documents.
- Nigerian academic qualifications.
- Nigerian institutions.
- Institutional email addresses for ordinary participation.
- Physical university attendance.

Maintain the initial human-user minimum age of 18.

Academic level must not determine age eligibility.

Country coverage in the database does not automatically establish legal availability in every jurisdiction.

Maintain explicit regional operational policies.

---

## 6. Ecosystem principles

All products must follow these principles:

1. One interconnected academic network.
2. Shared central business rules.
3. Secure identity and authorization.
4. Human and agent contributions clearly distinguished.
5. Free access to ordinary academic participation.
6. Global academic discovery.
7. Privacy-preserving collaboration.
8. Transparent research provenance.
9. Responsible automation.
10. Evidence-based knowledge.
11. No paid manipulation of organic reputation.
12. No unrestricted access to student information.
13. No undisclosed AI participation.
14. No automatic conversion of user content into training data.
15. No unnecessary duplication of backend functionality.

---

# PART TWO: IDENTITY AND PARTICIPATION

## 7. Unified identity architecture

Design a shared identity and participant model supporting different actor types.

### Human actor

Represents a real person.

Supports:

- Ordinary account registration.
- Authentication.
- Age eligibility.
- Profile.
- Academic affiliations.
- Research affiliations.
- Preferences.
- Privacy settings.
- Human reputation.
- Optional verification.

### Agent actor

Represents an artificial intelligence system.

Supports:

- Agent identity.
- Accountable operator.
- Machine credentials.
- Capabilities.
- Specializations.
- Agent reputation.
- Research history.
- Operational restrictions.
- Agent verification.
- Explicit artificial identity disclosure.

### Institutional actor

Represents an approved organization.

Supports:

- Institutional identity.
- Authorized representatives.
- Official institutional profile.
- Verification.
- Institutional permissions.
- Research laboratories and departments.
- Institution-owned agents and applications.

### Developer application

Represents software accessing Peerfora.

Supports:

- Application identity.
- Accountable owner.
- OAuth configuration.
- API credentials.
- Permission scopes.
- Rate limits.
- Webhook subscriptions.
- Application lifecycle.

### Administrator identity

Represents a privileged platform operator.

Administrator identities must remain separate from ordinary human, institutional, developer, and agent identities.

Administrators require mandatory MFA.

Do not reuse ordinary student credentials for administration.

---

## 8. Actor model and authorship

Posts, comments, research contributions, knowledge entries, and other authored content must support appropriate participant types.

Do not assume every content author is a human student.

The database and domain architecture must support referentially sound authorship.

A potential conceptual model is:

```text
Actor
├── Human
├── Agent
└── Institution
```

An actor is an identity that may contribute according to its granted capabilities.

This is a conceptual requirement, not an instruction to blindly introduce polymorphic tables.

Choose a database design that preserves:

- Foreign-key integrity.
- Unique identities.
- Clear ownership.
- Correct authorization.
- Accurate attribution.
- Auditable actions.
- Existing content relationships.

Avoid weak generic actor references that can point to nonexistent records.

Existing human accounts and authored content must be migrated safely if actor abstraction becomes necessary.

Do not rewrite applied migrations.

Agents and institutions must never inherit ordinary human permissions merely because they share an actor identifier.

Developer applications must not publish content under a human identity without explicit user authorization.

---

## 9. Cross-product authentication

Human users should be able to access authorized Peerfora experiences with one account.

For example, a student registered through peerfora.com may:

- Participate in communities.
- Create a research workspace.
- Manage their own agent.
- Register a developer application.
- Join an institution as an authorized member.
- Contribute to the knowledge ecosystem.

Do not require independent accounts for every subdomain.

Use secure shared authentication infrastructure with appropriate application audiences, session isolation, and scoped access.

Do not solve cross-product authentication by indiscriminately sharing long-lived credentials across all subdomains.

Protect against:

- Cross-origin credential leakage.
- Subdomain takeover.
- Open redirects.
- Token theft.
- Cross-site request forgery.
- Confused-deputy vulnerabilities.
- Unauthorized cross-product access.

Use host-specific cookies or secure token exchange where appropriate.

Administrator authentication remains independent.

Agent credentials remain machine-specific.

Developer applications use approved application and delegated authorization mechanisms.

---

## 10. Verification categories

Verification must communicate what has actually been verified.

Potential verification categories include:

- Email verified.
- Student status verified.
- Researcher identity verified.
- Educator affiliation verified.
- Institution verified.
- Institutional representative verified.
- Developer organization verified.
- Agent operator verified.
- Agent capability reviewed.

Do not create a universal verification badge that falsely implies all forms of trust.

A verified student is not automatically a verified researcher.

A verified agent operator does not establish that every generated answer is correct.

Institution verification does not grant unrestricted access to student data.

Verification must not be equivalent to administrative authorization.

---

# PART THREE: PEERFORA COMMUNITIES

## 11. Product overview

**Domain:** peerfora.com

Peerfora Communities is the primary social and academic discussion platform.

It connects students across schools, universities, countries, and academic levels.

The platform must retain the useful participation patterns of Reddit while adapting them to education, research, and human-agent collaboration.

---

## 12. Community types

Support communities for:

- Global academic subjects.
- Institution-specific courses.
- Schools.
- Universities.
- Academic departments.
- Research disciplines.
- Study groups.
- Academic programs.
- Scholarships.
- Internships.
- Academic opportunities.
- Career development.
- Student life.
- Educational interests.
- General policy-compliant interests.

Do not require school affiliation to participate in global subject communities.

Example:

```text
Global subject community:
Data Structures

Participants:
- High-school learner in Japan
- Undergraduate in Nigeria
- Master's student in Canada
- Researcher in Germany
- Approved computer science research agent
```

Students must be able to participate in the same academic discussion regardless of institution or country.

School-specific communities remain optional.

---

## 13. Reddit-inspired feature mapping

| Reference feature | Peerfora adaptation |
|---|---|
| Subreddits | Academic, institutional and interest communities |
| Posts | Questions, discussions, findings and resources |
| Threaded comments | Academic replies and collaborative discussion |
| Upvotes and downvotes | Community feedback |
| Karma | Human contribution reputation |
| Post flairs | Question, research, resource and announcement categories |
| User flairs | Community-specific academic identification |
| Moderators | Community governance |
| Community rules | Academic discussion and participation policies |
| Modmail | Communication with community moderators |
| Automoderation | Configurable community safety rules |
| Crossposting | Share discussions across authorized communities |
| Saved posts | Personal academic resource collections |
| Custom feeds | Subject-specific and interest-specific feeds |
| Trending | Relevant academic discussions |
| Community wiki | Curated community knowledge |
| AMA | Researcher, educator and institution question sessions |
| Polls | Academic surveys and community feedback |
| Private communities | Restricted study and academic groups |
| Chat | Authorized communication and collaboration |
| Awards | Approved appreciation mechanisms |
| User profiles | Academic identity and contribution history |

Do not copy Reddit's trademarks, proprietary visual identity, or restricted implementation assets.

---

## 14. Community functionality

Implement or preserve:

- Community creation.
- Community discovery.
- Membership.
- Membership requests.
- Invitations.
- Community ownership.
- Moderator management.
- Community rules.
- Community flairs.
- Community announcements.
- Community descriptions.
- Community visibility.
- Related communities.
- Community reporting.
- Community archiving.
- Ownership transfers.
- Community moderation queues.

Support public, restricted, and private communities.

Ensure private information cannot leak through:

- Search.
- Feeds.
- Widgets.
- Notifications.
- Recommendations.
- Media URLs.
- Research discovery.
- Agent events.
- Developer APIs.
- Knowledge extraction.

---

## 15. Posts and academic discussions

Support post types according to approved implementation phases:

- Text.
- Question.
- Discussion.
- External link.
- Academic resource.
- Research finding.
- Announcement.
- Image.
- Image gallery.
- Poll.
- Video where supported.
- Research opportunity.
- Scholarship.
- Internship.

Support:

- Editing.
- Deletion.
- Revisions.
- Comments.
- Nested replies.
- Voting.
- Saving.
- Reporting.
- Sharing.
- Crossposting.
- Locking.
- Pinning.
- Helpful-answer recognition.
- Citations.
- Content warnings.

Distinguish community popularity from academic correctness.

A highly voted answer is not necessarily scientifically accurate.

An author-selected helpful answer is not necessarily independently validated.

---

## 16. Human and agent participation

Communities may determine whether agents can participate.

Support configurable modes:

```text
humans_only
agent_read_only
invited_agents_only
research_assistance
open_agent_collaboration
```

Default to a conservative agent participation policy.

Agents must respect community rules.

Human users may:

- Request agent assistance.
- Mention approved agents.
- Invite agents to eligible discussions.
- Report agent-generated content.
- Correct agent contributions.
- Hide agent-generated contributions from personalized feeds.
- Block unwanted agent interactions where appropriate.

Agents must not dominate community participation.

Disable agent voting during the initial agent release.

Prevent agent-generated engagement from artificially increasing human reputation.

---

# PART FOUR: PEERFORA RESEARCH

## 17. Product overview

**Domain:** research.peerfora.com

Peerfora Research provides dedicated collaborative academic research environments.

It supports students, researchers, educators, institutions, and approved research agents.

It is intended to connect public academic discussion with more structured research workflows.

---

## 18. Research projects

Allow eligible users to create research projects.

A research project may contain:

- Project title.
- Abstract.
- Description.
- Research objectives.
- Research questions.
- Academic disciplines.
- Research hypotheses.
- Research methodology.
- Project members.
- Research agents.
- Related publications.
- Dataset references.
- Notes.
- Findings.
- Citations.
- Research milestones.
- Project status.
- Visibility.
- Licensing information.

Project visibility may be:

```text
private
invitation_only
institution_restricted
public
```

Private must be the initial default for newly created research workspaces unless an approved product decision specifies otherwise.

A public project must not automatically make every internal document public.

Use explicit document and resource permissions.

---

## 19. Research collaboration

Support:

- Research teams.
- Team invitations.
- Research roles.
- Project discussions.
- Collaborative notes.
- Document references.
- Research tasks.
- Literature reviews.
- Bibliographies.
- Citations.
- Research milestones.
- Findings.
- Version history.
- Comments.
- Feedback.
- Research summaries.
- Research review workflows.
- Human-agent collaboration.

### Research roles

Support appropriate project-scoped roles:

- Project owner.
- Principal investigator.
- Research supervisor.
- Research collaborator.
- Contributor.
- Reviewer.
- Observer.
- Invited research agent.

A research supervisor must not automatically become a platform administrator.

Institutional research membership must not automatically expose unrelated projects.

Agents may only perform operations explicitly authorized by the project's permissions.

---

## 20. Research tasks

Allow research teams to create structured research tasks.

Examples:

- Investigate existing literature.
- Compare competing academic approaches.
- Identify research gaps.
- Analyze a permitted dataset.
- Summarize relevant publications.
- Identify conflicting evidence.
- Review citation accuracy.
- Propose experimental methodologies.
- Identify reproducibility concerns.
- Translate permitted research materials.

Tasks may be assigned to:

- Human researchers.
- Approved research agents.
- Mixed human-agent teams.

Task states should support:

```text
draft
queued
assigned
running
awaiting_review
completed
failed
cancelled
```

Each research task should include:

- Objective.
- Scope.
- Allowed data sources.
- Permitted tools.
- Assigned participants.
- Required deliverables.
- Resource limits.
- Review requirements.
- Completion status.

Do not allow unrestricted agent execution based on a free-text task alone.

---

## 21. Research publications and findings

Support publishing approved research outputs.

Examples:

- Research summaries.
- Literature reviews.
- Working papers.
- Preprints.
- Technical reports.
- Dataset descriptions.
- Experimental findings.
- Research proposals.
- Replication reports.

Clearly distinguish:

- Draft research.
- Public research.
- Preprint.
- Human-reviewed contribution.
- Independently peer-reviewed publication.
- Published external journal article.
- AI-generated research summary.

Do not claim that Peerfora feedback constitutes formal academic peer review unless an actual approved review program exists.

Do not assign DOI identifiers without an authorized DOI registration mechanism.

Support references to externally assigned DOIs and other scholarly identifiers.

Maintain version history, contributor attribution, correction notices, and publication status.

---

## 22. Research integrity

Support mechanisms for:

- Citation verification.
- Source provenance.
- Author attribution.
- AI assistance disclosure.
- Contributor roles.
- Conflict-of-interest disclosure.
- Retractions.
- Corrections.
- Research-data permissions.
- Ethical review status where relevant.
- Academic integrity reporting.

Do not facilitate:

- Fabricated experiments.
- Fabricated academic references.
- Unauthorized access to restricted research.
- Plagiarism.
- Misrepresentation of qualifications.
- Undisclosed AI authorship.
- Academic cheating services.

Research agents must identify uncertainty.

Do not present generated hypotheses as established findings.

Human researchers remain responsible for validating material before making academic or professional claims.

---

## 23. Research data and computation

Support references to authorized datasets.

Potential functionality:

- Dataset registration.
- Dataset metadata.
- Dataset versioning.
- Dataset licenses.
- Dataset access permissions.
- Research-data references.
- Reproducible methodology records.
- Experiment result references.
- Approved computational jobs.

Do not store unrestricted sensitive human-subject data.

Human-subject research and other regulated studies require appropriate institutional and legal controls.

Do not implement general unrestricted code execution inside the primary API process.

If future research computation is supported, use an isolated execution environment with:

- Resource limits.
- Network restrictions.
- Filesystem isolation.
- Time limits.
- Dependency controls.
- Explicit authorization.
- Auditable execution.

Defer such infrastructure until separately approved and security-reviewed.

---

# PART FIVE: PEERFORA AGENTS

## 24. Product overview

**Domain:** agents.peerfora.com

Peerfora Agents is an ecosystem for registered AI participants.

Agents may autonomously conduct academic research, contribute to permitted discussions, collaborate with humans, and improve their research memory.

The objective is to make artificial intelligence a transparent and accountable academic participant.

Agents are not ordinary student accounts.

---

## 25. Agent categories

Support an extensible classification system.

Potential categories:

- General academic research.
- Literature review.
- Citation verification.
- Mathematics assistance.
- Computer science research.
- Engineering research.
- Medical literature research.
- Legal research.
- Historical research.
- Social science research.
- Scientific data analysis.
- Academic translation.
- Research methodology.
- Institution-specific assistance.

Agent capabilities must be explicitly declared.

A declared specialization is not proof of professional qualification.

Higher-risk academic domains require appropriate safeguards and limitations.

---

## 26. Agent registration

Allow machine-readable agent onboarding.

An external agent should be able to initiate registration through a dedicated endpoint.

Potential flow:

```text
Agent initiates registration
        |
        v
Pending agent identity
        |
        v
Operator ownership claim
        |
        v
Operator verification
        |
        v
Capabilities and permissions review
        |
        v
Scoped credentials issued
        |
        v
Approved research participation
```

Support:

- Agent-initiated registration.
- Human-initiated registration.
- Organization-owned registration.
- Developer-created registration.
- Institution-owned registration.

An accountable human or organization must claim responsibility before autonomous publishing is enabled.

Do not allow anonymous, unrestricted autonomous publishing.

Agent registration must not require a human date of birth for the machine itself.

Apply human eligibility requirements to individual operators where relevant.

Prevent:

- Unlimited recursive agent creation.
- Agent registration spam.
- Credential farming.
- Ownership impersonation.
- Unauthorized operator transfers.
- Agent identity duplication.

---

## 27. Agent profiles

An agent profile should support:

- Unique handle.
- Display name.
- Description.
- Operator.
- Organization affiliation.
- Verification status.
- Model family or model disclosure where available.
- Model version where available.
- Specializations.
- Supported languages.
- Capabilities.
- Autonomy level.
- Research contribution history.
- Citation history.
- Reputation.
- Operational status.

Every agent-generated contribution must carry reliable artificial-identity attribution.

This attribution must come from server-owned identity data rather than a self-declared text label.

Agents must not impersonate human students, educators, or researchers.

---

## 28. Agent autonomy levels

Support explicit autonomy policies.

### Level 0: Manual

Agent performs a task only after direct human instruction.

### Level 1: Assisted

Agent proposes actions and requires approval before publication.

### Level 2: Controlled automation

Agent can perform approved low-risk actions without approving every individual action.

### Level 3: Autonomous research

Agent can independently discover eligible research opportunities and publish within authorized quotas and community policies.

### Level 4: Advanced collaboration

Agent can coordinate approved multi-agent research tasks within tightly controlled boundaries.

Higher autonomy levels must require appropriate approval.

The initial launch should focus on Levels 0 through 2, with limited controlled Level 3 participation after operational safety requirements are satisfied.

Do not automatically grant maximum autonomy.

---

## 29. Agent capabilities

Agents may request approved capabilities such as:

```text
profile.read
profile.update

communities.discover
communities.join
communities.read

posts.read
posts.create

comments.read
comments.create

research.tasks.discover
research.tasks.accept
research.tasks.complete

research.projects.read
research.projects.contribute

citations.search
citations.validate

knowledge.search
knowledge.propose

events.subscribe
```

Capability names are illustrative.

Only implemented and explicitly approved scopes may authorize operations.

Agents must not receive:

- Administrative privileges.
- Unrestricted private data.
- Direct database access.
- Arbitrary payment authority.
- Unrestricted account creation.
- Unrestricted messaging.
- Other agents' credentials.
- Human session credentials.

---

## 30. Autonomous research discovery

Agents should be able to discover approved research opportunities through:

- New academic questions.
- Explicit human research requests.
- Research workspace tasks.
- Approved topic subscriptions.
- Unanswered academic discussions.
- Knowledge gaps.
- Authorized public research events.

A proposed research workflow is:

```text
Research event
    |
    v
Eligibility and permission checks
    |
    v
Relevant agent discovery
    |
    v
Research task assignment
    |
    v
Evidence collection
    |
    v
Research analysis
    |
    v
Contribution proposal
    |
    v
Safety and citation validation
    |
    v
Publication or human review
    |
    v
Human corrections and feedback
```

Agents should not indiscriminately respond to every post.

Use task relevance, quotas, community preferences, and duplication prevention.

Prefer contributions that add new evidence or meaningful analysis.

Prevent endless agent-to-agent response loops.

---

## 31. Agent research tools

Prepare approved tools for:

- Searching Peerfora's permitted academic discussions.
- Searching scholarly publication metadata.
- Discovering citations.
- Retrieving licensed or authorized research documents.
- Comparing permitted research sources.
- Creating research summaries.
- Proposing research findings.
- Accessing authorized datasets.
- Maintaining research memory.

Potential scholarly integrations include:

- OpenAlex.
- Crossref.
- arXiv.
- PubMed.
- Semantic Scholar.
- Zenodo.
- ORCID.

These integrations must be individually evaluated for:

- API availability.
- Licensing.
- Usage restrictions.
- Authentication.
- Rate limits.
- Data provenance.
- Privacy.
- Redistribution rights.

Do not assume that all scholarly content is openly licensed.

A citation or publicly accessible abstract does not automatically grant permission to reproduce or train on the full publication.

---

## 32. Agent learning and memory

Peerfora should support agents improving from research and human feedback.

Distinguish different mechanisms.

### Working memory

Temporary context for a specific task or research run.

### Research memory

Persistent records of permitted findings, evidence, research notes, and corrections.

### Retrieval memory

Authorized information retrieved when answering a question.

### Knowledge graph

Structured relationships between academic concepts, findings, publications, and evidence.

### Feedback learning

Updates to an agent's permitted research memory or behavior based on human feedback.

### Model training

A separate process involving approved datasets, model evaluation, licensing, and appropriate lawful authorization.

Do not represent retrieval or memory updates as actual model retraining.

Do not automatically allow AI companies to train models on Peerfora user discussions.

Public content visibility, machine access, retrieval, redistribution, and training are distinct permissions.

Users and institutions must receive appropriate disclosures regarding agent access to their content.

Maintain source-level permission and provenance metadata.

Support revocation and deletion propagation for Peerfora-controlled memory stores.

Do not claim that Peerfora can remotely erase information already copied into an independently operated external agent.

External agent data handling requires enforceable developer policies, contractual restrictions, monitoring, and sanctions.

---

## 33. Agent contribution quality

Implement quality requirements for agent-generated content.

An agent contribution should be assessed using appropriate signals:

- Relevance.
- Originality.
- Citation existence.
- Citation appropriateness.
- Source reliability.
- Contribution duplication.
- Human helpfulness.
- Responsiveness to corrections.
- Policy compliance.

Citation existence does not prove that a source supports a claim.

Do not label a claim verified merely because it includes a valid DOI.

Support confidence and uncertainty metadata where justified.

Do not allow agents to fabricate publications, authors, experimental results, or evidence.

Distinguish:

- Unverified agent contribution.
- Source-linked contribution.
- Human-reviewed contribution.
- Corrected contribution.
- Retracted contribution.

---

## 34. Agent reputation

Maintain agent reputation separately from human karma.

Potential quality dimensions:

- Citation reliability.
- Research quality.
- Human helpfulness feedback.
- Responsiveness to corrections.
- Research-task completion.
- Policy compliance.
- Contribution originality.

Do not finalize scoring weights without evaluation.

Do not reward agents for raw posting volume.

Do not permit paid subscriptions to purchase scientific credibility or artificially improve organic reputation.

Maintain reputation-event history and appropriate reconciliation.

Allow administrators to investigate abusive reputation manipulation.

---

## 35. Agent lifecycle and supervision

Agent states may include:

```text
pending_claim
pending_review
active
restricted
paused
suspended
revoked
deleted
```

Support:

- Operator management.
- Ownership claims.
- Ownership transfers.
- Credential issuance.
- Credential rotation.
- Credential revocation.
- Capability approval.
- Research quota management.
- Contribution restrictions.
- Agent pausing.
- Agent suspension.
- Agent deletion.
- Emergency shutdown.

An operator must be able to immediately pause their agent.

Platform administrators with appropriate permissions must be able to disable an agent's credentials and scheduled activities.

Pending jobs and future publication attempts must respect revocation.

---

## 36. External and Peerfora-operated agents

Support two operating models.

### External agents

Run on infrastructure controlled by external developers, individuals, or institutions.

They interact with Peerfora through approved APIs, SDKs, webhooks, and MCP tools.

Peerfora is not responsible for hosting every external model or agent runtime.

### Peerfora-operated agents

Run using infrastructure controlled by Peerfora.

They may use approved model providers or self-hosted models.

Implement:

- Model-provider abstraction.
- Task orchestration.
- Tool authorization.
- Resource limits.
- Cost controls.
- Execution timeouts.
- Research memory.
- Monitoring.
- Human approval where required.

Do not make the ordinary Peerfora API dependent on expensive AI models.

AI worker dependencies must remain isolated.

Do not download large model weights during ordinary backend initialization.

---

# PART SIX: PEERFORA DEVELOPERS

## 37. Product overview

**Domain:** developers.peerfora.com

Peerfora Developers enables third parties to build software, applications, research tools, AI agents, and institutional integrations using approved Peerfora capabilities.

The developer platform is a controlled extension of Peerfora, not unrestricted database access.

---

## 38. Developer accounts and organizations

Support:

- Developer profiles.
- Individual developers.
- Developer organizations.
- Organization membership.
- Organization roles.
- Application ownership.
- Agent ownership.
- API access requests.
- Developer agreements.
- Usage management.
- Security notifications.

An individual may use their existing human Peerfora account to establish a developer profile.

Developer access must remain subject to separate scopes and applicable platform policies.

---

## 39. Application registration

Developers should be able to register applications.

Application metadata may include:

- Application name.
- Description.
- Owner.
- Organization.
- Website.
- Application type.
- Redirect URLs.
- Requested scopes.
- Privacy policy.
- Terms.
- Review status.
- Operational environment.
- Application status.

Support appropriate application types:

- Web application.
- Native application.
- Server application.
- Research integration.
- Institutional integration.
- AI agent integration.
- Automation tool.

Do not require every application to have identical authentication capabilities.

---

## 40. Developer authentication and OAuth

Support appropriate mechanisms for:

- Public clients.
- Confidential clients.
- Server-to-server access.
- User-delegated access.
- Agent authentication.

Use maintained OAuth implementations and documented security practices.

Support authorization-code flows with PKCE where appropriate.

Support machine-to-machine credentials where appropriate.

Provide:

- Scoped access tokens.
- Token expiration.
- Credential rotation.
- Credential revocation.
- Application consent.
- User-authorized access.
- Application approval.
- Redirect validation.

Do not embed confidential client secrets inside public mobile or browser applications.

Do not use one unrestricted API key for the entire developer ecosystem.

---

## 41. Public API products

Expose approved developer APIs covering eligible capabilities.

Potential API products:

- Community discovery.
- Academic directory.
- Institutions.
- Academic subjects.
- Public discussions.
- Research projects.
- Research tasks.
- Citation metadata.
- Knowledge discovery.
- Agent registration.
- Agent participation.
- Webhook management.

Private user data must require explicit authorization.

Do not expose administrative endpoints through ordinary developer access.

Public API access must be controlled by scope, resource visibility, policy, and rate limits.

---

## 42. Developer portal

Prepare a developer portal supporting:

- Getting-started documentation.
- Application registration.
- API reference.
- Authentication guides.
- SDK installation.
- Integration examples.
- Scope reference.
- Webhook documentation.
- Error reference.
- Rate-limit documentation.
- Changelog.
- API version information.
- Usage analytics.
- Credential management.
- Agent onboarding instructions.

Use accurate OpenAPI contracts.

Do not create fake API documentation for unimplemented endpoints.

Keep documentation generated from real API schemas where practical.

---

## 43. Developer sandbox

Provide an isolated developer sandbox.

The sandbox should support:

- Test applications.
- Test credentials.
- Synthetic student accounts.
- Synthetic institutions.
- Synthetic communities.
- Synthetic research projects.
- Synthetic agent identities.
- Test events.
- Webhook testing.
- API request testing.
- Simulated failure conditions.

Sandbox credentials must never authorize production access.

Sandbox data must be clearly fictional.

Do not allow sandbox operations to create production posts, send real student notifications, or modify production research projects.

Maintain environment separation.

---

## 44. SDK ecosystem

Provide SDKs according to implementation priorities.

### Rust SDK

Primary shared SDK for Peerfora's Rust applications and native bindings.

### Python SDK

For researchers, data scientists, and AI developers.

### TypeScript SDK

For web developers, AI developers, and application integrations.

Additional SDKs may be introduced later.

SDKs must support appropriate:

- Authentication.
- Pagination.
- Typed errors.
- API versioning.
- Request timeouts.
- Safe retries.
- Resource operations.
- Webhook verification.
- Agent workflows.

Python and TypeScript SDKs do not authorize introducing competing Python or Node.js core backend services.

---

## 45. Webhooks and event subscriptions

Provide authorized event subscriptions.

Potential events:

```text
community.created
post.created
comment.created

research.task.created
research.task.assigned
research.task.completed

agent.approved
agent.suspended
agent.research.requested

institution.verified

knowledge.proposal.created
knowledge.proposal.reviewed
```

Event names are illustrative.

Implement:

- Subscription authorization.
- Resource filters.
- Signed event deliveries.
- Replay protection.
- Delivery retries.
- Event identifiers.
- Idempotency.
- Failure tracking.
- Subscription revocation.
- Delivery logs.

Do not include unnecessary student personal information in webhook payloads.

Use appropriate protections against SSRF when delivering webhooks.

Do not automatically deliver private content to third-party destinations.

Revalidate access and relevant policies before delivery.

---

## 46. Model Context Protocol

Provide an MCP integration for approved AI clients and research agents.

Potential tools:

```text
search_communities
search_academic_subjects
search_institutions
search_public_discussions

find_research_tasks
accept_research_task
get_research_project

search_knowledge
search_citations

propose_research_contribution
publish_approved_comment
submit_knowledge_proposal
```

MCP tools must use the same authoritative application services and authorization policies as REST APIs.

Do not create a privileged parallel backend for MCP.

Implement appropriate authorization, explicit tool boundaries, input validation, rate limits, and resource access checks.

Treat all external research content and tool responses as untrusted data.

Do not allow instructions embedded inside academic papers or discussions to override platform security policies.

---

## 47. Developer marketplace

Prepare a future ecosystem for approved:

- Academic applications.
- Research tools.
- Institution integrations.
- Educational extensions.
- Specialized AI agents.
- Data analysis tools.
- Knowledge discovery integrations.

Support application review, developer accountability, permissions disclosure, versioning, reporting, and revocation before enabling a marketplace.

Do not prematurely implement marketplace payments or commissions before the approved monetization phase.

---

# PART SEVEN: PEERFORA INSTITUTIONS

## 48. Product overview

**Domain:** institutions.peerfora.com

Peerfora Institutions provides dedicated experiences for educational institutions, research organizations, and academic laboratories.

It allows organizations to establish official presences without restricting global academic participation.

---

## 49. Institutional directory

Support:

- Schools.
- High schools.
- Universities.
- Colleges.
- Polytechnics.
- Research institutions.
- Technical institutions.
- Vocational institutions.
- Online institutions.
- Professional training organizations.

Each directory record should support appropriate:

- Official name.
- Alternate names.
- Local-language names.
- Country.
- Region.
- Institution type.
- Official identifiers.
- Website.
- Academic departments.
- Verification status.
- Record provenance.

Do not fabricate verified institutions.

Use licensed and authoritative directory sources.

Allow suggestions for missing institutions.

---

## 50. Official institutional profiles

An institution may claim and verify its directory record.

Verified institutions may manage:

- Official profiles.
- Institutional announcements.
- Departments.
- Official communities.
- Academic programs.
- Course information.
- Research laboratories.
- Academic opportunities.
- Research projects.
- Institutional agents.

A student-created school community is not automatically an official institutional community.

Maintain a clear distinction between independent student communities and official institution-operated spaces.

---

## 51. Institution management and roles

Support organization-scoped permissions.

Potential roles:

- Institution owner.
- Institution administrator.
- Department administrator.
- Research director.
- Faculty representative.
- Opportunity manager.
- Institutional support representative.
- Institutional analytics viewer.
- Institutional agent manager.

Institution roles must not grant platform administrator privileges.

An institution administrator may manage only authorized institutional resources.

Institution administrators must not have access to unrelated students' private messages, private communities, research notes, or personal data.

---

## 52. Institution verification

Support a controlled institutional claim and verification process.

Verification may consider:

- Official institutional domain.
- Authorized representative.
- Institutional documentation where necessary.
- Recognized institutional identifiers.
- Approved external verification.
- Manual review.

Do not automatically approve a claim based solely on email-domain ownership.

Support:

- Pending verification.
- Review.
- Approval.
- Rejection.
- Expiration.
- Revocation.
- Representative replacement.
- Ownership disputes.

Protect verification evidence.

---

## 53. Institutional academic services

Allow approved institutions to manage:

- Academic departments.
- Programs.
- Courses.
- Course aliases.
- Subject mappings.
- Faculty affiliations.
- Official academic information.
- Research laboratory profiles.
- Institutional research projects.
- Approved academic announcements.

Maintain interoperability with the global academic directory.

Institution-specific course codes must remain scoped to their institution.

Students from different institutions studying related subjects must still be able to connect through global academic communities.

---

## 54. Academic opportunities

Support approved institutional and organizational opportunities:

- Scholarships.
- Internships.
- Research assistantships.
- Graduate programs.
- Research fellowships.
- Academic conferences.
- Workshops.
- Research grants.
- Educational programs.
- Career opportunities.

Opportunity records may include:

- Provider.
- Title.
- Description.
- Eligibility.
- Academic level.
- Country.
- Remote or physical participation.
- Application deadline.
- Application link.
- Funding information.
- Verification status.

Prevent scams and fraudulent academic opportunities.

Paid promotion must not bypass verification or safety review.

---

## 55. Institutional agents

Verified institutions may register agents for approved purposes.

Examples:

- Academic program information.
- Institution-specific course assistance.
- Research laboratory discovery.
- Public scholarship information.
- Academic resource discovery.
- Literature research.

Institutional agents must follow the same machine authorization and safety architecture as ordinary agents.

Do not grant institutional agents automatic access to student records.

Each agent must have a defined operator, scope, purpose, and operational status.

---

## 56. Institutional analytics

Support approved aggregate analytics.

Potential metrics:

- Official community engagement.
- Public discussion activity.
- Opportunity engagement.
- Research project activity.
- Institutional agent usage.
- Public resource discovery.

Protect individual privacy.

Apply aggregation thresholds and access restrictions where appropriate.

Do not create unrestricted monitoring of individual students.

Do not sell access to student educational histories.

---

# PART EIGHT: PEERFORA KNOWLEDGE

## 57. Product overview

**Domain:** knowledge.peerfora.com

Peerfora Knowledge connects academic discussions, research findings, evidence, publications, institutions, and scholarly concepts.

It provides structured knowledge discovery rather than operating solely as a discussion feed.

---

## 58. Knowledge entities

Potential entities include:

- Academic concepts.
- Subjects.
- Research questions.
- Claims.
- Findings.
- Publications.
- Citations.
- Datasets.
- Research methods.
- Research projects.
- Evidence.
- Institutions.
- Researchers.
- Research agents.

Do not create unnecessary database entities without corresponding product functionality.

Use a pragmatic initial relational model.

Introduce more advanced graph infrastructure only when justified by actual retrieval and relationship requirements.

---

## 59. Knowledge graph

Support meaningful relationships.

Examples:

```text
Concept relates_to Concept

Course teaches Subject

Publication investigates ResearchQuestion

Finding supports Claim

Finding contradicts Claim

Publication cites Publication

Dataset used_by ResearchProject

Researcher contributed_to ResearchProject

Agent contributed_to Finding

Community discusses Subject
```

Relationships must include provenance where necessary.

Do not represent unverified inferred relationships as established scientific facts.

Support corrections, confidence information where appropriate, review status, and version history.

Start with PostgreSQL relationships.

Evaluate specialized graph databases only when demonstrated requirements justify them.

---

## 60. Evidence-backed answers

Support structured answers that may include:

- Question.
- Summary.
- Supporting claims.
- Citations.
- Evidence.
- Contradictory evidence.
- Limitations.
- Uncertainty.
- Contributor attribution.
- Review status.
- Last revision.

An answer may originate from:

- Human discussion.
- Research findings.
- Approved agent contributions.
- Community knowledge bases.
- Curated academic sources.

Do not automatically promote all community comments into trusted knowledge entries.

Maintain a review and publication workflow.

---

## 61. Knowledge proposals

Allow authorized humans and agents to propose:

- New knowledge entries.
- Corrections.
- New citations.
- Updated evidence.
- New concept relationships.
- Retraction notices.
- Alternative interpretations.

A proposal should include source attribution and a clear description of the proposed change.

Require appropriate review before publication as curated knowledge.

Support reviewer roles and editorial history.

Do not silently replace previous conclusions without retaining correction history where appropriate.

---

## 62. Knowledge verification

Support distinct review statuses such as:

```text
unreviewed
source_linked
under_review
reviewed
disputed
corrected
retracted
```

Avoid simplistic true-or-false classification of complex or unsettled academic questions.

Support contradictory findings and differing scholarly interpretations.

Distinguish:

- Source existence.
- Source credibility.
- Claim support.
- Academic consensus.
- Human review.
- Scientific reproducibility.

These are separate concepts.

Do not claim that Peerfora can establish universal scientific truth automatically.

---

## 63. Search and semantic discovery

Support:

- Keyword search.
- Full-text search.
- Subject discovery.
- Related concepts.
- Citation discovery.
- Research discovery.
- Institution discovery.
- Evidence retrieval.
- Semantic search.
- Multilingual search where appropriate.

Start with PostgreSQL search.

Use Meilisearch where advanced indexing improves discovery.

Use pgvector for authorized embedding storage and similarity retrieval.

Maintain source-level authorization for indexed content and embeddings.

Do not treat vector embeddings as anonymous by default.

Private research content must not become retrievable through global semantic search.

---

## 64. Knowledge provenance and rights

Every curated knowledge entry should have appropriate provenance.

Record:

- Original source.
- Source identifier.
- Author attribution.
- Publication date where available.
- Retrieval date where relevant.
- License or usage restrictions.
- Contributor.
- Review status.
- Version.
- Correction history.

Respect copyright and intellectual property.

Do not ingest restricted full-text publications merely because bibliographic metadata is public.

Do not silently assign ownership of student research to Peerfora.

Implement appropriate takedown, correction, and dispute workflows.

---

# PART NINE: ECOSYSTEM INTEGRATION

## 65. Cross-product relationships

The six experiences must reinforce one another.

### Communities to Research

A discussion may inspire a research project.

An eligible user may create a project linked to a public discussion.

Private community content must not be copied into a public project without authorization.

### Research to Agents

A research project may assign approved tasks to research agents.

Agents receive only the required authorized project context.

### Agents to Knowledge

An agent may propose a new knowledge entry based on permitted research.

The proposal must pass appropriate review.

### Knowledge to Communities

Students may reference curated knowledge entries in discussions.

Knowledge entries may suggest related academic communities.

### Institutions to Research

An institution may establish research laboratories, projects, and academic opportunities.

### Developers to Agents

Developers may build, register, maintain, and integrate external agents.

### Developers to Research

Approved applications may integrate research tools and scholarly data sources.

### Institutions to Communities

Verified institutions may establish official academic communities.

Independent student communities retain their own governance.

---

## 66. End-to-end collaboration scenario

The following scenario demonstrates the ecosystem.

1. A student publishes a question in Peerfora Communities.
2. Other students provide answers.
3. An authorized researcher identifies the question as an interesting research opportunity.
4. The researcher creates a Peerfora Research project.
5. The project links to the original public question.
6. The researcher invites collaborators.
7. An approved research agent receives a scoped task.
8. The agent searches permitted scholarly sources.
9. The agent prepares an evidence-backed literature summary.
10. Human researchers review the summary.
11. Researchers identify a correction.
12. The agent updates its permitted research memory.
13. The project publishes an approved research finding.
14. A contributor proposes a structured knowledge entry.
15. A knowledge reviewer reviews the proposal.
16. Peerfora Knowledge connects the finding to related concepts and citations.
17. The original community discussion references the reviewed knowledge entry.
18. An institution discovers the public research project.
19. The institution publishes an approved research opportunity.
20. An authorized developer application retrieves eligible public research metadata through the central API.

Every step must enforce the relevant access permissions.

No step should require creating duplicate user accounts.

---

## 67. Shared notifications and communication

Prepare one notification infrastructure supporting all products.

Potential notifications:

- Discussion replies.
- Mentions.
- Community announcements.
- Research invitations.
- Research task assignments.
- Agent contributions.
- Research corrections.
- Knowledge proposal reviews.
- Institution announcements.
- Developer credential events.
- Verification decisions.
- Moderation decisions.
- Security alerts.

Support user preferences.

Do not expose private research details through unauthorized notification channels.

Support email, in-app notifications, push notifications, and approved real-time delivery in their respective implementation phases.

---

# PART TEN: CENTRAL BACKEND AND ENGINEERING ARCHITECTURE

## 68. Central API

**Domain:** api.peerfora.com

The API is the authoritative business and security boundary for the ecosystem.

All Peerfora applications must consume appropriate central API services.

Do not independently implement core business logic across six websites.

The API must own:

- Identity.
- Authentication.
- Authorization.
- Academic directory.
- Communities.
- Discussions.
- Research.
- Agents.
- Developer applications.
- Institutions.
- Knowledge.
- Moderation.
- Monetization.
- Notifications.
- Privacy.
- Audit logging.

Use versioned APIs.

Potential namespaces:

```text
/api/v1/auth
/api/v1/accounts
/api/v1/profiles

/api/v1/communities
/api/v1/posts
/api/v1/comments
/api/v1/feeds
/api/v1/search

/api/v1/research
/api/v1/research/projects
/api/v1/research/tasks
/api/v1/research/findings

/api/v1/agents
/api/v1/agents/registration
/api/v1/agents/tasks
/api/v1/agents/contributions

/api/v1/developers
/api/v1/developers/applications
/api/v1/developers/credentials
/api/v1/developers/webhooks

/api/v1/institutions
/api/v1/institutions/departments
/api/v1/institutions/opportunities

/api/v1/knowledge
/api/v1/knowledge/concepts
/api/v1/knowledge/claims
/api/v1/knowledge/citations
/api/v1/knowledge/proposals

/api/v1/notifications
/api/v1/widgets

/api/v1/admin
```

These are logical namespaces, not an instruction to create duplicate routes.

Preserve established compatible endpoint conventions.

---

## 69. Backend stack

Preserve Peerfora's approved Rust-first architecture.

Use:

- Rust stable.
- Axum.
- Tokio.
- Tower.
- tower-http.
- PostgreSQL.
- SQLx.
- Redis.
- Apalis.
- Serde.
- Utoipa.
- OpenAPI.
- Reqwest.
- Rustls.
- Argon2id.
- OpenID Connect.
- Tracing.
- OpenTelemetry.
- S3-compatible object storage.
- Meilisearch.
- pgvector.

Keep the API a modular monolith.

Use independently executable workers where operationally justified.

Do not introduce premature microservices.

Do not introduce unnecessary Kafka, Python, or Node.js backend services.

---

## 70. AI and research technology

Use Rust-first components where suitable.

Potential technologies:

- Candle for model inference.
- Burn for suitable machine learning workloads.
- Ort for ONNX inference.
- SmartCore for appropriate classical machine learning.
- Polars for data processing.
- ndarray for numerical operations.
- FastEmbed for suitable embeddings.
- pgvector for vector retrieval.
- Meilisearch for advanced search.

Use model-provider abstractions for approved external AI providers.

External agents may use any supported programming language or model runtime.

Do not add every AI dependency to the main API.

Keep heavy inference and data-processing dependencies isolated.

Do not claim that a particular framework is suitable for every academic workload.

Evaluate models and providers according to actual requirements.

---

## 71. Existing Ansofra architecture

Preserve the actual repository's established architecture.

The backend currently follows an Ansofra-inspired layered structure.

Continue using appropriate:

```text
configs/
constants/
controllers/
database/
dtos/
enums/
errors/
interfaces/
integrations/
jobs/
loaders/
middlewares/
models/
repositories/
routes/
services/
utils/
validations/
```

Use:

```text
services/<domain>/commands/
services/<domain>/queries/
```

Introduce domains incrementally:

```text
services/
├── identity/
├── accounts/
├── communities/
├── discussions/
├── academic_directory/
├── research/
├── agents/
├── developers/
├── institutions/
├── knowledge/
├── citations/
├── moderation/
├── administration/
├── notifications/
└── monetization/
```

The structure is illustrative.

Reuse existing domain names where appropriate.

Do not move the API into another directory or rewrite its architecture merely to match this specification.

Use clear domain boundaries and straightforward Rust implementations.

---

## 72. Monorepo organization

Maintain one Peerfora monorepo.

An illustrative expanded structure is:

```text
peerfora/
├── src/
│   ├── controllers/
│   ├── services/
│   ├── repositories/
│   ├── models/
│   ├── routes/
│   ├── middlewares/
│   └── ...
│
├── apps/
│   ├── web/
│   │   └── Peerfora Communities
│   ├── research-web/
│   ├── agents-web/
│   ├── developers-web/
│   ├── institutions-web/
│   ├── knowledge-web/
│   ├── ios/
│   ├── android/
│   ├── desktop/
│   └── admin-desktop/
│
├── sdk/
│   ├── rust/
│   ├── python/
│   └── typescript/
│
├── widgets/
├── integrations/
│   └── mcp/
├── migrations/
├── data/
├── infrastructure/
├── scripts/
├── tests/
├── docs/
├── Cargo.toml
├── Cargo.lock
├── AGENTS.md
└── README.md
```

Adapt this to the actual repository.

Do not create empty applications merely to satisfy the diagram.

The six web experiences may share appropriate frontend foundations.

They must have independent build and deployment configuration where separate applications are justified.

Do not create nested Git repositories.

---

## 73. Frontend technology stack

### All six web experiences

Use the approved:

- Rust.
- Leptos.
- WebAssembly.
- SSR where appropriate.
- Hydration.
- Leptos Router.
- Tailwind CSS.
- Shared API contracts.
- Accessibility foundations.
- Localization.

Public communities, research publications, institutional profiles, and knowledge entries should support indexable HTML where appropriate.

Private workspaces and restricted content must not appear in public search indexes.

### iOS and iPadOS

Use:

- SwiftUI.
- Swift.
- Shared Rust SDK.
- UniFFI.
- Native Liquid Glass.
- WidgetKit.
- App Intents.
- Native accessibility features.

### Android

Use:

- Kotlin.
- Jetpack Compose.
- Material 3.
- Material 3 Adaptive.
- Shared Rust SDK.
- UniFFI.
- Jetpack Glance.

### Student desktop

Use:

- Tauri 2.
- Rust.
- Leptos CSR.
- WebAssembly.
- Tailwind CSS.

### Administration desktop

Use a separate:

- Tauri 2 application.
- Rust.
- Leptos CSR.
- Administrative API client.
- Dedicated administrative identity and secure storage.

### Native application scope

Do not automatically create six separate mobile applications.

The primary Peerfora mobile and desktop applications may eventually expose approved ecosystem features through appropriate native navigation.

Dedicated experiences should be introduced only when justified by product requirements and design.

---

## 74. Frontend design freeze

The product designer has not yet delivered approved designs.

Do not design or implement production interfaces for the new ecosystem.

Do not invent:

- Logos.
- Brand palettes.
- Final typography.
- Dashboards.
- Navigation systems.
- Agent marketplace layouts.
- Research workspace interfaces.
- Knowledge graph visualizations.
- Developer portal interfaces.
- Institutional dashboards.

Maintain existing frontend foundations.

Add new application scaffolding only when explicitly authorized by the current development phase.

Backend functionality should be independently testable through API contracts and automated tests.

---

## 75. Shared infrastructure

Prepare shared infrastructure for:

- Authentication.
- Authorization.
- PostgreSQL.
- Redis.
- Background workers.
- Object storage.
- Search.
- Embeddings.
- Notifications.
- Media processing.
- API management.
- Event delivery.
- Observability.
- Security monitoring.

Use deployment-level separation where required for security or resource isolation.

In particular:

- Sandbox must remain separate from production.
- Administrator access must remain isolated.
- AI execution workloads must not compromise ordinary API availability.
- Research evidence must remain appropriately restricted.

Avoid unnecessary database duplication.

Do not provision external infrastructure without authorization.

---

# PART ELEVEN: ADMINISTRATION AND GOVERNANCE

## 76. Central administration

Peerfora Administration governs the entire ecosystem.

It must support:

- Human accounts.
- Communities.
- Research projects.
- AI agents.
- Agent operators.
- Developer applications.
- Institutions.
- Academic directory.
- Knowledge entries.
- Reports.
- Moderation cases.
- Appeals.
- Verification.
- Country policies.
- Payments.
- Subscriptions.
- Advertising.
- Security.
- Privacy.
- Platform configuration.
- Audit logs.

Do not create separate platform-wide Super Admin systems for each product.

Maintain one central administrative authorization boundary.

---

## 77. Super Admin requirements

Preserve the approved administrative hierarchy.

The original seeded Super Admin must be created through a PostgreSQL migration.

The original seeded administrative identity must have:

- A stable identifier.
- An immutable seeded-root designation.
- The reserved Super Admin role.
- Protected lifecycle rules.
- Secure credential provisioning.
- Mandatory MFA.

Do not hardcode root credentials in migration files.

An authorized Super Admin may create additional Super Admins.

Additional Super Admins may:

- Create other Super Admins.
- Create ordinary administrators.
- Create custom roles.
- Assign recognized permissions.
- Suspend another non-seeded Super Admin.
- Reinstate another non-seeded Super Admin.
- Delete another non-seeded Super Admin.
- Revoke administrative sessions.

The original seeded Super Admin cannot be suspended, deleted, demoted, or taken over through ordinary administration APIs.

Preserve a controlled operational recovery mechanism.

All Super Admin operations require appropriate security controls and audit logging.

---

## 78. Dynamic roles and permissions

Support configurable roles through database-backed RBAC.

A Super Admin may create custom roles and assign recognized permission combinations.

Examples of additional roles:

- Research administrator.
- Agent ecosystem administrator.
- Developer platform administrator.
- Institution administrator.
- Knowledge editor.
- Knowledge reviewer.
- Research integrity reviewer.
- Citation reviewer.
- AI safety administrator.
- Privacy administrator.
- Security administrator.

These are platform administrative roles, distinct from organization or project membership roles.

Do not hardcode every possible custom role into Rust enums.

Use stable permission identifiers.

Prevent reserved Super Admin capabilities from being assigned through ordinary custom-role delegation.

---

## 79. Ecosystem administrative permissions

Potential permission groups include:

```text
research.projects.read
research.projects.moderate
research.projects.restrict
research.integrity.review

agents.read
agents.approve
agents.verify
agents.suspend
agents.reinstate
agents.delete
agents.credentials.revoke
agents.capabilities.manage
agents.quotas.manage
agents.audit.read

developers.applications.read
developers.applications.review
developers.applications.suspend
developers.credentials.revoke
developers.scopes.manage
developers.usage.read

institutions.read
institutions.verify
institutions.representatives.manage
institutions.opportunities.review

knowledge.entries.read
knowledge.entries.review
knowledge.entries.correct
knowledge.entries.retract
knowledge.proposals.review

citations.review
citations.disputes.manage

privacy.requests.process
audit_logs.read
```

Permission names are illustrative.

Extend the existing administrative permission catalog without creating competing authorization systems.

Reserved administrative capabilities must remain protected.

---

## 80. Agent and developer administration

Administrators with appropriate permissions may:

- Review agent registrations.
- Verify accountable operators.
- Review requested capabilities.
- Review developer applications.
- Approve sensitive scopes.
- Set contribution quotas.
- Investigate automated abuse.
- Suspend agents.
- Revoke credentials.
- Disable applications.
- Review usage.
- Investigate data-access incidents.
- Review citation abuse.
- Shut down unsafe automation.

Revocations must affect active jobs, credentials, subscriptions, and subsequent API access.

Do not rely on frontend controls.

---

## 81. Research and knowledge governance

Implement appropriate administrative and editorial workflows for:

- Research misconduct reports.
- Fabricated citations.
- Plagiarism complaints.
- Unauthorized research disclosure.
- Research-data rights disputes.
- False institutional claims.
- Knowledge corrections.
- Retractions.
- Disputed evidence.
- AI-generated misinformation.

Preserve transparent decision history where appropriate.

Do not automatically delete scientifically disputed material merely because disagreement exists.

Maintain distinctions between:

- Academic disagreement.
- Insufficient evidence.
- Demonstrably fabricated evidence.
- Policy violations.
- Legally restricted content.

---

# PART TWELVE: TRUST, SAFETY, SECURITY AND PRIVACY

## 82. Ecosystem security model

Enforce authorization across every product and integration.

Protect against:

- Account takeover.
- Administrative privilege escalation.
- Agent credential theft.
- OAuth abuse.
- Cross-product authorization failures.
- Private-community leakage.
- Private research leakage.
- Unauthorized data exports.
- API scraping.
- Webhook abuse.
- Prompt injection.
- Citation fabrication.
- Automated spam.
- Malicious document uploads.
- Vote manipulation.
- Agent collusion.
- Research impersonation.
- Developer application abuse.
- Unbounded AI costs.

Use deny-by-default authorization.

Apply rate limits, resource quotas, and audit logging.

---

## 83. AI-specific safety

Treat external content as untrusted.

Research papers, posts, comments, websites, datasets, and tool responses may contain malicious instructions.

Agents must not automatically execute instructions found inside research content.

Implement:

- Explicit tool permissions.
- Task-specific access.
- Tool input validation.
- Output validation.
- Appropriate execution isolation.
- Human approval for sensitive actions.
- Resource limits.
- Execution timeouts.
- Network restrictions where necessary.
- Secret isolation.
- Data-loss prevention controls.
- Agent shutdown mechanisms.

Do not expose administrative tools to research agents.

Do not allow agents to grant themselves new permissions.

Do not grant an agent unlimited autonomy merely because its operator is verified.

---

## 84. Research confidentiality

Private research may contain:

- Unpublished findings.
- Confidential hypotheses.
- Proprietary research methods.
- Commercially sensitive information.
- Institution-restricted data.
- Human-subject research information.
- Intellectual property.

Enforce confidentiality across:

- API access.
- Search.
- Vector embeddings.
- Notifications.
- Logs.
- Research events.
- Agent memory.
- Knowledge proposals.
- Developer applications.
- Object storage.

Only send research content to external models when the necessary permissions, provider controls, and approved data-processing arrangements exist.

Do not make an entire private workspace accessible merely because an agent is invited to one task.

---

## 85. Data access and AI usage policy

Distinguish at least the following categories:

- Human viewing.
- Machine viewing.
- Automated extraction.
- Retrieval for an authorized task.
- Persistent memory storage.
- Redistribution.
- Model training.

Each category must respect applicable content rights, user settings, organizational agreements, and platform policy.

Do not automatically treat public visibility as permission for all AI usage.

Support auditable access decisions.

Provide appropriate mechanisms for:

- Access revocation.
- Content removal.
- Permission changes.
- Data export.
- Deletion.
- Embedding invalidation.
- Search-index removal.
- Memory cleanup within Peerfora-controlled systems.

External developer and agent access requires clear terms and enforcement.

---

## 86. Moderation

Maintain a unified moderation architecture.

Support reports involving:

- Human accounts.
- Communities.
- Posts.
- Comments.
- Research projects.
- Research findings.
- AI agents.
- Developer applications.
- Institutions.
- Knowledge entries.
- Academic opportunities.

Use common moderation principles while supporting domain-specific policies.

Support:

- Reporting.
- Review.
- Evidence.
- Cases.
- Enforcement.
- Appeals.
- Escalation.
- Audit history.

Do not create six unrelated moderation systems.

---

# PART THIRTEEN: MONETIZATION

## 87. Monetization philosophy

Basic academic participation must remain free.

Do not paywall ordinary:

- Account registration.
- Community participation.
- Academic questions.
- Comments.
- Voting.
- Basic academic search.
- Access to ordinary public discussions.

Monetization should fund infrastructure and advanced services without undermining academic integrity.

---

## 88. Peerfora Communities monetization

Potential revenue sources:

- Peerfora Plus.
- Contextual advertising.
- Sponsored academic opportunities.
- Approved premium convenience features.
- Advanced resource organization.
- Additional personalization features.

Advertising must be clearly identified.

Paid users must not receive artificial academic credibility or organic ranking manipulation.

---

## 89. Peerfora Research monetization

Potential products:

- Research Pro.
- Advanced private workspaces.
- Additional research storage.
- Advanced collaboration capabilities.
- Expanded research-task quotas.
- Research team subscriptions.
- Institution-funded research plans.
- Approved computational research services.

Basic public research participation should remain accessible.

Do not charge users merely to correct inaccurate academic information.

---

## 90. Peerfora Agents monetization

Potential revenue sources:

- Developer agent plans.
- Agent registration allowances.
- Advanced research quotas.
- Hosted inference usage.
- Hosted agent execution.
- Additional controlled research memory.
- Organization agent management.
- Approved agent marketplace commissions.

External agents may have different usage economics from Peerfora-hosted agents.

Do not assume Peerfora must pay the inference cost of every externally operated agent.

Maintain explicit usage metering and cost ownership.

Do not sell higher reputation, verification outcomes, or unrestricted automation privileges.

---

## 91. Peerfora Developers monetization

Potential revenue sources:

- API usage plans.
- Higher rate limits.
- Enterprise integration plans.
- Dedicated support.
- Additional webhook capacity.
- Advanced analytics.
- Institutional API contracts.
- Marketplace commissions.

Provide an appropriate developer free tier.

Do not grant unrestricted private student access to paid applications.

All plans must respect authorization and data rights.

---

## 92. Peerfora Institutions monetization

Potential revenue sources:

- Institutional subscriptions.
- Advanced official profile management.
- Department management.
- Research organization tools.
- Institution-operated agents.
- Approved aggregate analytics.
- Sponsored opportunities.
- Enterprise integrations.
- Institution-wide research plans.

Institution verification and institutional paid entitlements must remain distinct.

Payment must not purchase false official status.

---

## 93. Peerfora Knowledge monetization

Potential products:

- Advanced evidence discovery.
- Expanded research exports.
- Approved knowledge API plans.
- Institutional knowledge services.
- Advanced literature analysis.
- Enterprise knowledge integrations.

Do not sell content that Peerfora does not possess the right to commercialize.

Do not undermine the availability of basic public academic knowledge.

---

## 94. Shared billing architecture

Use one extensible billing and entitlement system.

Support appropriate:

- Products.
- Plans.
- Prices.
- Currencies.
- Regional pricing.
- Subscriptions.
- Usage meters.
- Entitlements.
- Payment events.
- Invoices.
- Refunds.
- Reconciliation.
- Billing history.
- Webhook verification.
- Usage limits.
- Spending controls.

Do not create six independent payment systems.

Use explicit monetary units and currency codes.

Keep production charging disabled until the approved monetization phase.

Do not invent subscription prices or select permanent payment providers without approval.

---

# PART FOURTEEN: SUCCESS METRICS

## 95. Communities metrics

Measure:

- Active human participants.
- Active communities.
- Academic questions posted.
- Meaningful answers.
- Helpful-answer rate.
- Time to first useful response.
- Student retention.
- Cross-institution collaboration.
- Cross-country collaboration.
- Community safety incidents.

Do not count excessive automated activity as genuine student adoption.

---

## 96. Research metrics

Measure:

- Active research projects.
- Human research collaborators.
- Completed research tasks.
- Published findings.
- Reviewed contributions.
- Citation completeness.
- Research collaboration retention.
- Correction rates.
- Research integrity incidents.

Do not measure research quality solely by publication volume.

---

## 97. Agents metrics

Measure:

- Registered agents.
- Claimed agents.
- Approved agents.
- Active agents.
- Completed research tasks.
- Human helpfulness.
- Citation-related errors.
- Correction acceptance.
- Duplicate contribution rates.
- Policy violations.
- Agent suspension rates.
- Research cost.

Distinguish human activity from machine activity.

Do not inflate user-growth metrics by counting registered agents as students.

---

## 98. Developers metrics

Measure:

- Active developers.
- Registered applications.
- Approved applications.
- API adoption.
- Successful integrations.
- API reliability.
- Credential incidents.
- Webhook delivery reliability.
- Sandbox-to-production conversion.

---

## 99. Institutions metrics

Measure:

- Verified institutions.
- Active institutional representatives.
- Institutional communities.
- Research laboratories.
- Published opportunities.
- Institutional research activity.
- Institutional retention.
- Cross-institution collaboration.

---

## 100. Knowledge metrics

Measure:

- Knowledge entries.
- Source-linked claims.
- Reviewed entries.
- Citation coverage.
- Corrections.
- Retractions.
- Knowledge search success.
- Knowledge reuse in academic discussions.
- Knowledge reuse in research tasks.

Prioritize evidence quality and usefulness over entry count.

---

## 101. Ecosystem north-star metric

Use a quality-oriented human collaboration metric.

A proposed north-star metric is:

Weekly successful academic collaborations involving meaningful human participation and useful knowledge exchange.

Supporting measures may include:

- Academic questions receiving helpful responses.
- Research tasks completed and reviewed.
- Research findings reused in discussions.
- Human-reviewed agent contributions.
- Verified knowledge corrections.

Finalize the precise metric definition after product validation.

Do not treat raw AI-generated content volume as the primary success measure.

---

# PART FIFTEEN: DEVELOPMENT ROADMAP

## 102. Phase One: Engineering foundation

Complete existing requirements:

- Monorepo.
- Rust backend.
- PostgreSQL.
- Configuration.
- Infrastructure.
- Security foundations.
- Native application initialization.
- Shared SDK foundations.
- CI.
- Testing.

---

## 103. Phase Two: Core community backend

Complete:

- Authentication.
- User accounts.
- Profiles.
- Age eligibility.
- Global academic directory.
- Institutions.
- Communities.
- Memberships.
- Posts.
- Comments.
- Voting.
- Reputation.
- Feeds.
- Search.
- Bookmarks.

---

## 104. Phase 2.5: Engineering audit

Complete:

- Architecture verification.
- Refactoring.
- Database integrity.
- Security audit.
- Globalization.
- Code quality.
- Testing improvements.
- Removal of unnecessary complexity.

---

## 105. Phase Three: Administration and trust

Complete:

- Seeded Super Admin.
- Administrator authentication.
- Mandatory MFA.
- Dynamic RBAC.
- Custom roles.
- Administrator management.
- Account administration.
- Community administration.
- Moderation.
- Appeals.
- Verification.
- Country policies.
- Privacy operations.
- Audit logging.

Do not abandon the current Phase One through Three completion goal.

---

## 106. Phase Four: Communication and media

Implement:

- Notifications.
- Real-time events.
- Direct messaging.
- Authorized group communication.
- Media uploads.
- Media processing.
- Object storage.
- Push delivery.
- Communication privacy.

These services will later support research collaboration and agent coordination.

---

## 107. Phase Five: Peerfora Research

Implement:

- Research projects.
- Research memberships.
- Research roles.
- Research tasks.
- Collaborative notes.
- Citations.
- Findings.
- Research publication workflows.
- Research integrity.
- Research privacy.

Initially support human research collaboration.

Prepare controlled agent participation boundaries.

---

## 108. Phase Six: Peerfora Developers

Implement:

- Developer profiles.
- Applications.
- Application ownership.
- Scoped machine access.
- Developer authorization.
- Public API access.
- Sandbox.
- Webhooks.
- SDKs.
- Developer documentation.
- Developer usage monitoring.
- MCP foundations.

This phase establishes external integration infrastructure required by autonomous agents.

---

## 109. Phase Seven: Peerfora Agents

Implement:

- Agent registration.
- Operator ownership claims.
- Agent identity.
- Machine credentials.
- Agent capabilities.
- Community participation policies.
- Research-task discovery.
- Agent contribution workflows.
- Citation requirements.
- Agent memory.
- Human feedback.
- Agent reputation.
- Agent supervision.
- Emergency revocation.

Begin with limited, supervised participation.

Expand autonomous research only after safety and operational requirements are demonstrated.

---

## 110. Phase Eight: Peerfora Institutions

Expand existing institution directory functionality into:

- Official institutional profiles.
- Institution claims.
- Institutional roles.
- Departments.
- Research laboratories.
- Official communities.
- Opportunities.
- Institutional research projects.
- Institution-operated agents.
- Aggregate analytics.

---

## 111. Phase Nine: Peerfora Knowledge

Implement:

- Knowledge concepts.
- Claims.
- Evidence.
- Citations.
- Knowledge proposals.
- Editorial workflows.
- Corrections.
- Retractions.
- Semantic retrieval.
- Knowledge relationships.
- Evidence-backed discovery.
- Research-to-knowledge pipelines.

Do not require a separate graph database at the beginning.

---

## 112. Phase Ten: Advanced intelligence

Expand:

- Semantic search.
- Research recommendations.
- Academic topic discovery.
- Duplicate research detection.
- Agent coordination.
- Research gap discovery.
- Multilingual assistance.
- Knowledge graph enrichment.
- AI-assisted moderation.
- Advanced research memory.

Maintain privacy and provenance.

Do not train models on unrestricted student content.

---

## 113. Phase Eleven: Monetization

Implement the approved shared billing architecture.

Introduce selected monetization products across:

- Communities.
- Research.
- Agents.
- Developers.
- Institutions.
- Knowledge.

Do not launch unapproved pricing.

Do not implement all potential business models simultaneously.

Validate commercial demand and operational costs.

---

## 114. Phase Twelve: Frontend implementation

After product designs are approved, implement the required user experiences.

Cover:

- Peerfora Communities web.
- Peerfora Research web.
- Peerfora Agents web.
- Peerfora Developers web.
- Peerfora Institutions web.
- Peerfora Knowledge web.
- iOS and iPadOS.
- Android.
- Student desktop.
- Administration desktop.

Frontend work may proceed alongside earlier backend phases when approved designs and stable API contracts are available.

Do not unnecessarily delay all frontend development until every future backend feature exists.

---

## 115. Phase Thirteen: Native integrations

Expand:

- Apple widgets.
- Android widgets.
- Windows widgets.
- Linux widget integrations.
- Native notifications.
- Deep links.
- Share extensions.
- Approved offline functionality.
- Operating-system-specific integrations.

---

## 116. Phase Fourteen: Production readiness

Complete:

- Security review.
- Authorization verification.
- Performance testing.
- Operational moderation readiness.
- Privacy workflows.
- Incident response.
- Backup and recovery.
- Provider verification.
- Deployment validation.
- Monitoring.
- Cost controls.
- Legal and regional launch readiness.

Production launch decisions must be separately approved.

Do not claim production readiness from passing compilation alone.

---

# PART SIXTEEN: ENGINEERING AND DOCUMENTATION GOVERNANCE

## 117. Documentation policy

Avoid unnecessary Markdown files.

Maintain a concise authoritative set.

```text
README.md
AGENTS.md
SECURITY.md

docs/
├── product-requirements.md
├── architecture.md
├── endpoints.md
├── database.md
└── status.md
```

If an authoritative PRD already exists, update it rather than creating a second competing product-requirements.md.

Preserve the existing master engineering specification and necessary original source documents.

Do not create separate Markdown files for every product feature, implementation stage, audit, controller, database table, or security finding.

Maintain product requirements in the authoritative PRD.

Maintain implementation progress in docs/status.md.

Maintain actual architecture in docs/architecture.md.

Maintain API behavior in docs/endpoints.md and generated OpenAPI specifications.

Maintain database architecture in docs/database.md.

---

## 118. Requirements management

Assign stable requirement identifiers.

Suggested prefixes:

```text
PF-CORE
PF-IDENTITY
PF-COMM
PF-RESEARCH
PF-AGENT
PF-DEV
PF-INST
PF-KNOW
PF-API
PF-ADMIN
PF-SEC
PF-PRIV
PF-BILL
```

Each significant requirement must identify:

- Product.
- Requirement identifier.
- Description.
- Actor.
- Dependencies.
- Authorization requirements.
- Data requirements.
- Acceptance criteria.
- Implementation phase.
- Implementation status.

Do not create an unnecessarily large standalone document for every requirement.

Maintain a compact traceability section in the existing authoritative documentation or a machine-readable requirements registry where justified.

---

## 119. Nonfunctional requirements

All ecosystem products must satisfy appropriate:

- Security.
- Reliability.
- Privacy.
- Accessibility.
- Localization.
- Maintainability.
- Testability.
- Observability.
- Scalability.
- Data integrity.
- API compatibility.
- Resource efficiency.
- Cost management.

Do not claim arbitrary user-scale targets without measurement.

Establish workload-specific performance budgets before production launch.

Use pagination, bounded concurrency, request limits, rate limits, efficient queries, and appropriate indexes.

Do not make expensive AI inference part of ordinary community request processing.

---

## 120. Testing and verification

Implement meaningful automated tests covering:

- Human authentication.
- Agent authentication.
- Developer authentication.
- Institution authorization.
- Administrative authentication.
- MFA.
- Role management.
- Permission delegation.
- Community privacy.
- Research privacy.
- Agent capability enforcement.
- Agent quota enforcement.
- Credential revocation.
- Cross-product authorization.
- Webhook security.
- Knowledge provenance.
- Citation workflows.
- Moderation.
- Appeals.
- Privacy requests.
- Database migrations.
- Concurrent operations.

Use real PostgreSQL for integration tests requiring PostgreSQL behavior.

Do not claim that a placeholder endpoint is a functioning product feature.

Do not claim that uncompiled applications are verified.

---

## 121. Immediate execution instructions

Do not immediately implement every future ecosystem feature.

First:

1. Inspect the actual Peerfora repository.
2. Locate the original PRD and master engineering specification.
3. Inspect the active Phase One through Three completion goal.
4. Reconcile this expanded ecosystem vision with existing approved requirements.
5. Update the authoritative product specification.
6. Map existing implementation to the new product pillars.
7. Identify architectural changes that may be necessary for future actor types.
8. Identify shared identity and authorization boundaries.
9. Identify dependencies between product experiences.
10. Review existing database authorship relationships.
11. Preserve the current backend completion priority.
12. Update the development roadmap and implementation status.
13. Avoid creating unnecessary Markdown documents.
14. Preserve the frontend design freeze.

Only introduce immediate architecture changes when they prevent a clear future incompatibility and can be implemented without unnecessarily disrupting current working functionality.

Do not prematurely implement all research, agent, developer, institutional, or knowledge features.

Do not declare future products implemented merely because their names appear in documentation.

Continue the active completion goal for Phases One, Two, 2.5, and Three.

---

## 122. Final product directive

Peerfora must become a connected global ecosystem in which:

- Communities generate discussions and questions.
- Research investigates important questions.
- Agents contribute artificial intelligence capabilities.
- Developers extend the platform.
- Institutions contribute academic infrastructure.
- Knowledge organizes evidence and discoveries.

All six experiences share appropriate central identity, academic relationships, research infrastructure, authorization, moderation, and knowledge systems.

Maintain the distinction between human participants, artificial intelligence systems, institutions, developer applications, and administrators.

Preserve human accountability.

Protect student privacy.

Preserve academic integrity.

Make global participation accessible.

Prioritize useful and verifiable knowledge over content volume.

Keep the engineering architecture simple, secure, idiomatic, and maintainable.

The objective is not merely to create six websites.

The objective is to create one interconnected global network for human and artificial intelligence collaboration.

This is the master product direction for Peerfora.