-- Database setup for The Warehouse Hub
-- Run this against your PostgreSQL database to set up the initial schema

-- Create database (run this as superuser if needed)
-- CREATE DATABASE twh;

-- Connect to the twh database and run the following:

-- Users table for authentication
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_active ON users(active);

-- Tenants table for multi-tenancy
CREATE TABLE IF NOT EXISTS tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    tenant_type VARCHAR(50) NOT NULL CHECK (tenant_type IN ('PRODUCTION', 'SANDBOX')),
    status VARCHAR(50) NOT NULL DEFAULT 'PROVISIONING' CHECK (status IN ('PROVISIONING', 'ACTIVE', 'SUSPENDED', 'DELETING')),
    database_schema VARCHAR(100) NOT NULL UNIQUE,
    created_by UUID REFERENCES users(id),
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for tenants
CREATE INDEX IF NOT EXISTS idx_tenants_tenant_type ON tenants(tenant_type);
CREATE INDEX IF NOT EXISTS idx_tenants_status ON tenants(status);
CREATE INDEX IF NOT EXISTS idx_tenants_database_schema ON tenants(database_schema);
CREATE INDEX IF NOT EXISTS idx_tenants_expires_at ON tenants(expires_at);

-- Items table for inventory management
CREATE TABLE IF NOT EXISTS items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sku VARCHAR(100) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(100),
    unit VARCHAR(50) NOT NULL,
    barcode VARCHAR(100),
    cost_price DOUBLE PRECISION NOT NULL CHECK (cost_price >= 0),
    sale_price DOUBLE PRECISION CHECK (sale_price >= 0),
    reorder_point INTEGER CHECK (reorder_point >= 0),
    reorder_qty INTEGER CHECK (reorder_qty >= 0),
    weight DOUBLE PRECISION CHECK (weight >= 0),
    dimensions JSONB,
    metadata JSONB,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for items
CREATE INDEX IF NOT EXISTS idx_items_sku ON items(sku);
CREATE INDEX IF NOT EXISTS idx_items_name ON items(name);
CREATE INDEX IF NOT EXISTS idx_items_category ON items(category);
CREATE INDEX IF NOT EXISTS idx_items_active ON items(active);
CREATE INDEX IF NOT EXISTS idx_items_created_at ON items(created_at);

-- Locations table for inventory locations
CREATE TABLE IF NOT EXISTS locations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    code VARCHAR(100) UNIQUE,
    address JSONB,
    type VARCHAR(50) CHECK (type IN ('warehouse', 'store', 'drop-ship')),
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for locations
CREATE INDEX IF NOT EXISTS idx_locations_code ON locations(code);
CREATE INDEX IF NOT EXISTS idx_locations_name ON locations(name);
CREATE INDEX IF NOT EXISTS idx_locations_type ON locations(type);
CREATE INDEX IF NOT EXISTS idx_locations_active ON locations(active);
CREATE INDEX IF NOT EXISTS idx_locations_created_at ON locations(created_at);

-- Insert some test locations for development
INSERT INTO locations (id, name, code, address, type, active)
VALUES
    (
        '550e8400-e29b-41d4-a716-446655440010',
        'Main Warehouse',
        'MAIN-WH',
        '{"line1": "123 Industrial Blvd", "city": "Springfield", "region": "IL", "postal_code": "62701", "country": "US"}',
        'warehouse',
        true
    ),
    (
        '550e8400-e29b-41d4-a716-446655440011',
        'Downtown Store',
        'DOWNTOWN',
        '{"line1": "456 Main St", "city": "Springfield", "region": "IL", "postal_code": "62701", "country": "US"}',
        'store',
        true
    ) ON CONFLICT (code) DO NOTHING;

-- Insert some test items for development
INSERT INTO items (id, sku, name, description, category, unit, barcode, cost_price, sale_price, reorder_point, reorder_qty, weight, dimensions, metadata, active)
VALUES
    (
        '550e8400-e29b-41d4-a716-446655440001',
        'WIDGET-001',
        'Standard Widget',
        'A standard widget for testing purposes',
        'Widgets',
        'Each',
        '1234567890123',
        10.50,
        15.99,
        50,
        100,
        0.250,
        '{"length": 10.0, "width": 5.0, "height": 2.0}',
        '{"supplier": "Test Supplier", "lead_time_days": 7}',
        true
    ),
    (
        '550e8400-e29b-41d4-a716-446655440002',
        'GADGET-001',
        'Advanced Gadget',
        'An advanced gadget with special features',
        'Gadgets',
        'Each',
        '1234567890124',
        25.00,
        39.99,
        25,
        50,
        0.500,
        '{"length": 15.0, "width": 8.0, "height": 3.0}',
        '{"supplier": "Premium Supplier", "lead_time_days": 14}',
        true
    ) ON CONFLICT (sku) DO NOTHING;

