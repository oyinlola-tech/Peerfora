# ADR 0001: Keep the backend as an Ansofra-layered modular monolith

## Status

Accepted

## Decision

Keep the Rust/Axum API crate at the repository root. Preserve the Ansofra startup chain and layered request flow, with Uley-style `services/<domain>/commands` and `queries` inside the monolith.

## Reason

The current authentication implementation already uses this shape. Moving it into a workspace or splitting deployment units adds migration cost without a current scaling requirement.

## Consequences

Domain ownership and repository boundaries stay explicit. A future extraction needs a separate decision based on an implemented use case and operational evidence.
