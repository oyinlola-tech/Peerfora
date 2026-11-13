# ADR 0002: Initialize client platforms as separate projects

## Status

Accepted

## Decision

Keep web, iOS, Android, student desktop, administration desktop, and widget projects in separate platform directories. Share neutral static tokens and localization resources where the platform can consume them.

## Reason

The master specification calls for native conventions on mobile and widgets, distinct student/admin trust boundaries, and a minimal web app. A single cross-platform UI would not satisfy those needs.

## Consequences

Each target has its own build prerequisites and must report its build status separately. The current shells are temporary and do not establish final designs or account integrations.