-- Insert a test user for development
-- Password is 'password' hashed with bcrypt
INSERT INTO users (id, email, password_hash, first_name, last_name, active)
VALUES (
    '550e8400-e29b-41d4-a716-446655440000',
    'test@example.com',
    '$2b$12$kQMaeUg9psmEZ/L6aS/d6.YrMZv5VCym3ebSX7D1.3lvT3iq6/wfC', -- 'password'
    'Test',
    'User',
    true
) ON CONFLICT (email) DO NOTHING;

-- Stock movements table - append-only ledger for inventory changes
CREATE TABLE IF NOT EXISTS stock_movements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id UUID NOT NULL REFERENCES items(id),
    location_id UUID NOT NULL REFERENCES locations(id),
    movement_type VARCHAR(20) NOT NULL CHECK (movement_type IN ('inbound', 'outbound', 'adjustment', 'transfer', 'initial')),
    quantity INTEGER NOT NULL,
    reference_type VARCHAR(20) NOT NULL CHECK (reference_type IN ('purchase_order', 'sales_order', 'adjustment', 'transfer', 'initial', 'return')),
    reference_id UUID,
    reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID REFERENCES users(id),
    CONSTRAINT positive_quantity CHECK (
        (movement_type IN ('inbound', 'adjustment', 'initial') AND quantity >= 0) OR
        (movement_type IN ('outbound', 'transfer') AND quantity <= 0)
    )
);

-- Create indexes for stock movements
CREATE INDEX IF NOT EXISTS idx_stock_movements_item_location ON stock_movements(item_id, location_id);
CREATE INDEX IF NOT EXISTS idx_stock_movements_reference ON stock_movements(reference_type, reference_id);
CREATE INDEX IF NOT EXISTS idx_stock_movements_created_at ON stock_movements(created_at);
CREATE INDEX IF NOT EXISTS idx_stock_movements_movement_type ON stock_movements(movement_type);

-- Stock levels table - current inventory snapshots
CREATE TABLE IF NOT EXISTS stock_levels (
    item_id UUID NOT NULL REFERENCES items(id),
    location_id UUID NOT NULL REFERENCES locations(id),
    quantity_on_hand INTEGER NOT NULL DEFAULT 0,
    last_movement_id UUID REFERENCES stock_movements(id),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (item_id, location_id),
    CONSTRAINT non_negative_stock CHECK (quantity_on_hand >= 0)
);

-- Create indexes for stock levels
CREATE INDEX IF NOT EXISTS idx_stock_levels_item ON stock_levels(item_id);
CREATE INDEX IF NOT EXISTS idx_stock_levels_location ON stock_levels(location_id);
CREATE INDEX IF NOT EXISTS idx_stock_levels_quantity ON stock_levels(quantity_on_hand);

-- Idempotency store table for request deduplication
CREATE TABLE IF NOT EXISTS idempotency_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    idempotency_key VARCHAR(255) NOT NULL UNIQUE,
    request_path VARCHAR(500) NOT NULL,
    request_method VARCHAR(10) NOT NULL,
    request_body_hash VARCHAR(64) NOT NULL,
    response_status INTEGER,
    response_body TEXT,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for idempotency keys
CREATE INDEX IF NOT EXISTS idx_idempotency_keys_key ON idempotency_keys(idempotency_key);
CREATE INDEX IF NOT EXISTS idx_idempotency_keys_expires ON idempotency_keys(expires_at);

-- Search indexes table for full-text search
CREATE TABLE IF NOT EXISTS search_indexes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(50) NOT NULL, -- 'item', 'stock_level', etc.
    entity_id UUID NOT NULL,
    search_vector TSVECTOR NOT NULL,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(entity_type, entity_id)
);

