-- Data Migration: Populate tenant_id for existing data
-- This script should be run after the schema migration
-- It creates a default tenant and assigns all existing data to it

-- First, create a default tenant for existing data
-- This assumes you have a user with ID '550e8400-e29b-41d4-a716-446655440000' (from database_setup.sql)
INSERT INTO tenants (id, name, tenant_type, status, database_schema, created_by)
VALUES (
    '550e8400-e29b-41d4-a716-446655440001',
    'Default Tenant',
    'PRODUCTION',
    'ACTIVE',
    'default_tenant',
    '550e8400-e29b-41d4-a716-446655440000'
) ON CONFLICT (database_schema) DO NOTHING;

-- Set the tenant context for the migration
SELECT set_tenant_context('550e8400-e29b-41d4-a716-446655440001');

-- Update all existing tables with the default tenant_id
-- Note: These updates will fail if RLS is enabled, so run this before enabling RLS
-- or temporarily disable RLS for the migration

-- Disable RLS temporarily for migration
ALTER TABLE items DISABLE ROW LEVEL SECURITY;
ALTER TABLE locations DISABLE ROW LEVEL SECURITY;
ALTER TABLE stock_movements DISABLE ROW LEVEL SECURITY;
ALTER TABLE stock_levels DISABLE ROW LEVEL SECURITY;
ALTER TABLE purchase_orders DISABLE ROW LEVEL SECURITY;
ALTER TABLE purchase_order_lines DISABLE ROW LEVEL SECURITY;
ALTER TABLE sales_orders DISABLE ROW LEVEL SECURITY;
ALTER TABLE sales_order_lines DISABLE ROW LEVEL SECURITY;
ALTER TABLE transfers DISABLE ROW LEVEL SECURITY;
ALTER TABLE transfer_lines DISABLE ROW LEVEL SECURITY;
ALTER TABLE returns DISABLE ROW LEVEL SECURITY;
ALTER TABLE return_lines DISABLE ROW LEVEL SECURITY;
ALTER TABLE webhooks DISABLE ROW LEVEL SECURITY;
ALTER TABLE webhook_events DISABLE ROW LEVEL SECURITY;
ALTER TABLE webhook_deliveries DISABLE ROW LEVEL SECURITY;
ALTER TABLE search_indexes DISABLE ROW LEVEL SECURITY;
ALTER TABLE idempotency_keys DISABLE ROW LEVEL SECURITY;
ALTER TABLE jobs DISABLE ROW LEVEL SECURITY;

-- Populate tenant_id for existing data
UPDATE items SET tenant_id = '550e8400-e29b-41d4-a716-446655440001' WHERE tenant_id IS NULL;
UPDATE locations SET tenant_id = '550e8400-e29b-41d4-a716-446655440001' WHERE tenant_id IS NULL;
UPDATE stock_movements SET tenant_id = '550e8400-e29b-41d4-a716-446655440001' WHERE tenant_id IS NULL;
UPDATE stock_levels SET tenant_id = '550e8400-e29b-41d4-a716-446655440001' WHERE tenant_id IS NULL;
UPDATE purchase_orders SET tenant_id = '550e8400-e29b-41d4-a716-446655440001' WHERE tenant_id IS NULL;
UPDATE purchase_order_lines SET tenant_id = '550e8400-e29b-41d4-a716-446655440001' WHERE tenant_id IS NULL;
UPDATE sales_orders SET tenant_id = '550e8400-e29b-41d4-a716-446655440001' WHERE tenant_id IS NULL;
UPDATE sales_order_lines SET tenant_id = '550e8400-e29b-41d4-a716-446655440001' WHERE tenant_id IS NULL;
UPDATE transfers SET tenant_id = '550e8400-e29b-41d4-a716-446655440001' WHERE tenant_id IS NULL;
UPDATE transfer_lines SET tenant_id = '550e8400-e29b-41d4-a716-446655440001' WHERE tenant_id IS NULL;
UPDATE returns SET tenant_id = '550e8400-e29b-41d4-a716-446655440001' WHERE tenant_id IS NULL;
UPDATE return_lines SET tenant_id = '550e8400-e29b-41d4-a716-446655440001' WHERE tenant_id IS NULL;
UPDATE webhooks SET tenant_id = '550e8400-e29b-41d4-a716-446655440001' WHERE tenant_id IS NULL;
UPDATE webhook_events SET tenant_id = '550e8400-e29b-41d4-a716-446655440001' WHERE tenant_id IS NULL;
UPDATE webhook_deliveries SET tenant_id = '550e8400-e29b-41d4-a716-446655440001' WHERE tenant_id IS NULL;
UPDATE search_indexes SET tenant_id = '550e8400-e29b-41d4-a716-446655440001' WHERE tenant_id IS NULL;
UPDATE idempotency_keys SET tenant_id = '550e8400-e29b-41d4-a716-446655440001' WHERE tenant_id IS NULL;
UPDATE jobs SET tenant_id = '550e8400-e29b-41d4-a716-446655440001' WHERE tenant_id IS NULL;

-- Create default quota for the default tenant
INSERT INTO tenant_quotas (tenant_id) VALUES ('550e8400-e29b-41d4-a716-446655440001')
ON CONFLICT (tenant_id) DO NOTHING;

