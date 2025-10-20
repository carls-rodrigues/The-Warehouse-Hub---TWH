# Project overview

**Name:** The Warehouse Hub (TWH)  
**Purpose:** Developer-first, ledger-first inventory backend providing production-ready APIs for items, POs, SOs, stock ledger/snapshots, webhooks, jobs, and billing.  
**Primary goals:** Data correctness, fast developer onboarding, predictable performance, single-founder operational viability, seamless scale to enterprise.  
**Audience:** Developers, platform engineers, product managers, SREs, security/compliance reviewers, and sales/support teams.

---

## Current Development Status

**Sprint 2: Core Ledger Infrastructure** - *75% Complete*  
**Status:** Active development with 3 of 4 core tasks completed  
**Progress Report:** See [SPRINT_2_PROGRESS.md](SPRINT_2_PROGRESS.md) for detailed implementation status

**Completed Infrastructure:**
- ✅ Stock Management Ledger (TASK-004) - Immutable ledger with transactional snapshots
- ✅ Idempotency Middleware (TASK-005) - Dual-storage Redis/PostgreSQL resilience
- ✅ Search Indexing Pipeline (TASK-008) - Full-text search with automatic indexing

**Remaining Tasks:**
- 🔄 Stock Search API Endpoints (TASK-030)
- 🔄 Items Search Endpoint (TASK-031)

---

## Requirements and success criteria

## Functional requirements
- Public OpenAPI 3.0 contract implementing items, stock, purchase_orders, sales_orders, transfers, adjustments, webhooks, jobs, audit, reports, exports, auth.
- Idempotent POSTs via Idempotency-Key; optimistic concurrency via ETag / If-Match.
- Immutable stock_movements ledger and transactional stock_levels snapshots.
- Webhook subscriptions with HMAC signing, retry policy, DLQ and replay.
- Job API for long-running imports/exports with /jobs/{id} status.
- Multi-tenant scoping via X-Tenant-ID header and per-tenant quotas.

## Non-functional requirements
- Availability: tiered SLOs (Startup: 99.5%, Growth: 99.9%, Scale: 99.95+, Enterprise: custom).  
- Performance: tiered rate-limits (see Rate Limits doc); POS search latency <100ms for indexed queries.  
- Security: TLS everywhere, HMAC-signed webhooks, secrets in vault, JWT/API key auth, audit logs.  
- Correctness: stock_movements authoritative; nightly reconciliation detects and reports mismatches.  
- Observability: metrics, traces, logs, and dashboards with alerting.  
- Cost posture: support self-hosted minimal footprint initially; cloud-managed options for scale.

## Success metrics
- Activation rate (new signups -> first successful API call) > 60% in sandbox.  
- Time-to-first-order (from signup) < 1 hour for sample quickstart.  
- Monthly Active Developers (MAD) and MRR targets per pricing plan.  
- API error rate < 0.5% for paid tiers.  
- Webhook success rate > 99% for Growth/Scale tiers.

---

## Architecture and design

## High-level architecture
- API Gateway → Domain Services (Items, Stock, Orders, Webhooks, Jobs, Reports, Audit) → Command Service → Event Bus → Projections/Read Stores → Webhook Dispatcher → Object Storage/Artifacts → Admin Console.

**Backend runtime:** Primary language: **Rust** (memory safety, performance, small binaries). A minimal Rust scaffold using axum is included in `backend-rust/`.

## Core design choices
- Ledger-first inventory: every inventory-affecting action writes immutable StockMovement; StockLevels are snapshots updated transactionally with movements.  
- CQRS: separate command path (write correctness) from read projections (optimized queries, search).  
- Event-driven: Kafka (or durable queue) as durable log for events, replayable for reconciliation and read rebuilds.  
- Idempotency + ETag: persisted idempotency mapping for create requests; ETag/If-Match for optimistic concurrency.

