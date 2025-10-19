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

-- Insert a test user for development
-- Password is 'password' hashed with bcrypt
INSERT INTO users (id, email, password_hash, first_name, last_name, active)
VALUES (
    '550e8400-e29b-41d4-a716-446655440000',
    'test@example.com',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPjYQmO7K3Q7e', -- 'password'
    'Test',
    'User',
    true
) ON CONFLICT (email) DO NOTHING;