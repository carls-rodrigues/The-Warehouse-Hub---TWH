-- Migration: Add tenant isolation with Row-Level Security
-- This migration adds tenant_id columns to all tenant-scoped tables and enables RLS

-- Add tier column to tenants table
ALTER TABLE tenants ADD COLUMN IF NOT EXISTS tier VARCHAR(50) NOT NULL DEFAULT 'FREE' CHECK (tier IN ('FREE', 'DEVELOPER', 'STARTUP', 'GROWTH', 'SCALE', 'ENTERPRISE'));
CREATE INDEX IF NOT EXISTS idx_tenants_tier ON tenants(tier);

-- Enable RLS on all tenant-scoped tables
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

-- Migration: Add tenant isolation with Row-Level Security
-- This migration adds tenant_id columns to all tenant-scoped tables and enables RLS

-- Add tenant_id columns to all tenant-scoped tables (nullable first)
ALTER TABLE items ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
ALTER TABLE locations ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
ALTER TABLE stock_movements ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
ALTER TABLE stock_levels ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
ALTER TABLE purchase_orders ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
ALTER TABLE purchase_order_lines ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
ALTER TABLE sales_orders ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
ALTER TABLE sales_order_lines ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
ALTER TABLE transfers ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
ALTER TABLE transfer_lines ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
ALTER TABLE returns ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
ALTER TABLE return_lines ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
ALTER TABLE webhooks ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
ALTER TABLE webhook_events ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
ALTER TABLE webhook_deliveries ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
ALTER TABLE search_indexes ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);
ALTER TABLE idempotency_keys ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenants(id);

-- Add tier column to tenants table
ALTER TABLE tenants ADD COLUMN IF NOT EXISTS tier VARCHAR(50) NOT NULL DEFAULT 'FREE' CHECK (tier IN ('FREE', 'DEVELOPER', 'STARTUP', 'GROWTH', 'SCALE', 'ENTERPRISE'));

-- Create indexes on tenant_id columns for performance
CREATE INDEX IF NOT EXISTS idx_items_tenant_id ON items(tenant_id);
CREATE INDEX IF NOT EXISTS idx_locations_tenant_id ON locations(tenant_id);
CREATE INDEX IF NOT EXISTS idx_stock_movements_tenant_id ON stock_movements(tenant_id);
CREATE INDEX IF NOT EXISTS idx_stock_levels_tenant_id ON stock_levels(tenant_id);
CREATE INDEX IF NOT EXISTS idx_purchase_orders_tenant_id ON purchase_orders(tenant_id);
CREATE INDEX IF NOT EXISTS idx_purchase_order_lines_tenant_id ON purchase_order_lines(tenant_id);
CREATE INDEX IF NOT EXISTS idx_sales_orders_tenant_id ON sales_orders(tenant_id);
CREATE INDEX IF NOT EXISTS idx_sales_order_lines_tenant_id ON sales_order_lines(tenant_id);
CREATE INDEX IF NOT EXISTS idx_transfers_tenant_id ON transfers(tenant_id);
CREATE INDEX IF NOT EXISTS idx_transfer_lines_tenant_id ON transfer_lines(tenant_id);
CREATE INDEX IF NOT EXISTS idx_returns_tenant_id ON returns(tenant_id);
CREATE INDEX IF NOT EXISTS idx_return_lines_tenant_id ON return_lines(tenant_id);
CREATE INDEX IF NOT EXISTS idx_webhooks_tenant_id ON webhooks(tenant_id);
CREATE INDEX IF NOT EXISTS idx_webhook_events_tenant_id ON webhook_events(tenant_id);
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_tenant_id ON webhook_deliveries(tenant_id);
CREATE INDEX IF NOT EXISTS idx_search_indexes_tenant_id ON search_indexes(tenant_id);
CREATE INDEX IF NOT EXISTS idx_idempotency_keys_tenant_id ON idempotency_keys(tenant_id);

