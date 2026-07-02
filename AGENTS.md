# Beacon Cast Project Instructions

## Project Goal

Beacon Cast is a production-grade personal activity beacon service for ESAP's public website.

The service collects activity signals from trusted local agents, stores them on the server, applies
server-owned visibility policy, and exposes a public page/API that shows what ESAP is currently
doing and a reviewable activity history.

This is not a toy script, a WakaTime clone, or a generic productivity dashboard. Treat it as an
Aster-style service that may be deployed on the existing public website server.

## Required Architecture

- Use the AsterForge `aster-service` template as the baseline for the server.
- Keep API routes under `/api/v1`.
- Follow AsterDrive and AsterYggdrasil conventions for frontend structure, embedded frontend
  serving, config loading, migrations, runtime assembly, health checks, and validation.
- Treat AsterDrive and AsterYggdrasil as module-level implementation references. Before adding or
  reshaping an API route, DTO, service, repository, migration, auth boundary, or frontend module,
  inspect the closest production pattern in those repos and mirror its structure unless there is a
  BeaconCast-specific reason to diverge.
- Use Actix and AsterForge runtime/middleware patterns from the template. Do not replace the
  server stack with Axum or an ad hoc HTTP framework unless explicitly requested.
- Keep the local agent as Rust code, either as a workspace binary/crate or a clearly separated
  package inside this repository.
- Prefer small, explicit modules over large mixed-responsibility files.

## Storage

- Use the normal AsterForge database setup with SeaORM and migrations.
- Do not make the project SQLite-only.
- Keep SQLite, PostgreSQL, and MySQL support enabled unless the user explicitly changes the
  deployment target.
- Product tables must be added through migrations. Avoid runtime schema creation in product code.
- Design storage so SQLite is the easy single-instance default, but PostgreSQL/MySQL remain valid
  deployment choices.
- Runtime-adjustable product policy must use the `system_config` pattern from AsterDrive instead
  of static `config.toml` values. Examples: default public visibility, offline timeout, history
  retention, public history enablement, and other admin-tunable behavior.
- Static config is for boot-critical service wiring only: server bind, database, cache,
  config-sync, logging, and similar startup concerns.

Expected product tables include, at minimum:

- `admin_users`
- `admin_sessions`
- `beacon_devices`
- `beacon_device_tokens`
- `activity_events`
- `activity_sessions`
- `manual_overrides`
- `audit_logs` or product-specific audit records integrated with the Forge audit pattern

## Privacy And Security Boundaries

Privacy is a core product invariant, not a UI option.

- Public APIs must return only server-approved public fields.
- Do not expose raw file paths, raw window titles, shell commands, chat contents, browser titles,
  environment variables, token values, or private directory names.
- Agent payloads must be treated as untrusted input even when they come from ESAP's own devices.
- Device tokens must be hashed at rest and revocable per device.
- Admin passwords must be hashed with a modern password hash. Do not store plaintext or reversible
  credentials.
- Admin APIs must require authenticated sessions.
- Agent ingest APIs must require bearer device tokens.
- Public APIs must not share internal IDs when public stable identifiers are enough.
- Any visibility override must be enforced server-side.
- Private mode must take precedence over all device signals.

## Visibility Model

The public page should support adjustable detail, controlled by server policy:

- `hidden`: publish nothing for this scope.
- `status_only`: publish only broad status such as coding, studying, writing, idle, offline.
- `project`: publish project/category level.
- `activity`: publish activity type such as fixing CI, writing frontend, studying IELTS.
- `session_note`: publish a sanitized session summary.
- `rich`: publish richer sanitized metadata such as branch or commit label only when explicitly
  allowed.

Never let the agent decide final public visibility. The agent can propose activity detail; the
server decides what becomes public.

Global visibility defaults and retention settings belong in `system_config`. More specific
per-device or per-project publication rules may use product tables later, but they must still be
resolved by server-side policy before any public DTO is emitted.

## Multi-Device Model

- The server is authoritative.
- Agents only report candidate signals.
- Each device must have an identity, display name, token, capability set, and last-seen metadata.
- Manual override has highest priority.
- Private mode overrides every device.
- Device priority and freshness should decide the primary activity when multiple devices report at
  the same time.
- First versions may expose only the primary activity publicly, but the storage model should not
  block richer multi-device summaries later.

## Authentication And Access

Initial admin authentication is username/password.

- Keep the authentication boundary replaceable so Forge/OIDC/GitHub login can be added later.
- Use secure HTTP-only cookies for admin browser sessions.
- Keep agent bearer-token authentication separate from admin browser authentication.
- Record security-relevant admin actions in audit logs.

## API Shape

Use `/api/v1`.

Expected public routes:

- `GET /api/v1/now`
- `GET /api/v1/activity-log`
- `GET /api/v1/activity-summary`

Expected agent routes:

- `POST /api/v1/beacons`
- `GET /api/v1/agent/config`

Expected admin routes:

- `POST /api/v1/admin/auth/login`
- `POST /api/v1/admin/auth/logout`
- `GET /api/v1/admin/me`
- `GET /api/v1/admin/devices`
- `POST /api/v1/admin/devices`
- `POST /api/v1/admin/devices/{id}/revoke`
- `GET /api/v1/admin/events`
- `GET /api/v1/admin/sessions`
- `PUT /api/v1/admin/visibility-policy`
- `POST /api/v1/admin/manual-override`
- `DELETE /api/v1/admin/manual-override`

Adjust exact DTOs after inspecting the generated template and current AsterForge APIs.

## Frontend

- Build the actual public status page and admin surface, not a landing page.
- Follow AsterDrive/AsterYggdrasil frontend separation patterns where practical.
- The public page should feel like a personal signal beacon or status terminal, not a corporate
  timesheet dashboard.
- The admin page is allowed to be quieter and denser, with clear device, policy, and log controls.
- Public UI must make stale/offline status obvious.

## Implementation Rules

- Before editing, inspect current generated files and relevant AsterDrive/AsterYggdrasil/Forge
  patterns.
- Keep changes scoped and compile after each meaningful batch.
- Add tests for policy enforcement, token handling, public DTO redaction, and route behavior.
- Do not add broad new abstractions without a concrete product boundary.
- Do not hardcode ESAP's private machine paths in server code. Put local path mappings in agent
  config.
- Do not write docs-only answers when code or config surfaces need to be fixed.

## Validation

Use focused validation while developing:

- `cargo fmt`
- `cargo check`
- targeted `cargo test`
- migration smoke tests after schema changes
- frontend typecheck/test after frontend edits

Full validation can be heavier, but do not leave known compile errors in touched crates.