-- Create indexes for search
CREATE INDEX IF NOT EXISTS idx_search_indexes_vector ON search_indexes USING gin(search_vector);
CREATE INDEX IF NOT EXISTS idx_search_indexes_entity ON search_indexes(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_search_indexes_updated ON search_indexes(updated_at);

-- Purchase orders table
CREATE TABLE IF NOT EXISTS purchase_orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    po_number VARCHAR(100) NOT NULL UNIQUE,
    supplier_id UUID NOT NULL, -- References external supplier system
    status VARCHAR(20) NOT NULL CHECK (status IN ('DRAFT', 'OPEN', 'RECEIVING', 'PARTIAL_RECEIVED', 'RECEIVED', 'CANCELLED')),
    expected_date TIMESTAMPTZ,
    total_amount DOUBLE PRECISION NOT NULL DEFAULT 0 CHECK (total_amount >= 0),
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for purchase orders
CREATE INDEX IF NOT EXISTS idx_purchase_orders_po_number ON purchase_orders(po_number);
CREATE INDEX IF NOT EXISTS idx_purchase_orders_supplier ON purchase_orders(supplier_id);
CREATE INDEX IF NOT EXISTS idx_purchase_orders_status ON purchase_orders(status);
CREATE INDEX IF NOT EXISTS idx_purchase_orders_created_by ON purchase_orders(created_by);
CREATE INDEX IF NOT EXISTS idx_purchase_orders_created_at ON purchase_orders(created_at);

-- Purchase order lines table
CREATE TABLE IF NOT EXISTS purchase_order_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    po_id UUID NOT NULL REFERENCES purchase_orders(id) ON DELETE CASCADE,
    item_id UUID NOT NULL REFERENCES items(id),
    qty_ordered INTEGER NOT NULL CHECK (qty_ordered > 0),
    qty_received INTEGER NOT NULL DEFAULT 0 CHECK (qty_received >= 0),
    unit_cost DOUBLE PRECISION NOT NULL CHECK (unit_cost >= 0),
    line_total DOUBLE PRECISION NOT NULL CHECK (line_total >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT qty_received_not_exceeds_ordered CHECK (qty_received <= qty_ordered)
);

-- Create indexes for purchase order lines
CREATE INDEX IF NOT EXISTS idx_po_lines_po_id ON purchase_order_lines(po_id);
CREATE INDEX IF NOT EXISTS idx_po_lines_item_id ON purchase_order_lines(item_id);
CREATE INDEX IF NOT EXISTS idx_po_lines_created_at ON purchase_order_lines(created_at);

-- Sales orders table
CREATE TABLE IF NOT EXISTS sales_orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    so_number VARCHAR(100) NOT NULL UNIQUE,
    customer_id UUID, -- References external customer system
    status VARCHAR(20) NOT NULL CHECK (status IN ('DRAFT', 'CONFIRMED', 'PICKING', 'SHIPPED', 'INVOICED', 'CANCELLED', 'RETURNED')),
    total_amount DOUBLE PRECISION NOT NULL DEFAULT 0 CHECK (total_amount >= 0),
    fulfillment_location_id UUID REFERENCES locations(id),
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for sales orders
CREATE INDEX IF NOT EXISTS idx_sales_orders_so_number ON sales_orders(so_number);
CREATE INDEX IF NOT EXISTS idx_sales_orders_customer ON sales_orders(customer_id);
CREATE INDEX IF NOT EXISTS idx_sales_orders_status ON sales_orders(status);
CREATE INDEX IF NOT EXISTS idx_sales_orders_fulfillment_location ON sales_orders(fulfillment_location_id);
CREATE INDEX IF NOT EXISTS idx_sales_orders_created_by ON sales_orders(created_by);
CREATE INDEX IF NOT EXISTS idx_sales_orders_created_at ON sales_orders(created_at);

-- Sales order lines table
CREATE TABLE IF NOT EXISTS sales_order_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    so_id UUID NOT NULL REFERENCES sales_orders(id) ON DELETE CASCADE,
    item_id UUID NOT NULL REFERENCES items(id),
    qty INTEGER NOT NULL CHECK (qty > 0),
    unit_price DOUBLE PRECISION NOT NULL CHECK (unit_price >= 0),
    tax DOUBLE PRECISION DEFAULT 0 CHECK (tax >= 0),
    reserved BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for sales order lines
CREATE INDEX IF NOT EXISTS idx_so_lines_so_id ON sales_order_lines(so_id);
CREATE INDEX IF NOT EXISTS idx_so_lines_item_id ON sales_order_lines(item_id);
CREATE INDEX IF NOT EXISTS idx_so_lines_reserved ON sales_order_lines(reserved);
CREATE INDEX IF NOT EXISTS idx_so_lines_created_at ON sales_order_lines(created_at);

-- Transfers table
CREATE TABLE IF NOT EXISTS transfers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    transfer_number VARCHAR(100) NOT NULL UNIQUE,
    from_location_id UUID NOT NULL REFERENCES locations(id),
    to_location_id UUID NOT NULL REFERENCES locations(id),
    status VARCHAR(20) NOT NULL CHECK (status IN ('DRAFT', 'OPEN', 'IN_TRANSIT', 'RECEIVED', 'CANCELLED')),
    total_quantity INTEGER NOT NULL DEFAULT 0 CHECK (total_quantity >= 0),
    notes TEXT,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (from_location_id != to_location_id)
);

