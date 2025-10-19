# Business Analyst Full Analysis - The Warehouse Hub (TWH)

This document contains the complete, human-readable version of the structured requirements generated from `business_plan.md` following the Business Analyst Agent prompt. It is intentionally comprehensive and includes each requirement with acceptance criteria, dependencies, ambiguities, user stories, as well as out-of-scope items, assumptions, and risks.

## Requirements

### REQ-001 — Publish OpenAPI v1.1 public API (P0, S, functional)

Description

Publish a stable, versioned OpenAPI 3.0+ specification (v1.1) that describes all public REST endpoints, request/response schemas, authentication, error codes, and examples. Include a Postman collection and JSON/YAML artifacts in the repo.

Business value

Allows developers to evaluate and integrate with the platform quickly; foundation for SDK generation and sandbox testing.

Acceptance criteria

- OpenAPI YAML and JSON artifacts are committed to the repository under `/openapi` and reference v1.1
- Postman collection exported and included in `/openapi/postman_collection.json`
- Automated schema validation (CI job) runs and passes on pull requests
- API example in README demonstrates an end-to-end call that succeeds against sandbox

Ambiguities

- Confirm whether OpenAPI must include internal/admin endpoints or only public endpoints
- Clarify the expected stability policy & deprecation timeline

User story

- As a developer, I want a complete OpenAPI spec so that I can auto-generate client SDKs and understand the API contract.

### REQ-002 — Core Items CRUD endpoints (P0, M, functional)

Description

Implement REST endpoints for Items including create, read, update, delete, list with pagination, filtering, and relationships to stock records. Ensure validation, structured errors, and ETag support on read responses.

Business value

Fundamental building block for inventory modeling - enables customers to store product metadata and link to stock movements.

Acceptance criteria

- POST `/items` creates an item and returns 201 with item body
- GET `/items/{id}` returns 200 and includes ETag header
- PUT/PATCH `/items/{id}` supports If-Match header and returns 412 on ETag mismatch
- DELETE `/items/{id}` returns 204 and subsequent GET returns 404
- List endpoint supports limit/offset (or cursor) and optional filters by sku, name

Ambiguities

- Item schema required fields (sku vs barcode) - confirm canonical identifiers
- Soft-delete vs hard delete policy for items

User story

- As a product manager, I want to create and update items so that our inventory can be tracked in the system.

### REQ-003 — Ledger-first stock_movements write API (P0, L, functional)

Description

Provide an append-only `stock_movements` API that records every change to stock with source metadata, quantity deltas, and `causation_id` to link to orders, adjustments, or external systems. Each write must be atomic and durable.

Business value

Ensures immutable audit trail and enables consistent reconciliation and analytics; core differentiation of ledger-first approach.

Acceptance criteria

- POST `/stock_movements` accepts batch and single writes and returns 201 with persisted IDs
- Writes are appended; attempting to modify an existing movement returns 400/405
- Each movement stores `source_type`, `source_id`, `location_id`, `quantity_delta`, `timestamp`, and `causation_id`
- A nightly reconciliation job (REQ-011) can consume movements and produce consistent `stock_levels`

Ambiguities

- Exactly which fields are mandatory on `stock_movements` (e.g., `batch_id`, lot information)
- Retention policy for movements and archival strategy

User story

- As an integration developer, I want to append stock movements with source metadata so that downstream systems can reconcile inventory changes with an immutable ledger.

### REQ-004 — Transactional stock_levels snapshot and reconciliation (P0, XL, functional)

Description

Maintain a materialized `stock_levels` projection (per item/location/tenant) derived from `stock_movements`. Implement nightly reconciliation that validates ledger totals against snapshots and produces discrepancy reports and alerts.

Business value

Provides fast read access for POS queries while preserving ledger correctness; reduces operational risk from data drift.

Acceptance criteria

- Read stores expose `GET /stock_levels?item_id=&location_id=` returning current available quantity
- Nightly reconciliation job runs and emits a report with list of mismatched items and corrective actions
- An alert is created when discrepancy rate exceeds configured threshold (e.g., >0.1% of SKUs)
- Snapshot updates are idempotent and do not lose concurrent `stock_movements`

Ambiguities

- SLA for reconciliation (time budget), and acceptable mismatch thresholds
- Whether projections must be strongly consistent or eventual for MVP

