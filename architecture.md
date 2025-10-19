# Solution Architecture — The Warehouse Hub (TWH)

A single-source, production-ready Solution Architecture for The Warehouse Hub (TWH). This document is exhaustive and structured so engineering, DevOps, security, QA, product, and sales can implement, operate, and validate the system.

---

## 1 System overview and architecture pattern

### System overview
TWH is a developer-first, ledger-first inventory platform exposing a production-ready REST API, reliable webhooks, and async job processing. Every inventory mutation is recorded as an immutable ledger (stock_movements) and reflected in transactional snapshots (stock_levels). The system supports multi-tenant isolation, tiered quotas/rate-limits, idempotent flows, optimistic concurrency (ETag / If-Match), event-driven projections, nightly reconciliation, billing/metering, and an admin DevEx console. Designed to start self-hosted for a solo founder and evolve to cloud-managed multi-AZ production with sharding and enterprise features.

### Architecture pattern
Hybrid: Modular Monolith → Microservices, Event-driven CQRS.

### Rationale
- Start modular to reduce ops overhead for solo founder; split services later as load and team grow.  
- Event-driven CQRS isolates write (ledger-first correctness) from read (projections/search), enables reliable webhooks, replayable projections for reconciliation, and decoupled scaling.

---

## 2 Tech stack (recommended)

### Frontend
- Framework: **React + TypeScript**  
- Rationale: strong ecosystem for admin DevEx, quick component development, good tooling.  
- Key libraries: React Router; React Query (TanStack); Chakra UI or TailwindCSS.

### Backend
- Primary language: **Go** (performance, small binaries, concurrency)  
- Secondary tooling: **Node.js** for SDK generation and DevEx utilities (Express/Fastify where needed)  
- Frameworks: Gin (Go); lightweight Node frameworks for tooling.

### Datastores & Indexing
- Primary DB: **PostgreSQL** (ACID, transactional guarantees, JSONB).  
- Cache & fast counters: **Redis or Dragonfly** (token-bucket rate-limiter, idempotency cache, locks).  
- Event bus: **Kafka** (or managed equivalent) for durable, replayable events and outbox publishing.  
- Search: **Elasticsearch** or **Meilisearch** for low-latency POS search.  
- Object storage: **S3** / **MinIO** for job artifacts and backups.

### Infrastructure & hosting
- Initial: self-hosted single VM / VPS (Docker Compose or k3s) for founder.  
- Production: **AWS** recommended (EKS, RDS Postgres, MSK/Aiven Kafka, ElastiCache Redis, S3, Route53, CloudFront).  
- CI/CD: GitHub Actions for CI; ArgoCD / Flux for GitOps CD.

### Observability & security
- Metrics: Prometheus + Grafana.  
- Tracing: OpenTelemetry + Jaeger/Tempo.  
- Logging: ELK or Loki (structured JSON).  
- Secrets: Vault or cloud secrets manager.  
- Alerts: PagerDuty / OpsGenie; Slack for ops channels.

---

## 3 System components (responsibilities, interfaces, scaling)

Each component summary includes responsibilities, public/internal interfaces, dependencies, and scaling guidance.

### API Gateway (edge)
- Responsibilities: TLS termination, WAF, auth (JWT/API keys), per-key & per-tenant rate limiting, routing, header enrichment (ETag, X-RateLimit-*), canary routing, logging.  
- Interfaces: proxies all public and admin endpoints; exposes /health and /metrics.  
- Dependencies: Auth Service, Redis (rate counters), Logging/Tracing.  
- Scaling: horizontal stateless; use managed gateway or envoy/traefik in k8s.

### Auth Service
- Responsibilities: issue/validate JWTs, manage API keys, key rotation, sandbox token issuance, SSO/SAML for Enterprise, RBAC enforcement.  
- Interfaces: POST /auth/login, POST /auth/refresh, POST /api-keys/rotate.  
- Dependencies: User/Tenant DB, Vault.  
- Scaling: horizontal; cache key lookups in Redis.

