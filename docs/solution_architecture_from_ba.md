# Solution Architecture derived from Business Analyst requirements

This document translates `docs/business_analyst_analysis.json` into a prescriptive architecture blueprint. It maps each requirement to architecture components, non-functional requirements, acceptance/validation steps, and technical mitigations.

---

## 1. System overview

TWH is a ledger-first, developer-focused inventory API that must provide an immutable write ledger (stock_movements), transactional snapshots (stock_levels), webhooks with DLQ & replay, idempotency/ETag concurrency controls, sandbox quickstarts, metering/billing, and multi-tenant isolation. The architecture below prioritizes correctness and developer experience while keeping initial operations feasible for a solo founder.

## 2. Pattern and rationale

Pattern: Hybrid — Modular Monolith that evolves into Microservices plus Event-driven CQRS. Reason: lower initial ops complexity, faster iteration, and ability to migrate components (write-path, webhook dispatcher, projections) into independently scalable microservices as load and team grow.

## 3. High-level tech choices

- Backend: Rust (Axum/Actix) for core write-path, Node.js for SDKs and DevEx tooling. Consider Node/Python for MVP if Rust expertise isn't available.
- Database: PostgreSQL for ACID transactions (ledger + snapshot atomicity) and JSONB metadata.
- Event bus: Kafka (managed in production); begin with lightweight queue for MVP if needed.
- Cache: Redis for rate-limiting and idempotency caches, with DB fallback.
- Search: Elasticsearch or Meilisearch for low-latency POS search.
- Object storage: S3/MinIO for artifacts and backups.
- Observability: Prometheus + Grafana, OpenTelemetry + Jaeger/Tempo, centralized logs (ELK/Loki).

## 4. Component mapping (how BA requirements are satisfied)

- REQ-001 (OpenAPI): Domain APIs + CI pipeline. Implement OpenAPI artifacts, CI contract tests, and expose Postman collection from `/openapi`.
- REQ-002 (Items CRUD): Domain APIs backed by Postgres read model; write-path goes through Command Service to ensure idempotency and ETag enforcement.
- REQ-003 (Ledger writes): Command Service ensures append-only `stock_movements` and transactional writes to `stock_levels` within shard-local DB TX.
- REQ-004 (Stock snapshot & reconciliation): Projection workers and nightly reconciliation jobs consume ledger to verify snapshots, emit reports and alerts.
- REQ-005 (Idempotency/ETag): API layer + Command Service persist idempotency keys in Redis with DB fallback; ETag computed from `stock_level` version/hash.
- REQ-006 (Webhooks): Webhook Dispatcher consumes Kafka events, signs payloads, retries with exponential backoff, writes DLQ entries and exposes replay API.
- REQ-007 (Jobs): Jobs Service accepts job submissions, workers process and store artifacts in S3; quota enforcement via Metering service.
- REQ-008 (Projection read stores & search): Projection Workers update read-optimized tables and search indexes; Search Service serves queries with p95 < 100ms target.
- REQ-009 (Sandbox & quickstart): Admin Console provisions sandbox tenants, seeds sample data, and provides quickstart scripts + SDK examples.
- REQ-010 (Metering & billing): Event-sourced metering pipeline consumes usage events, enforces quotas, and produces invoices and billing dashboard.
- REQ-011 (Reconciliation reporting): Nightly reconciliation pipeline produces CSV/JSON reports available via API and drive corrective jobs.
- REQ-012 (SDKs): Node/Python SDKs wrapping auth/idempotency helpers; publish to npm/pypi and run CI tests.
- REQ-013 (Observability): OpenTelemetry instrumentation, Prometheus metrics, Grafana dashboards and synthetic probes.
- REQ-014 (Security): JWT/API keys, RBAC for admin console, TLS everywhere, Vault for secrets, HMAC webhook signing.
- REQ-015 (SOC2 readiness): Controls mapping documented; audit artifacts stored under `/compliance`.
- REQ-016 (Multi-tenancy): Tenant context enforced across services; enforce quotas and isolation; migration path to dedicated infra per tenant.
- REQ-017 (DR/Backups): PITR Postgres, daily snapshots to S3, periodic restore drills, runbooks in repo.
- REQ-018 (Admin console): React UI connecting to Admin APIs, supporting onboarding, DLQ replay and billing management.
- REQ-019 (Deployment for founder): Docker Compose or k3s manifests and scripted deployment to Ubuntu VM; runbooks for restore and DLQ handling.
- REQ-020 (Connectors skeletons): `/connectors` repo templates and mapping guides; sample adapters demonstrate mapping to `stock_movements`.
- REQ-021 (Forecasting add-on): Design data interfaces and storage for future ML models; defer implementation.

## 5. Data model and shard strategy