User story

- As an operations manager, I want daily reconciliation reports so that we can investigate and resolve inventory discrepancies.

### REQ-005 — Idempotency and ETag concurrency controls (P0, L, non-functional)

Description

Support idempotency keys on POST endpoints and ETag/If-Match for safe concurrent updates across distributed clients. Persist idempotency keys and results to ensure at-least-once-safe semantics.

Business value

Prevents duplicate events and race conditions in integrations; improves reliability and developer confidence.

Acceptance criteria

- Endpoints that mutate state accept `Idempotency-Key` header and return same response for repeated keys within TTL
- Responses that change resources include `ETag` header; conditional requests using `If-Match` succeed or return 412
- Idempotency record retention is configurable and documented
- Automated tests demonstrate duplicate POSTs with same key produce one ledger entry

Ambiguities

- Default TTL for idempotency keys and storage sizing
- Which endpoints require idempotency for MVP

User story

- As an integrator, I want idempotency keys so that retries from network failures don't create duplicate stock movements.

### REQ-006 — Webhook dispatcher with DLQ and replay (P0, L, functional)

Description

Implement a reliable webhook delivery system with signature verification (HMAC), retry/backoff strategy, Dead Letter Queue (DLQ), and replay API to re-send failed events. Include monitoring and per-tenant webhook metrics.

Business value

Enables robust event-driven integrations and meets enterprise reliability expectations.

Acceptance criteria

- Webhook subscription endpoints available: `POST /webhooks/subscriptions` and management APIs
- Dispatcher signs payloads with HMAC and recipients can verify signature
- Failed deliveries are retried with exponential backoff and eventually moved to DLQ after configurable attempts
- DLQ exposes an API to list failed deliveries and replay them; replayed deliveries increment a replay counter
- Metrics exist for webhook success rate, retries, and DLQ size

Ambiguities

- Exact retry policy (max attempts, base delay) and DLQ retention period
- Message ordering guarantees for webhooks (per subscription)

User story

- As a SaaS product owner, I want reliable webhooks with DLQ so that my downstream systems remain in sync even with transient failures.

### REQ-007 — Jobs API and background workers (P1, M, functional)

Description

Provide `/jobs` endpoint to submit async workloads (reconciliation, bulk imports/exports, reporting). Implement worker processes to process queued jobs and store artifacts (logs, result files) in object storage.

Business value

- Enables asynchronous heavy-lift operations without blocking API; supports large imports and reconciliation required by customers.

Acceptance criteria

- POST `/jobs` returns `job_id` and status; GET `/jobs/{id}` returns current state
- Workers pick up queued jobs and update status atomically
- Artifacts are stored and retrievable via pre-signed URLs or authenticated endpoints
- Job logs are available and searchable for troubleshooting

Ambiguities

- Limits and quotas for job submissions per tier
- Retention policy for job artifacts

User story

- As an operator, I want to submit a bulk import job and get a downloadable artifact when complete so that I can migrate data efficiently.

### REQ-008 — Projection read stores and search endpoints (P1, L, performance)

Description

Build projection read-stores optimized for low-latency queries (POS lookups), and provide search endpoints (or integrate Meilisearch/Elasticsearch) for product discovery and POS queries.

Business value

Delivers sub-100ms reads for POS scenarios and supports flexible query patterns for admin UIs and storefronts.

Acceptance criteria

- POS read endpoint (e.g., GET `/pos/stock`) returns 95th percentile latency < 100ms under test load
- Search endpoint supports keyword, SKU, and filter based queries with pagination
- Projection update lag from ledger is documented and within acceptable SLA for MVP (e.g., <5s eventual consistency) or clarified otherwise
- Unit and load tests demonstrate projection correctness under concurrent writes

Ambiguities

- Target latency and consistency SLA definitions for different tiers
- Choice of search engine for MVP vs later (Meilisearch vs Elasticsearch)

User story

- As a retail POS integrator, I want sub-100ms stock lookups so that my checkout flow isn't impacted.

### REQ-009 — Sandbox tenant, quickstart, and example apps (P0, M, usability)

Description

Create a sandbox provisioning workflow that creates an isolated tenant with sample data, and provide a one-click quickstart that executes the first-10-minute integration script. Publish example apps for Node and Python demonstrating a full flow.