### Domain APIs (Items, POs, SOs, Transfers, Returns)
- Responsibilities: implement OpenAPI endpoints, validation, orchestration; call Command Service for inventory writes.  
- Interfaces: GET/POST /items, /purchase_orders, /sales_orders, /transfers, etc.  
- Dependencies: Command Service, Postgres, Auth.  
- Scaling: stateless horizontal.

### Command Service (Stock write-path)
- Responsibilities: append immutable stock_movements, update stock_levels snapshot transactionally within shard/local DB tx, persist idempotency mapping, write outbox entries.  
- Interfaces: internal RPC/gRPC invoked by Domain APIs.  
- Dependencies: Primary DB (Postgres), Redis (locks/cache), Event outbox.  
- Scaling: single-writer-per-shard; scale by adding shards (tenant_id or tenant+location_id).

### Outbox Publisher & Event Bus
- Responsibilities: read outbox table (transactional), publish to Kafka topics (domain.events, webhook.deliveries), ensure at-least-once semantics.  
- Interfaces: outbox read; Kafka publish.  
- Dependencies: Postgres, Kafka.  
- Scaling: worker pool; partitions mapped to shards.

### Projection Workers / Read Services (CQRS)
- Responsibilities: consume domain events; update read-optimized stores (read replicas, denormalized tables) and search indexes.  
- Interfaces: Kafka consumers; write to read DB and search.  
- Dependencies: Kafka, read DBs, search index.  
- Scaling: consumer groups, parallel by partition.

### Search Service
- Responsibilities: handle /items/search; support fuzzy search and exact SKU/barcode; ensure low-latency responses.  
- Interfaces: /items/search; indexing pipeline from projection workers.  
- Dependencies: Search cluster (ES/Meilisearch).  
- Scaling: dedicated search nodes; tune refresh intervals.

### Webhook Dispatcher
- Responsibilities: sign payloads (HMAC-SHA256 with subscription secret), enqueue deliveries, retry with exponential backoff + jitter, DLQ, admin replay.  
- Interfaces: Kafka subscription; POST to subscriber URLs; admin DLQ APIs.  
- Dependencies: Kafka, Redis (per-subscription rate limits), Vault (secrets), DLQ storage (Postgres/S3).  
- Scaling: worker pool; quarantine failing subscribers.

### Jobs Service & Workers
- Responsibilities: accept /jobs, orchestrate long-running tasks (imports/exports), stream progress, produce artifact results to object storage.  
- Interfaces: POST /jobs, GET /jobs/{id}; workers consume job queue.  
- Dependencies: Object Storage, Kafka/queue, Postgres.  
- Scaling: autoscale workers by queue depth; priority queues for paid tenants.

### Metering & Billing Service
- Responsibilities: collect usage events, enforce quotas, expose usage dashboard and billing exports.  
- Interfaces: consume events; admin APIs for billing exports; gateway quota enforcement hooks.  
- Dependencies: Kafka, Redis counters, Postgres billing tables.  
- Scaling: partitioned by tenant; offline batch aggregation.

### Admin / DevEx Console
- Responsibilities: onboarding, key management, usage dashboards, upgrade UX, webhook management, DLQ replay UI, audit viewer, quickstarts.  
- Interfaces: REST admin APIs; React UI.  
- Dependencies: Auth, Metering, Postgres, Logging.  
- Scaling: stateless UI + API backends; cache heavy endpoints.

### Audit & Logs Store
- Responsibilities: persist immutable audit records for all writes; centralize structured logs.  
- Interfaces: Audit write/read API.  
- Dependencies: Postgres (append-only), ELK/Loki.  
- Scaling: append-only schemas, TTL/archival to S3.

### Monitoring & Observability
- Responsibilities: metrics, traces, dashboards, SLOs, alerts.  
- Interfaces: Prometheus scrape endpoints, trace ingestion endpoints, Grafana dashboards.  
- Dependencies: All services instrumented with OpenTelemetry.  
- Scaling: separate monitoring cluster or managed service for large retention.

---

## 4 Data model (core entities & relationships)

