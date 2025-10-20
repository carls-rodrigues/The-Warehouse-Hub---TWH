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