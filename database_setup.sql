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
    reference_type VARCHAR(20) NOT NULL CHECK (reference_type IN ('purchase_order', 'sales_order', 'adjustment', 'transfer', 'initial')),
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