- tenant: id (UUID PK), name, billing_plan, created_at.  
- user: id, tenant_id (FK), email, role, password_hash (nullable), created_at.  
- item: id, tenant_id, sku, name, unit, cost_price, sale_price, metadata (JSONB), created_at, updated_at.  
- location: id, tenant_id, name, type (warehouse/store/drop-ship).  
- stock_movement: id, tenant_id, item_id, location_id, change_qty, type (RECEIVE/SALE/RESERVE/ADJUST/TRANSFER), ref_type, ref_id, user_id, metadata, idempotency_key, created_at. (Immutable append-only)  
- stock_level: composite PK (tenant_id, item_id, location_id), qty_on_hand, qty_reserved, last_counted_at, updated_at, e_tag. (Transactional snapshot updated with movements)  
- purchase_order: id, tenant_id, po_number, status, created_by, created_at.  
- job: job_id, tenant_id, type, status (QUEUED/RUNNING/SUCCESS/FAILED/PARTIAL), result_url, errors (JSONB), timestamps.  
- webhook_subscription: id, tenant_id, url, events (JSON), secret_encrypted, created_at.

Indexes: tenant_id on all tenant-scoped tables, item indexes on sku and name, stock_movement on (tenant_id, item_id, location_id, created_at) for replay performance.

Shard key recommendation: tenant_id initially; consider tenant_id + location_id for very high-volume tenants.

---

## 5 API specification (high-level conventions & critical endpoints)

Base URL: https://api.thewarehousehub.com/v1  
Authentication: JWT Bearer token; API keys for machine-to-machine.  
Common headers:
- Idempotency-Key: required for POSTs that create/mutate state; retained (default 30d).  
- If-Match: required for mutating requests that rely on optimistic concurrency.  
- X-Tenant-ID: optional header to scope request when not implicit from key.  
- X-TWH-Signature: HMAC-SHA256 signature on webhook deliveries.

Representative endpoints:
- POST /auth/login — issue JWT (sandbox flows)  
- GET /items — list/paginate; supports filters and cursor pagination  
- POST /items — create item (requires Idempotency-Key) — returns ETag  
- POST /purchase_orders/{id}/receive — receive PO lines; transactional stock_movement writes and snapshot update; emits po.received  
- POST /sales_orders/{id}/ship — write SALE stock_movement(s) and emit order.shipped  
- POST /webhooks/register — returns subscription id and one-time secret  
- POST /jobs — enqueue job (import/export), returns job_id (202)  
- GET /jobs/{jobId} — job status and result_url

Errors: structured Error { code, message, details, request_id }. Use HTTP 429 for quota and rate-limit, 412 for ETag mismatch, 409 for conflicts, 401/403 for auth/authorization.

Rate limits: enforced per-key and per-tenant via token-bucket; return headers X-RateLimit-Limit, X-RateLimit-Remaining, X-RateLimit-Reset, Retry-After when needed.

---

## 6 Transactions, concurrency and correctness guarantees

- Ledger-first: stock_movement is authoritative; never mutate historical rows (except correction metadata).  
- Atomic writes: append stock_movement(s) and update stock_level in same DB transaction within shard to avoid drift.  
- Idempotency: unique constraint (tenant_id, idempotency_key) or write-if-not-exists pattern; same key + different payload -> IDEMPOTENCY_KEY_MISMATCH.  
- ETag/If-Match: strong ETag (hash of version/updated_at) recommended; server returns 412 on mismatch.  
- Outbox pattern: write event to outbox table inside same transaction; publisher reads outbox and publishes to Kafka to guarantee event DB/Bus atomicity.  
- Reconciliation: nightly replay per tenant/shard to detect mismatches; prioritized reports; remediation jobs optionally auto-run or require manual approval.

---

## 7 Security architecture