Business value

Dramatically improves activation and conversion by enabling developers to get first successful API call within 10 minutes.

Acceptance criteria

- Sandbox provisioning completes within 60 seconds for a new signup
- Quickstart script runs with provided sample keys and completes a create->read->webhook flow
- Example apps for Node.js and Python are available in `/examples` and installable/run locally
- Sandbox tenants are rate limited and separated from production billing

Ambiguities

- Sandbox data refresh cadence and whether sandbox supports full webhook delivery to public endpoints
- Limits for sandbox tenants (calls, webhooks, job runtime)

User story

- As a developer, I want a sandbox with sample data so I can try the API end-to-end without affecting production.

### REQ-010 — Metering, billing, and upgrade flow (P0, L, functional)

Description

Implement usage metering (API calls, webhook deliveries, job minutes) per tenant, billing records, invoice generation, and an upgrade flow that moves tenants between plan tiers. Expose billing dashboard and usage alerts.

Business value

Enables monetization, transparent billing, and self-serve upgrades for conversion growth.

Acceptance criteria

- Usage metering records API calls and relevant events with timestamp and tenant_id
- Billing dashboard shows current period usage and projected month-end cost
- Upgrade flow upgrades tenant and applies quotas immediately
- Invoices are generated for paid customers and stored for download
- Automated tests validate metering correctness for sample traffic patterns

Ambiguities

- Exact overage pricing and billing cadence (monthly, prorated?)
- Tax handling and payment provider choice

User story

- As a paying customer, I want to see my current usage and be able to upgrade plans so that I avoid service limits.

### REQ-011 — Nightly reconciliation job and reporting (P1, M, functional)

Description

Create a scheduled reconciliation job that validates stock_levels against aggregated stock_movements to detect and report discrepancies; provide exportable reconciliation reports and corrective action guidance.

Business value

Operational safety net that builds trust with retailers and reduces shrink/stock inaccuracies.

Acceptance criteria

- Scheduled job runs nightly and produces a report in CSV/JSON with discrepancy counts and details
- Reconciliation job exposes metrics and alerts when discrepancies exceed threshold
- Reports are accessible via GET `/reports/reconciliation?date=`
- Report includes suggested corrective operations (e.g., adjustments) with one-click job creation

Ambiguities

- Which thresholds trigger alerts and who receives them by default
- Retention and access control for reconciliation reports

User story

- As an operations manager, I want nightly reconciliation reports so I can identify mismatches and schedule fixes.

### REQ-012 — SDKs: Node.js and Python (v0.1) (P0, M, usability)

Description

Provide official SDKs for Node.js and Python that wrap authentication, retry logic, idempotency support, and helper functions for common flows (create item, append movement, subscribe webhook). Include basic tests and publish to npm/pypi.

Business value

Lowers integration friction and increases activation/conversion among developer builders.

Acceptance criteria

- npm package and PyPI package can be installed and used to perform an end-to-end quickstart
- SDKs implement authentication flows and include examples for idempotency and ETag usage
- Unit tests cover SDK basic flows and run in CI
- SDK README includes minimal example to get first API call in <10 minutes

Ambiguities

- Which exact helper abstractions to include initially (e.g., automatic idempotency key generation)
- Release cadence and compatibility guarantees

User story

- As a developer, I want an SDK so that I can integrate faster without writing raw HTTP calls.

### REQ-013 — Observability: metrics, tracing, logs, and billing telemetry (P1, M, non-functional)

Description

Instrument APIs, workers, and webhooks with metrics (Prometheus), distributed tracing (OpenTelemetry), structured logs (JSON), and billing telemetry to the dashboard. Provide Grafana dashboards or equivalent.

Business value

Enables debugging, platform reliability tracking, and supports enterprise due diligence (SRE expectations).

Acceptance criteria

- Key metrics (API latency, error rates, webhook success rate, job backlog, reconciliation mismatch rate) are emitted and viewable
- Traces can be correlated across API -> worker -> webhook flows for at least a sampled percentage (e.g., 10%)
- Billing telemetry emits usage events to billing system with <1s lag for near-real-time dashboards
- Dashboards are present and baseline alerts configured for high-error-rate and high-latency scenarios

Ambiguities

- Which metrics are required for MVP along with precise alert thresholds
- Retention period for logs and metrics