-- Create indexes for transfers
CREATE INDEX IF NOT EXISTS idx_transfers_transfer_number ON transfers(transfer_number);
CREATE INDEX IF NOT EXISTS idx_transfers_from_location ON transfers(from_location_id);
CREATE INDEX IF NOT EXISTS idx_transfers_to_location ON transfers(to_location_id);
CREATE INDEX IF NOT EXISTS idx_transfers_status ON transfers(status);
CREATE INDEX IF NOT EXISTS idx_transfers_created_by ON transfers(created_by);
CREATE INDEX IF NOT EXISTS idx_transfers_created_at ON transfers(created_at);

-- Transfer lines table
CREATE TABLE IF NOT EXISTS transfer_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    transfer_id UUID NOT NULL REFERENCES transfers(id) ON DELETE CASCADE,
    item_id UUID NOT NULL REFERENCES items(id),
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    quantity_received INTEGER NOT NULL DEFAULT 0 CHECK (quantity_received >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for transfer lines
CREATE INDEX IF NOT EXISTS idx_transfer_lines_transfer_id ON transfer_lines(transfer_id);
CREATE INDEX IF NOT EXISTS idx_transfer_lines_item_id ON transfer_lines(item_id);
CREATE INDEX IF NOT EXISTS idx_transfer_lines_created_at ON transfer_lines(created_at);

-- Create returns table
CREATE TABLE IF NOT EXISTS returns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    return_number VARCHAR(50) NOT NULL UNIQUE,
    location_id UUID NOT NULL REFERENCES locations(id),
    customer_id UUID REFERENCES users(id),
    status VARCHAR(20) NOT NULL DEFAULT 'DRAFT' CHECK (status IN ('DRAFT', 'OPEN', 'RECEIVED')),
    total_quantity INTEGER NOT NULL DEFAULT 0 CHECK (total_quantity >= 0),
    notes TEXT,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create return lines table
CREATE TABLE IF NOT EXISTS return_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    return_id UUID NOT NULL REFERENCES returns(id) ON DELETE CASCADE,
    item_id UUID NOT NULL REFERENCES items(id),
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    quantity_received INTEGER NOT NULL DEFAULT 0 CHECK (quantity_received >= 0),
    unit_price DOUBLE PRECISION NOT NULL DEFAULT 0.00 CHECK (unit_price >= 0),
    reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for returns
CREATE INDEX IF NOT EXISTS idx_returns_return_number ON returns(return_number);
CREATE INDEX IF NOT EXISTS idx_returns_location_id ON returns(location_id);
CREATE INDEX IF NOT EXISTS idx_returns_customer_id ON returns(customer_id);
CREATE INDEX IF NOT EXISTS idx_returns_status ON returns(status);
CREATE INDEX IF NOT EXISTS idx_returns_created_by ON returns(created_by);
CREATE INDEX IF NOT EXISTS idx_returns_created_at ON returns(created_at);

-- Create indexes for return lines
CREATE INDEX IF NOT EXISTS idx_return_lines_return_id ON return_lines(return_id);
CREATE INDEX IF NOT EXISTS idx_return_lines_item_id ON return_lines(item_id);
CREATE INDEX IF NOT EXISTS idx_return_lines_created_at ON return_lines(created_at);

-- Webhooks table for webhook configurations
CREATE TABLE IF NOT EXISTS webhooks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    url VARCHAR(2048) NOT NULL,
    secret VARCHAR(255) NOT NULL,
    events TEXT[] NOT NULL DEFAULT '{}',
    status VARCHAR(50) NOT NULL DEFAULT 'ACTIVE' CHECK (status IN ('ACTIVE', 'INACTIVE', 'FAILED')),
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_delivery_at TIMESTAMPTZ,
    failure_count INTEGER NOT NULL DEFAULT 0 CHECK (failure_count >= 0)
);