-- Create composite indexes for common query patterns
CREATE INDEX IF NOT EXISTS idx_items_tenant_sku ON items(tenant_id, sku);
CREATE INDEX IF NOT EXISTS idx_locations_tenant_code ON locations(tenant_id, code);
CREATE INDEX IF NOT EXISTS idx_stock_movements_tenant_item_location ON stock_movements(tenant_id, item_id, location_id);
CREATE INDEX IF NOT EXISTS idx_stock_levels_tenant_item ON stock_levels(tenant_id, item_id);
CREATE INDEX IF NOT EXISTS idx_purchase_orders_tenant_po_number ON purchase_orders(tenant_id, po_number);
CREATE INDEX IF NOT EXISTS idx_sales_orders_tenant_so_number ON sales_orders(tenant_id, so_number);
CREATE INDEX IF NOT EXISTS idx_transfers_tenant_transfer_number ON transfers(tenant_id, transfer_number);
CREATE INDEX IF NOT EXISTS idx_returns_tenant_return_number ON returns(tenant_id, return_number);
CREATE INDEX IF NOT EXISTS idx_webhooks_tenant_created_by ON webhooks(tenant_id, created_by);

-- Create Row-Level Security policies
-- These policies ensure tenants can only access their own data

-- Items policy
CREATE POLICY tenant_items_policy ON items
    FOR ALL USING (tenant_id = current_setting('app.tenant_id')::UUID);

-- Locations policy
CREATE POLICY tenant_locations_policy ON locations
    FOR ALL USING (tenant_id = current_setting('app.tenant_id')::UUID);

-- Stock movements policy
CREATE POLICY tenant_stock_movements_policy ON stock_movements
    FOR ALL USING (tenant_id = current_setting('app.tenant_id')::UUID);

-- Stock levels policy
CREATE POLICY tenant_stock_levels_policy ON stock_levels
    FOR ALL USING (tenant_id = current_setting('app.tenant_id')::UUID);

-- Purchase orders policy
CREATE POLICY tenant_purchase_orders_policy ON purchase_orders
    FOR ALL USING (tenant_id = current_setting('app.tenant_id')::UUID);

-- Purchase order lines policy
CREATE POLICY tenant_purchase_order_lines_policy ON purchase_order_lines
    FOR ALL USING (tenant_id = current_setting('app.tenant_id')::UUID);

-- Sales orders policy
CREATE POLICY tenant_sales_orders_policy ON sales_orders
    FOR ALL USING (tenant_id = current_setting('app.tenant_id')::UUID);

-- Sales order lines policy
CREATE POLICY tenant_sales_order_lines_policy ON sales_order_lines
    FOR ALL USING (tenant_id = current_setting('app.tenant_id')::UUID);

-- Transfers policy
CREATE POLICY tenant_transfers_policy ON transfers
    FOR ALL USING (tenant_id = current_setting('app.tenant_id')::UUID);

-- Transfer lines policy
CREATE POLICY tenant_transfer_lines_policy ON transfer_lines
    FOR ALL USING (tenant_id = current_setting('app.tenant_id')::UUID);

-- Returns policy
CREATE POLICY tenant_returns_policy ON returns
    FOR ALL USING (tenant_id = current_setting('app.tenant_id')::UUID);

-- Return lines policy
CREATE POLICY tenant_return_lines_policy ON return_lines
    FOR ALL USING (tenant_id = current_setting('app.tenant_id')::UUID);

-- Webhooks policy
CREATE POLICY tenant_webhooks_policy ON webhooks
    FOR ALL USING (tenant_id = current_setting('app.tenant_id')::UUID);

-- Webhook events policy
CREATE POLICY tenant_webhook_events_policy ON webhook_events
    FOR ALL USING (tenant_id = current_setting('app.tenant_id')::UUID);

-- Webhook deliveries policy
CREATE POLICY tenant_webhook_deliveries_policy ON webhook_deliveries
    FOR ALL USING (tenant_id = current_setting('app.tenant_id')::UUID);

-- Search indexes policy
CREATE POLICY tenant_search_indexes_policy ON search_indexes
    FOR ALL USING (tenant_id = current_setting('app.tenant_id')::UUID);

-- Idempotency keys policy
CREATE POLICY tenant_idempotency_keys_policy ON idempotency_keys
    FOR ALL USING (tenant_id = current_setting('app.tenant_id')::UUID);