User story

- As an SRE, I want dashboards and alerts so I can act quickly during incidents.

### REQ-014 — Security foundations: TLS, secrets vault, RBAC, webhook signing (P0, M, security)

Description

Implement transport security (TLS), a secrets management approach, RBAC for the admin console, and HMAC signing for webhooks. Define and document security controls required for SOC2 readiness.

Business value

Reduces enterprise concerns and accelerates pilot conversions by demonstrating basic security hygiene.

Acceptance criteria

- All external endpoints require TLS (HTTPS) and reject insecure connections
- Secrets are encrypted at rest and not stored in plaintext in repos
- Admin console supports role-based access and at least two roles (admin, viewer)
- Webhook signatures are HMAC-SHA256 and documented for consumers

Ambiguities

- Which identity provider(s) are supported for SSO in MVP
- Exact encryption key management strategy for self-hosted vs managed plans

User story

- As a security officer, I want webhooks signed and admin RBAC so that only authorized users can manage tenants.

### REQ-015 — SOC2 readiness checklist and controls mapping (initial) (P2, M, non-functional)

Description

Create an initial SOC2 readiness plan that maps implemented controls to SOC2 requirements and identifies gaps. Include documentation artifacts and control owners.

Business value

Shortens enterprise procurement timelines and gives confidence to larger customers.

Acceptance criteria

- SOC2 control mapping document exists in `/compliance` and lists implemented controls and owners
- Gap list identifies at minimum 10 items and estimated remediation timeline
- At least one third-party counsel or auditor contact is identified for future audit guidance

Ambiguities

- Whether formal SOC2 Type 1 audit is required in the first 12 months
- Budget available for audits

User story

- As an enterprise buyer, I want evidence of security controls so that my procurement team can approve the vendor.

### REQ-016 — Multi-tenant data isolation and tenancy model (P0, L, non-functional)

Description

Design and implement a tenancy model that ensures logical isolation between tenants, enforces per-tenant quotas, and supports eventual migration to dedicated clusters for enterprise customers.

Business value

Protects customer data and enables charging and resource isolation per plan.

Acceptance criteria

- Tenants are isolated logically; queries always include tenant context and cannot leak across tenants
- Per-tenant quotas are enforced and violations produce clear error codes and notifications
- Migration path documented for moving a tenant to dedicated infrastructure

Ambiguities

- Whether storage isolation (separate DB schemas vs shared schema + tenant_id) is required for MVP
- Expectations for dedicated infra SLAs and costs

User story

- As a tenant admin, I want my data to be isolated from other tenants so that my operations remain secure.

### REQ-017 — Backup, PITR and disaster recovery (P1, M, operational)

Description

Implement backup strategy for Postgres (PITR), object storage backups, and documented DR runbooks to restore data within target RTO/RPO objectives.

Business value

Reduces risk of data loss and supports enterprise risk requirements.

Acceptance criteria

- Automated daily backups are taken and verified for integrity
- Point-in-time recovery tested quarterly and restores complete within documented RTO
- DR runbook exists and is versioned in the repo

Ambiguities

- Target RTO and RPO values for self-hosted vs managed tiers
- Retention period for backups

User story

- As an operations lead, I want tested backups and DR runbooks so that we can recover from incidents with minimal downtime.

### REQ-018 — Admin dashboard and tenant management console (P0, M, usability)

Description

Build a basic admin console for tenant onboarding, plan upgrades, billing overview and webhook subscription management. Include role-based admin access and activity logs.

Business value

Enables self-serve operations, decreases founder support load, and provides sales with visibility during pilots.

Acceptance criteria

- Admin console allows creating sandbox and production tenants, assigning plans, and viewing usage
- Activity log shows top-level tenant actions with timestamps and actor
- RBAC applied so only authorized users can perform billing and migrations

Ambiguities

- Exact roles and permissions needed for MVP
- UI requirements and framework choice

User story

- As the founder, I want to create and manage pilot tenants quickly so I can run pilots without heavy engineering involvement.

### REQ-019 — Deployment and runbooks for solo-founder self-hosted stack (P0, M, operational)

Description

Provide a reproducible deployment for a single-VM/VPS stack (reverse proxy, API, worker, Postgres, Redis, MinIO, search) along with runbooks for incidents, DB restore, and webhook DLQ handling.