- Authentication: JWT access tokens; API keys for M2M; SSO (SAML/OIDC) for Enterprise admin.  
- Authorization: RBAC (Admin, Dev, ReadOnly, Billing) and fine-grained API scopes (read:stock, write:orders).  
- Webhook security: HMAC-SHA256 signature (X-TWH-Signature) with timestamp and tolerance window (default 5 minutes); optional mutual TLS for enterprise.  
- Secrets: vault/cloud secret manager; webhook subscription secret shown once on creation and stored encrypted.  
- Transport & storage: TLS 1.3 for transport; AES-256 at rest using KMS-managed keys.  
- API security: input validation (OpenAPI schema validation), rate limiting, WAF on gateway, CORS restrictions for admin UI.  
- Logging & audit: immutable audit records for all writes (tenant_id, user_id, ip, user_agent, action, old/new values, request_id, timestamp).  
- Vulnerability & compliance: SCA/SAST/DAST in CI; scheduled pentests and SOC2 readiness mapping.

---

## 8 Scalability, resilience and DR

### Database & storage
- Start: single Postgres with logical partitions per tenant.  
- Scale: add read replicas for reads; add sharding (physical Postgres instances) by tenant_id or tenant+location_id for write scale; use WAL/PITR for backups and point-in-time recovery.  
- Object storage: S3 for artifacts and snapshot archives.

### Event bus & projections
- Kafka for durable, replayable events; managed Kafka recommended in production.  
- Projection workers rebuild read models via replay; store consumer offsets for checkpointing.

### API & workers
- API pods stateless and autoscaled; HPA by RPS/CPU.  
- Worker autoscaling by queue/Kafka lag; priority processing for paid tenants.

### Rate-limiting and QoS
- Token-bucket per-key and per-tenant in Redis; reserved token pools for Scale/Enterprise tenants; throttle low-tier tenants under pressure.

### High-availability & fault domains
- Multi-AZ deployments for Postgres, Kafka, Redis; Kafka replication factor ≥ 3.  
- Circuit-break low-tier tenants and degrade non-essential features under platform pressure (e.g., include_movements flag).  
- Implement fallbacks: local conservative limits when Redis unavailable; DB-backed idempotency lookup for correctness.

### Disaster recovery
- PITR-enabled Postgres; daily snapshots to S3; tested restore drills quarterly.  
- Outbox replay and projection rebuild scripts to reconstruct read stores.  
- Defined RTO/RPO per tier (set per SLA): e.g., Growth RTO ≤ 2 hrs, RPO ≤ 15 min; Free tier relaxed.

---

## 9 Observability, testing and operational controls

### Observability
- Tracing: OpenTelemetry across services, traces to Jaeger/Tempo.  
- Metrics: Prometheus metrics for latency histograms, error counts, 429s, queue lag, webhook success rate, reconciliation mismatch rate.  
- Dashboards: Grafana per-service SLO dashboards and per-tenant usage panels.  
- Logs: Structured JSON logs with request_id, tenant_id, user_id; central ELK/Loki.

### Testing
- Unit tests for business logic.  
- Contract tests (OpenAPI-based) run in CI for every PR.  
- Integration tests for idempotency, ETag flows, webhook verification, job lifecycle.  
- Reconciliation tests: replay stock_movements into clean DB and assert snapshots.  
- Load tests: simulate tiered traffic patterns (steady + bursts).  
- Chaos tests: simulate Redis/Kafka failures and validate graceful degradation.

### Runbooks (must-have)
- Incident triage: identify tenant, capture request_ids, circuit-break, run reconciliation, rollback if necessary.  
- DB restore: step-by-step PITR restore, outbox replay, projection rebuild verification.  
- Webhook DLQ recovery: inspect failures, rotate secrets if compromised, manual replay.  
- On-call: severity matrix, escalation list, communications templates.

---

## 10 CI/CD, deployment and rollback

### Environments
- development (local + docker-compose), sandbox (ephemeral tenants), staging (prod-like), production (multi-AZ).

### CI/CD
- CI: GitHub Actions — lint, unit tests, OpenAPI contract tests.  
- CD: build artifacts → image registry (ECR/GCR) → ArgoCD/Flux deploys to k8s.  
- Canary releases: route % traffic to canary; observe SLOs for observation window and auto-rollback on regression.

### Rollback strategy
- Canary / Blue-Green deployments; automated rollback on SLO breach (latency/error/429 thresholds).

---

## 11 Operational cost posture & deployment choices