## Data partitioning and scaling
- Shard stock_movements by tenant_id or location_id to ensure single-writer per shard where feasible.  
- Read replicas and separate search cluster for POS endpoints.  
- Reserved capacity or dedicated clusters for Scale/Enterprise tenants.

## Components and responsibilities (brief)
- API Gateway: auth, rate-limiting, TLS, request routing.  
- Domain Services: implement OpenAPI endpoints; stateless; horizontally scalable.  
- Command Service: atomic write of stock_movements + snapshot update; emit events.  
- Event Bus: durable events store; retention for replay.  
- Projections: update read stores (Postgres read replicas, Elasticsearch/Meilisearch).  
- Webhook Dispatcher: signed delivery with retries, DLQ, admin visibility.  
- Job Workers: process /jobs tasks and produce artifacts to object storage.  
- Metering/Billing: usage collection, quota enforcement, billing export.  
- Admin Console: tenant management, usage, webhooks, audit.

---

## API contract and developer experience

## OpenAPI & SDKs
- Single source OpenAPI 3.0 YAML (v1.1.0) published and versioned.  
- Official SDKs: Node and Python initial releases; include idempotency helper, ETag helper, and webhook verification utility.  
- Postman collection and “first 10 minutes” quickstart sample app.

## Key API behaviors
- Idempotency-Key header required for create POSTs (server persists mapping and returns identical response on retries).  
- ETag header returned on GET/POST responses; If-Match required for mutating requests to avoid lost updates; 412 on mismatch.  
- Error response: structured Error{code, message, details, request_id}.  
- Pagination: support page/per_page and cursor with CursorMeta.  
- include_movements query param on /stock to include recent movements (costly).

## Developer portal
- Sandbox environment with ephemeral tenants and sample data.  
- Metering dashboard showing usage, quota, projected month-end usage and one-click upgrade links.  
- Code samples for idempotency, ETag flows, and webhook verification.

---

## Deployment, CI/CD and infrastructure

## Minimal self-hosted stack (solo-founder start)
- Reverse proxy (NGINX/Traefik) for TLS and routing.  
- Single app host running API and worker processes in containers.  
- Postgres (managed or self-hosted), Redis (for rate-limit token buckets and idempotency cache), RabbitMQ (or single-node Kafka/Rust-based durable queue), MinIO for object storage, Meilisearch for search.  
- Backups: scheduled DB dumps + WAL archiving to object storage.

## Cloud recommended stack (prod)
- Kubernetes cluster (multi-AZ) with deployments per service, HPA; API Gateway (Ingress + WAF).  
- Managed Postgres primary + read replicas (PITR enabled).  
- Kafka cluster for event bus with replication factor >= 3.  
- Redis cluster with persistence for rate-limiter and idempotency mapping.  
- Object storage (S3); Elasticsearch / OpenSearch; Prometheus/Grafana.  
- Terraform modules and Helm charts for infra-as-code.

## CI/CD and release flow
- Branch-based CI: unit tests, contract tests (OpenAPI), integration tests in sandbox.  
- Deploy pipeline: build -> test -> push images -> deploy to staging -> contract tests -> canary to prod -> promote.  
- Feature flags for rolling out breaking changes and heavy features.

---

## Testing, observability and operations

## Testing strategy
- Unit tests for business rules and command validations.  
- Contract tests against OpenAPI (automated) ensuring API responses match schema.  
- Integration tests for idempotency flows, ETag/If-Match concurrency, webhook signature verification.  
- Reconciliation tests: automated replay and compare stock_levels vs stock_movements.  
- Load tests simulating tiered traffic (steady + burst) and webhook deliveries.  
- Chaos/injection tests for Redis/Kafka failures and projection rebuild scenarios.