Business value

Allows the founder to operate and support the platform with limited resources initially.

Acceptance criteria

- One-click (scripted) deployment to a fresh Ubuntu VM is documented and tested
- Runbooks for DB restore, incident triage, and webhook DLQ procedure are present
- Deployment produces monitoring endpoints and basic dashboards

Ambiguities

- Supported cloud providers for the initial deployment scripts
- Whether containers or system packages are preferred for MVP

User story

- As the founder, I want a documented deployment path so I can spin up a reproducible environment for pilots.

### REQ-020 — Integrations: POS and ecommerce connectors (skeleton) (P2, M, functional)

Description

Provide connector skeletons and integration guides for common POS systems and ecommerce platforms to accelerate partner integrations. Include sample adapters and mapping guidance.

Business value

- Speeds partner integrations and supports channel-led growth through easier onboarding for POS/ecommerce customers.

Acceptance criteria

- At least two connector skeletons (one POS, one ecommerce) are present in `/connectors` with README and sample mapping
- Integration guide outlines authentication, rate limits, and reconciliation steps
- Sample adapter demonstrates mapping between external order events and `stock_movements`

Ambiguities

- Which POS/ecommerce platforms to prioritize for connectors
- Level of completeness required for each connector in MVP

User story

- As a partner engineer, I want connector skeletons so I can build adapters faster for our customers.

### REQ-021 — Forecasting / Reorder AI (deferred to add-on) (P3, XL, functional)

Description

Design the forecasting/add-on architecture for future monetization (AI-driven reorder suggestions) but defer implementation to post-MVP as a paid add-on.

Business value

High-margin upsell to paying customers once core product is validated.

Acceptance criteria

- Architecture document exists describing data requirements and endpoints for future forecasting
- Prototype dataset and evaluation plan to validate algorithms
- Clearly marked as an add-on in pricing docs

Ambiguities

- Specific forecasting algorithms and expected accuracy targets
- Pricing model for AI add-on

User story

- As a growth customer, I want reorder suggestions so I can reduce stockouts and optimize inventory.

## Out of scope

- Full warehouse management features (e.g., advanced picking, packing, conveyor control) for MVP
- Physical IoT integrations (barcode scanners hardware drivers) in MVP
- Complete migration tooling for large ERPs in MVP (provide minimal import and connector skeletons)
- Native mobile apps and POS terminal software

## Assumptions

- Solo-founder will own initial development and operations until MRR signals hiring
- Initial infrastructure will be self-hosted on a single VM/VPS for MVP
- OpenAPI v1.1 and sandbox must be public and easy to access for developers
- Developer adoption will be driven by a generous free tier and good DX
- Enterprise-grade features (SSO, SOC2) can be phased after product-market fit

## Risks


- Skill gap: solo-founder lacks deep Rust/CQRS/ES experience
  - Impact: high
  - Mitigation: Start with simpler stack for MVP or hire contractors; split architecture into incremental deliverables.

- Scope creep: trying to build enterprise-grade features before validating product-market fit
  - Impact: high
  - Mitigation: Adopt phased roadmap; lock MVP scope and require explicit sign-off for scope additions.

- Underestimated infrastructure costs leading to cashflow stress
  - Impact: medium
  - Mitigation: Use conservative cost estimates, monitor infrastructure spend, and postpone managed services until justified.

- Slow developer adoption despite free tier
  - Impact: medium
  - Mitigation: Invest in DX: sample apps, quickstarts, RapidAPI listing, and developer outreach; run targeted experiments.

- Data integrity: reconciliation fails to detect/resolve drift in production
  - Impact: high
  - Mitigation: Automate reconciliation, surface alerts early, provide repair workflows and manual adjustment tools.

## Next steps and recommendations

- Review the ambiguities listed in each requirement and confirm choices for mutable design decisions (idempotency TTLs, tenant storage model, webhook retry policies).
- Convert P0 requirements into a sprint backlog for an initial 4–8 week phase focused on activation and metering.
- Run 20+ customer discovery interviews (developers and retail ops) before building enterprise features.
- Decide on MVP tech stack (Rust vs Node/Python) based on founder capabilities and hiring plan.
- Assign owners for P0 items and create acceptance-test suites to validate acceptance criteria automatically.