-- Jobs policy (already has tenant_id)
CREATE POLICY tenant_jobs_policy ON jobs
    FOR ALL USING (tenant_id = current_setting('app.tenant_id')::UUID);

-- Tenants policy - tenants can only see themselves (for API access)
CREATE POLICY tenant_tenants_policy ON tenants
    FOR ALL USING (id = current_setting('app.tenant_id')::UUID);

-- Users policy - this might need special handling for cross-tenant users
-- For now, allow all users to be visible (they might be shared across tenants)
-- This can be refined based on business requirements

-- Create a function to set the tenant context
CREATE OR REPLACE FUNCTION set_tenant_context(tenant_uuid UUID)
RETURNS VOID AS $$
BEGIN
    PERFORM set_config('app.tenant_id', tenant_uuid::TEXT, false);
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Create a function to get current tenant
CREATE OR REPLACE FUNCTION get_current_tenant_id()
RETURNS UUID AS $$
BEGIN
    RETURN current_setting('app.tenant_id')::UUID;
EXCEPTION
    WHEN OTHERS THEN
        RETURN NULL;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Create tenant quota tracking table
CREATE TABLE IF NOT EXISTS tenant_quotas (
    tenant_id UUID PRIMARY KEY REFERENCES tenants(id) ON DELETE CASCADE,
    max_items INTEGER NOT NULL DEFAULT 10000,
    max_locations INTEGER NOT NULL DEFAULT 100,
    max_webhooks INTEGER NOT NULL DEFAULT 10,
    max_api_calls_per_hour INTEGER NOT NULL DEFAULT 10000,
    max_storage_mb INTEGER NOT NULL DEFAULT 1000,
    current_items INTEGER NOT NULL DEFAULT 0,
    current_locations INTEGER NOT NULL DEFAULT 0,
    current_webhooks INTEGER NOT NULL DEFAULT 0,
    current_storage_mb INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Enable RLS on tenant quotas
ALTER TABLE tenant_quotas ENABLE ROW LEVEL SECURITY;

-- Tenant quotas policy
CREATE POLICY tenant_tenant_quotas_policy ON tenant_quotas
    FOR ALL USING (tenant_id = current_setting('app.tenant_id')::UUID);

-- Create indexes for tenant quotas
CREATE INDEX IF NOT EXISTS idx_tenant_quotas_tenant_id ON tenant_quotas(tenant_id);

-- Create API rate limiting table
CREATE TABLE IF NOT EXISTS api_rate_limits (
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    endpoint VARCHAR(255) NOT NULL,
    window_start TIMESTAMPTZ NOT NULL,
    request_count INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (tenant_id, endpoint, window_start)
);

-- Enable RLS on API rate limits
ALTER TABLE api_rate_limits ENABLE ROW LEVEL SECURITY;

-- API rate limits policy
CREATE POLICY tenant_api_rate_limits_policy ON api_rate_limits
    FOR ALL USING (tenant_id = current_setting('app.tenant_id')::UUID);

-- Create indexes for API rate limits
CREATE INDEX IF NOT EXISTS idx_api_rate_limits_tenant_endpoint ON api_rate_limits(tenant_id, endpoint);
CREATE INDEX IF NOT EXISTS idx_api_rate_limits_window ON api_rate_limits(window_start);

-- Create function to check and increment rate limit
CREATE OR REPLACE FUNCTION check_rate_limit(
    p_tenant_id UUID,
    p_endpoint VARCHAR(255),
    p_max_requests INTEGER,
    p_window_minutes INTEGER DEFAULT 60
)
RETURNS BOOLEAN AS $$
DECLARE
    window_start TIMESTAMPTZ;
    current_count INTEGER;
BEGIN
    window_start := date_trunc('minute', NOW()) - INTERVAL '1 minute' * ((EXTRACT(MINUTE FROM NOW()) % p_window_minutes));

    -- Get current count
    SELECT request_count INTO current_count
    FROM api_rate_limits
    WHERE tenant_id = p_tenant_id
      AND endpoint = p_endpoint
      AND window_start = window_start;

    IF current_count IS NULL THEN
        -- First request in this window
        INSERT INTO api_rate_limits (tenant_id, endpoint, window_start, request_count)
        VALUES (p_tenant_id, p_endpoint, window_start, 1);
        RETURN TRUE;
    ELSIF current_count < p_max_requests THEN
        -- Increment count
        UPDATE api_rate_limits
        SET request_count = request_count + 1
        WHERE tenant_id = p_tenant_id
          AND endpoint = p_endpoint
          AND window_start = window_start;
        RETURN TRUE;
    ELSE
        -- Rate limit exceeded
        RETURN FALSE;
    END IF;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Create function to update tenant storage usage
CREATE OR REPLACE FUNCTION update_tenant_storage_usage()
RETURNS TRIGGER AS $$
BEGIN
    -- This trigger will be attached to tables to track storage usage
    -- For now, just update the updated_at timestamp
    -- Actual storage calculation would be more complex and might be done via background jobs

    UPDATE tenant_quotas
    SET updated_at = NOW()
    WHERE tenant_id = COALESCE(NEW.tenant_id, OLD.tenant_id);

    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Create function to validate tenant quotas before insert/update
CREATE OR REPLACE FUNCTION validate_tenant_quotas()
RETURNS TRIGGER AS $$
DECLARE
    quota_record RECORD;
    current_count INTEGER;
BEGIN
    -- Get tenant quota
    SELECT * INTO quota_record
    FROM tenant_quotas
    WHERE tenant_id = NEW.tenant_id;

    IF quota_record IS NULL THEN
        -- Create default quota if not exists
        INSERT INTO tenant_quotas (tenant_id) VALUES (NEW.tenant_id);
        RETURN NEW;
    END IF;

    -- Check different quotas based on table
    CASE TG_TABLE_NAME
        WHEN 'items' THEN
            SELECT COUNT(*) INTO current_count
            FROM items
            WHERE tenant_id = NEW.tenant_id;

            IF current_count >= quota_record.max_items THEN
                RAISE EXCEPTION 'Item quota exceeded for tenant %', NEW.tenant_id;
            END IF;

        WHEN 'locations' THEN
            SELECT COUNT(*) INTO current_count
            FROM locations
            WHERE tenant_id = NEW.tenant_id;

            IF current_count >= quota_record.max_locations THEN
                RAISE EXCEPTION 'Location quota exceeded for tenant %', NEW.tenant_id;
            END IF;

        WHEN 'webhooks' THEN
            SELECT COUNT(*) INTO current_count
            FROM webhooks
            WHERE tenant_id = NEW.tenant_id;

            IF current_count >= quota_record.max_webhooks THEN
                RAISE EXCEPTION 'Webhook quota exceeded for tenant %', NEW.tenant_id;
            END IF;
    END CASE;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Add triggers for quota validation (will be applied after data migration)
-- These are commented out until we migrate existing data
/*
CREATE TRIGGER validate_items_quota
    BEFORE INSERT ON items
    FOR EACH ROW EXECUTE FUNCTION validate_tenant_quotas();

CREATE TRIGGER validate_locations_quota
    BEFORE INSERT ON locations
    FOR EACH ROW EXECUTE FUNCTION validate_tenant_quotas();

CREATE TRIGGER validate_webhooks_quota
    BEFORE INSERT ON webhooks
    FOR EACH ROW EXECUTE FUNCTION validate_tenant_quotas();
*/

-- Add triggers for storage usage tracking
CREATE TRIGGER update_storage_usage_items
    AFTER INSERT OR UPDATE OR DELETE ON items
    FOR EACH ROW EXECUTE FUNCTION update_tenant_storage_usage();

CREATE TRIGGER update_storage_usage_locations
    AFTER INSERT OR UPDATE OR DELETE ON locations
    FOR EACH ROW EXECUTE FUNCTION update_tenant_storage_usage();

CREATE TRIGGER update_storage_usage_webhooks
    AFTER INSERT OR UPDATE OR DELETE ON webhooks
    FOR EACH ROW EXECUTE FUNCTION update_tenant_storage_usage();