### Solo-founder start (cost conscious)
- Single VM / small VPS with docker-compose or lightweight k3s: API + workers, Postgres (managed or local), Redis, RabbitMQ or lightweight Kafka, MinIO, Meilisearch.  
- Backup to S3-compatible; basic Prometheus + Grafana.  
- Monitor spend and plan migration to managed services when MRR justifies.

### Production (managed)
- AWS recommended: RDS (Postgres), MSK/Aiven Kafka, ElastiCache Redis, EKS, S3, CloudFront.  
- Use managed services to reduce ops burden and support enterprise SLAs.

---

## 12 Architecture decisions (summary)

1. PostgreSQL primary store — ACID guarantees, JSONB, strong integrity.  
2. Event-driven with Kafka + transactional outbox — required for replayability and decoupling.  
3. Modular monolith → microservices evolution — minimize early ops complexity.  
4. Redis for token-bucket rate limiting and idempotency caching — low-latency counters.  
5. Start self-hosted, migrate to AWS managed services as scale/cost allows.

---

## 13 Technical risks & mitigations

- Database hotspot under high write load — mitigation: shard by tenant/location, single-writer-per-shard, write batching.  
- Kafka operational complexity and lag — mitigation: start with lightweight queue, adopt managed Kafka for production, instrument consumer lag.  
- Redis unavailability affecting rate-limits and idempotency — mitigation: local fallback, DB-backed idempotency lookup, multi-AZ Redis cluster.  
- Incorrect metering — mitigation: event-sourced metering pipeline, reconciliation tests, transparent usage dashboards.  
- Outbox publish failures leading to missed events — mitigation: outbox repair worker, idempotent publishing, nightly reconciliation.

---

## 14 Non-functional requirements mapping (examples)

- REQ-015 Observability → OpenTelemetry + Prometheus + Grafana.  
- REQ-009 Performance (rate-limit behavior) → Redis token-bucket + API Gateway enforcement.  
- REQ-008 Latency (search p95 < 100ms) → dedicated search cluster + optimized indexing.  
- REQ-020 Resilience (DR & restore) → PITR Postgres, snapshots, outbox replay.

---

## 15 Diagrams (text mermaid blocks to embed)

- System architecture (component): flowchart shows API Gateway → Domain Services → Command Service → Postgres/outbox → Kafka → Webhooks/Projections → Search/Object Store → Admin/Metering.  
- Data flow (sequence): Purchase order receive — API → PO → Command Service → DB TX → outbox → Kafka → Webhook dispatcher.  
- Deployment: Internet → ALB/WAF → K8s ingress → API pods & worker pods → Postgres primary + replicas, Kafka cluster, Redis cluster, ES, S3, Vault.

(Embed mermaid blocks from prior docs into repo docs/diagrams/ as needed.)

---

## 16 Acceptance & validation checklist

- OpenAPI spec published and validated; contract tests pass.  
- Idempotency and ETag behavior implemented and covered by tests.  
- stock_movements and stock_levels atomicity demonstrated via integration tests.  
- Outbox writing and event publishing verified; projection rebuild works in staging.  
- Webhook signing and DLQ/replay flow works end-to-end.  
- Metering, quota enforcement, and billing export are producing expected outputs and dashboards.  
- Monitoring, tracing, and alerting configured and synthetic probes trigger expected alerts.  
- DR restore walk-through executed in staging and documented.

---

## 17 Next-step deliverables (pick one to generate immediately)

- Terraform skeleton for minimal production footprint (RDS, EKS, ElastiCache, S3).  
- Docker Compose / k3s manifest for solo-founder self-hosted deployment.  
- PlantUML or mermaid diagrams exported as PNG/SVG for README.  
- Sprint backlog (2-week sprints) converting PRD requirements into Epics/Stories with acceptance tests.  
- Runbooks: INCIDENT.md, DB_RESTORE.md, WEBHOOK_DLQ.md.

---

This document is the canonical Solution Architecture for TWH. Tell me which deliverable above you want produced next and I will generate it as a ready-to-commit artifact (Terraform, Helm, docker-compose, mermaid diagrams, runbooks, or sprint backlog).