-- Create indexes for webhooks
CREATE INDEX IF NOT EXISTS idx_webhooks_created_by ON webhooks(created_by);
CREATE INDEX IF NOT EXISTS idx_webhooks_status ON webhooks(status);
CREATE INDEX IF NOT EXISTS idx_webhooks_events ON webhooks USING GIN(events);
CREATE INDEX IF NOT EXISTS idx_webhooks_created_at ON webhooks(created_at);

-- Webhook events table for storing events that triggered webhooks
CREATE TABLE IF NOT EXISTS webhook_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type VARCHAR(100) NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for webhook events
CREATE INDEX IF NOT EXISTS idx_webhook_events_event_type ON webhook_events(event_type);
CREATE INDEX IF NOT EXISTS idx_webhook_events_created_at ON webhook_events(created_at);

-- Webhook deliveries table for storing webhook delivery attempts
CREATE TABLE IF NOT EXISTS webhook_deliveries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    webhook_id UUID NOT NULL REFERENCES webhooks(id) ON DELETE CASCADE,
    event_id UUID NOT NULL REFERENCES webhook_events(id) ON DELETE CASCADE,
    status VARCHAR(50) NOT NULL CHECK (status IN ('PENDING', 'SUCCESS', 'FAILED', 'RETRY')),
    attempt_count INTEGER NOT NULL DEFAULT 0 CHECK (attempt_count >= 0),
    response_status INTEGER,
    response_body TEXT,
    error_message TEXT,
    next_retry_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for webhook deliveries
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_webhook_id ON webhook_deliveries(webhook_id);
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_event_id ON webhook_deliveries(event_id);
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_status ON webhook_deliveries(status);
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_next_retry ON webhook_deliveries(next_retry_at);
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_created_at ON webhook_deliveries(created_at);

-- Tenants table for multi-tenancy support
CREATE TABLE IF NOT EXISTS tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    tenant_type VARCHAR(20) NOT NULL CHECK (tenant_type IN ('SANDBOX', 'PRODUCTION')),
    status VARCHAR(20) NOT NULL DEFAULT 'ACTIVE' CHECK (status IN ('ACTIVE', 'SUSPENDED', 'DELETING', 'DELETED')),
    database_schema VARCHAR(255) NOT NULL UNIQUE,
    created_by UUID NOT NULL REFERENCES users(id),
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for tenants
CREATE INDEX IF NOT EXISTS idx_tenants_name ON tenants(name);
CREATE INDEX IF NOT EXISTS idx_tenants_tenant_type ON tenants(tenant_type);
CREATE INDEX IF NOT EXISTS idx_tenants_status ON tenants(status);
CREATE INDEX IF NOT EXISTS idx_tenants_created_by ON tenants(created_by);
CREATE INDEX IF NOT EXISTS idx_tenants_expires_at ON tenants(expires_at);
CREATE INDEX IF NOT EXISTS idx_tenants_created_at ON tenants(created_at);

-- Jobs table for async job processing

-- Create indexes for webhook deliveries
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_webhook_id ON webhook_deliveries(webhook_id);
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_event_id ON webhook_deliveries(event_id);
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_status ON webhook_deliveries(status);
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_next_attempt_at ON webhook_deliveries(next_attempt_at);
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_created_at ON webhook_deliveries(created_at);

-- Jobs table for async job processing
CREATE TABLE IF NOT EXISTS jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_id VARCHAR(255) NOT NULL UNIQUE,
    tenant_id UUID NOT NULL,
    type VARCHAR(100) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'QUEUED' CHECK (status IN ('QUEUED', 'RUNNING', 'SUCCESS', 'FAILED', 'PARTIAL_SUCCESS')),
    progress INTEGER NOT NULL DEFAULT 0 CHECK (progress >= 0 AND progress <= 100),
    payload JSONB,
    result_url VARCHAR(500),
    errors JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ
);

-- Create indexes for jobs
CREATE INDEX IF NOT EXISTS idx_jobs_job_id ON jobs(job_id);
CREATE INDEX IF NOT EXISTS idx_jobs_tenant_id ON jobs(tenant_id);
CREATE INDEX IF NOT EXISTS idx_jobs_type ON jobs(type);
CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status);
CREATE INDEX IF NOT EXISTS idx_jobs_created_at ON jobs(created_at);