## Observability
- Metrics: Prometheus metrics for request latency, error rate, webhook success, job backlog, reconciliation mismatches.  
- Tracing: OpenTelemetry propagated across services; sample traces for long transactions.  
- Logging: Structured JSON logs with request_id, tenant_id, user_id, and correlation IDs; central log store (ELK/Loki).  
- Dashboards: Grafana dashboards for SLOs, per-tenant throttles, webhook DLQ, job queue backlog.  
- Alerts: Alert rules (429 spike, webhook DLQ growth, queue backlog threshold, reconciliation mismatch rate) routed to PagerDuty/Slack.

## Runbooks (short list)
- Incident triage: identify tenant, circuit-break, capture request_ids, run reconciliation job, escalate per severity.  
- DB restore: PITR restore steps, rebuild projections from event log.  
- Webhook DLQ: inspect failures, replay, rotate secret if compromised.  
- On-call guide: severity levels, paging list, rollback steps, communication templates.

---

## Security, compliance and governance

## Authentication & authorization
- JWT + API keys; scopes per endpoint; admin RBAC for console; optional SAML/OIDC SSO for Enterprise.

## Secrets & keys
- Use secret manager (Vault) for all secrets; rotate webhook secrets and API keys on-demand; deliver subscription secret only on creation.

## Webhook security
- HMAC-SHA256 signature in X-TWH-Signature header; timestamps to mitigate replay; optional mutual TLS for enterprise.

## Data protection & privacy
- TLS in transit; encryption at rest for DB and object storage; tenant isolation; data retention and deletion policy documented in privacy docs.

## Compliance preparation
- Map controls for SOC2 readiness: logging, access reviews, change control, incident response, and vendor risk.  
- Prepare audit artifacts: structured audit logs, access history, configuration change history.

---

## Product roadmap, deliverables and diagram artifacts

## Immediate (0–4 weeks)
- Finalize and publish OpenAPI v1.1.0.  
- Implement idempotency store and ETag handling.  
- Deploy minimal self-hosted stack and sandbox.  
- Publish quickstart and Postman collection.

## Near term (4–12 weeks)
- Implement webhook dispatcher with HMAC signing, retry, DLQ.  
- Job queue and /jobs endpoints with workers and artifact storage.  
- Metering and quota enforcement; billing exporter stub.  
- SDKs v0.1 (Node, Python) and sample apps.

## Medium term (3–6 months)
- Sharding strategy for stock_movements; projection rebuild tooling; reconciliation automation.  
- Performance tuning and HA infra (Kafka cluster, Redis cluster).  
- Sales enablement: pricing page, RapidAPI listing, pilot onboarding flow.

## Diagrams to produce next (for documentation)
- One-page system architecture diagram (components + flows).  
- Sequence diagrams (PO receive, SO ship, webhook delivery, bulk import).  
- Deployment diagram (sandbox, staging, prod network boundaries).  
- Security diagram (secrets, ingress controls, audit flow).  
- Operational runbook flowchart (incident triage → remediation → postmortem).

---

## Governance, contributors and templates

## Roles and responsibilities
- Owner: Carlos — product, dev, ops.  
- Contributors: list contractors or future hires by role (Backend, DevOps, SRE, Docs, Sales).  
- Support: community-first initially, escalation to Owner; document SLA and escalation matrix as you onboard paying customers.

## Documentation templates to include in repo
- README.md with quickstart and architecture one-pager.  
- ARCHITECTURE.md with component descriptions and data flows.  
- API/OPENAPI.yaml (versioned) and CHANGELOG.md.  
- DEPLOYMENT.md with infra and deployment steps (Terraform/Helm snippets).  
- RUNBOOKS/ folder: INCIDENT.md, DB_RESTORE.md, WEBHOOK_DLQ.md, RECONCILIATION.md.  
- TESTING.md: how to run unit, contract, integration, load and reconciliation tests.  
- SECURITY.md: threat model summary, key rotation, secrets handling, SSO/SAML notes.  
- ROADMAP.md with milestones and prioritized backlog.

## Templates (examples)
- API change request template (breaking change checklist, migration guide).  
- Onboarding checklist for new tenant (keys, quotas, sandbox, sample data).  
- Incident report template for postmortem.

---
