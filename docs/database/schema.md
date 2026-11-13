# Database schema

The currently persisted schema is defined by the SQL files in `migrations/`. Account authentication persists account, session, one-time token, and social identity records. Date of birth is used for eligibility checks and is not stored.

No community, academic directory, discussion, feed, moderation, or billing tables are implemented yet. Review the migration before changing persistence and keep SQL in migrations and repository code.