Core tables: tenant, user, item, location, stock_movement (append-only), stock_level (composite PK), job, webhook_subscription. Shard key: `tenant_id` initially; escalate to `tenant_id+location_id` for extreme write volumes. Indexes: ensure (tenant_id, item_id, location_id, created_at) on stock_movement for replay efficiency.

## 6. Transactions, idempotency, and correctness

- Use DB transactions to append stock_movements and update stock_levels in the same TX within a shard.
- Implement outbox pattern by writing events to outbox table within same TX; publisher reads and publishes to Kafka.
- Idempotency: unique constraint on (tenant_id, idempotency_key) or conditional insert; Redis cache for performance with DB fallback.
- ETag: compute stable ETag from stock_level version (NUMERIC increment or cryptographic hash) and enforce If-Match semantics.

## 7. Non-functional mapping & targets (selected)

- Observability (REQ-013): traces+metrics+logs available; critical metrics visible within 1 minute. Support: instrumented services + Prometheus + Grafana.
- Activation (REQ-009): first successful API call within 10 minutes. Support: sandbox provisioning (<60s), quickstart scripts, SDK examples.
- Performance (REQ-008): search and POS reads p95 < 100ms. Support: projection read stores + dedicated search cluster + caches.
- Correctness (REQ-004): nightly reconciliation completes and flags discrepancies > threshold. Support: reconciliation jobs, alerting, repair jobs.
- Concurrency (REQ-005): ETag mismatch returns 412; idempotency dedupes retries with high confidence. Support: idempotency store and ETag enforcement.
- Billing accuracy (REQ-010): usage variance <1% vs recorded billing exports. Support: event-sourced metering and reconciliation tests.

## 8. Security & compliance

- All endpoints TLS 1.3; webhook HMAC-SHA256; vault-managed secrets; RBAC for admin; SSO for enterprise.
- CI runs SCA/SAST/DAST; schedule pentests prior to enterprise pilots and document controls for SOC2 readiness.

## 9. Scalability, resilience and DR patterns

- DB: partition then shard; read replicas; PgBouncer for pooling; PITR for recovery.
- Event bus: start with durable lightweight queue for MVP; adopt managed Kafka in production.
- Redis: multi-AZ with fallback strategies for idempotency and rate-limits.
- HA: multi-AZ deployments for RDS/Kafka/Redis; Kafka replication >=3; worker autoscaling by lag.
- DR: projection rebuild from outbox, PITR restores, nightly snapshots to S3, documented runbooks and quarterly drills.

## 10. Monitoring, testing & runbooks

- CI: unit tests, OpenAPI contract tests, integration tests for idempotency/ETag/webhooks, reconciliation integration tests.
- Load and chaos tests to verify graceful degradation under Redis/Kafka failures.
- Runbooks: INCIDENT.md, DB_RESTORE.md, WEBHOOK_DLQ.md, and onboarding SOPs in `/RUNBOOKS`.

## 11. Acceptance & validation checklist (mapped to BA acceptance criteria)

- REQ-001: OpenAPI artifacts in `/openapi`, Postman collection present, CI contract tests green.
- REQ-002: Items endpoints pass CRUD tests including ETag/If-Match behavior and pagination.
- REQ-003: stock_movements API persists immutable rows; attempts to modify movement fail.
- REQ-004: Nightly reconciliation runs produce reports and alerts when thresholds exceeded.
- REQ-005: Idempotency-key repeated requests produce single ledger entry; TTL and retention documented.
- REQ-006: Webhook deliveries retried and DLQ entries created; admin can replay failed deliveries.
- REQ-009: Sandbox provisioning <60s; quickstart executes create->read->webhook flow.
- REQ-010: Metering pipeline records usage; billing exports produced; upgrade applies quotas.

## 12. Technical risks and mitigation (prioritized)

1. Skill gap for Rust/CQRS — Mitigation: use Node/Python for MVP or hire contractors; restrict Rust to performance-critical modules.
2. DB hotspots — Mitigation: shard by tenant, single-writer-per-shard, monitor write latencies.
3. Event bus complexity — Mitigation: start with lightweight queue; adopt managed Kafka later, instrument consumer lag.
4. Redis outage affecting idempotency/rate limiting — Mitigation: DB fallback for idempotency; degrade non-essential features temporarily.
5. Metering inaccuracies — Mitigation: event-sourced metering, reconciliation tests and dispute flows.

## 13. Recommended immediate deliverables

- Docker Compose / k3s manifest for solo-founder deployment (priority: enable pilots quickly).
- OpenAPI CI contract tests and Postman collection automation.
- Sandbox provisioning script and quickstart example apps (Node/Python).
- Minimal Node-based SDK wrappers for quick activation (if Rust hiring uncertain).

---