-- Re-enable RLS
ALTER TABLE items ENABLE ROW LEVEL SECURITY;
ALTER TABLE locations ENABLE ROW LEVEL SECURITY;
ALTER TABLE stock_movements ENABLE ROW LEVEL SECURITY;
ALTER TABLE stock_levels ENABLE ROW LEVEL SECURITY;
ALTER TABLE purchase_orders ENABLE ROW LEVEL SECURITY;
ALTER TABLE purchase_order_lines ENABLE ROW LEVEL SECURITY;
ALTER TABLE sales_orders ENABLE ROW LEVEL SECURITY;
ALTER TABLE sales_order_lines ENABLE ROW LEVEL SECURITY;
ALTER TABLE transfers ENABLE ROW LEVEL SECURITY;
ALTER TABLE transfer_lines ENABLE ROW LEVEL SECURITY;
ALTER TABLE returns ENABLE ROW LEVEL SECURITY;
ALTER TABLE return_lines ENABLE ROW LEVEL SECURITY;
ALTER TABLE webhooks ENABLE ROW LEVEL SECURITY;
ALTER TABLE webhook_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE webhook_deliveries ENABLE ROW LEVEL SECURITY;
ALTER TABLE search_indexes ENABLE ROW LEVEL SECURITY;
ALTER TABLE idempotency_keys ENABLE ROW LEVEL SECURITY;
ALTER TABLE jobs ENABLE ROW LEVEL SECURITY;

-- Make tenant_id columns NOT NULL after populating data
-- Note: This will fail if there are still NULL values
ALTER TABLE items ALTER COLUMN tenant_id SET NOT NULL;
ALTER TABLE locations ALTER COLUMN tenant_id SET NOT NULL;
ALTER TABLE stock_movements ALTER COLUMN tenant_id SET NOT NULL;
ALTER TABLE stock_levels ALTER COLUMN tenant_id SET NOT NULL;
ALTER TABLE purchase_orders ALTER COLUMN tenant_id SET NOT NULL;
ALTER TABLE purchase_order_lines ALTER COLUMN tenant_id SET NOT NULL;
ALTER TABLE sales_orders ALTER COLUMN tenant_id SET NOT NULL;
ALTER TABLE sales_order_lines ALTER COLUMN tenant_id SET NOT NULL;
ALTER TABLE transfers ALTER COLUMN tenant_id SET NOT NULL;
ALTER TABLE transfer_lines ALTER COLUMN tenant_id SET NOT NULL;
ALTER TABLE returns ALTER COLUMN tenant_id SET NOT NULL;
ALTER TABLE return_lines ALTER COLUMN tenant_id SET NOT NULL;
ALTER TABLE webhooks ALTER COLUMN tenant_id SET NOT NULL;
ALTER TABLE webhook_events ALTER COLUMN tenant_id SET NOT NULL;
ALTER TABLE webhook_deliveries ALTER COLUMN tenant_id SET NOT NULL;
ALTER TABLE search_indexes ALTER COLUMN tenant_id SET NOT NULL;
ALTER TABLE idempotency_keys ALTER COLUMN tenant_id SET NOT NULL;

-- Add foreign key constraints for tenant_id (already added during schema migration)
-- These ensure referential integrity

-- Update the jobs table to reference tenants properly
-- (jobs table already has tenant_id but might need FK constraint)
-- ALTER TABLE jobs ADD CONSTRAINT fk_jobs_tenant_id FOREIGN KEY (tenant_id) REFERENCES tenants(id);

-- Create a view for tenant usage statistics
CREATE OR REPLACE VIEW tenant_usage_stats AS
SELECT
    t.id as tenant_id,
    t.name as tenant_name,
    t.tenant_type,
    t.status,
    COUNT(DISTINCT i.id) as item_count,
    COUNT(DISTINCT l.id) as location_count,
    COUNT(DISTINCT w.id) as webhook_count,
    COUNT(DISTINCT po.id) as purchase_order_count,
    COUNT(DISTINCT so.id) as sales_order_count,
    COUNT(DISTINCT tr.id) as transfer_count,
    COUNT(DISTINCT r.id) as return_count,
    q.max_items,
    q.max_locations,
    q.max_webhooks,
    q.max_api_calls_per_hour,
    q.max_storage_mb,
    q.current_items,
    q.current_locations,
    q.current_webhooks,
    q.current_storage_mb
FROM tenants t
LEFT JOIN tenant_quotas q ON t.id = q.tenant_id
LEFT JOIN items i ON t.id = i.tenant_id
LEFT JOIN locations l ON t.id = l.tenant_id
LEFT JOIN webhooks w ON t.id = w.tenant_id
LEFT JOIN purchase_orders po ON t.id = po.tenant_id
LEFT JOIN sales_orders so ON t.id = so.tenant_id
LEFT JOIN transfers tr ON t.id = tr.tenant_id
LEFT JOIN returns r ON t.id = r.tenant_id
GROUP BY t.id, t.name, t.tenant_type, t.status, q.max_items, q.max_locations, q.max_webhooks,
         q.max_api_calls_per_hour, q.max_storage_mb, q.current_items, q.current_locations,
         q.current_webhooks, q.current_storage_mb;

-- Grant permissions on the view
GRANT SELECT ON tenant_usage_stats TO PUBLIC;