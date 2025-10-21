# TWH API Quickstart

This directory contains quickstart materials for The Warehouse Hub API.

## Getting Started

1. **Create a Sandbox Tenant**

   ```bash
   curl -X POST http://localhost:8080/tenants/sandbox \
     -H "Content-Type: application/json" \
     -d '{}'
   ```

   This will return a response like:
   ```json
   {
     "id": "550e8400-e29b-41d4-a716-446655440000",
     "status": "ACTIVE",
     "created_at": "2025-10-21T10:00:00Z",
     "expires_at": "2025-11-20T10:00:00Z"
   }
   ```

2. **Set Your Tenant ID**
   - Use the `id` from the response as your tenant identifier
   - Set it as an environment variable: `export TWH_TENANT_ID=your_tenant_id_here`

3. **Test Basic Connectivity**

   ```bash
   curl -H "X-Tenant-ID: $TWH_TENANT_ID" \
        http://localhost:8080/healthz
   ```

## Sample Data

Your sandbox tenant comes pre-loaded with:

- **Locations**: Main Warehouse (WH-001), Retail Store (ST-001)
- **Items**: Laptop (electronics), Mouse (electronics), Keyboard (electronics), T-Shirt (apparel)

## Postman Collection

Import `twh-postman-collection.json` into Postman to explore all API endpoints with pre-configured requests.

### Environment Variables for Postman

Set these variables in your Postman environment:

- `base_url`: `http://localhost:8080`
- `tenant_id`: Your sandbox tenant ID

## Example: List Items

```bash
curl -H "X-Tenant-ID: $TWH_TENANT_ID" \
     http://localhost:8080/items
```

## Example: Create an Item

```bash
curl -X POST http://localhost:8080/items \
  -H "X-Tenant-ID: $TWH_TENANT_ID" \
  -H "Idempotency-Key: $(uuidgen)" \
  -H "Content-Type: application/json" \
  -d '{
    "sku": "WIDGET-001",
    "name": "Sample Widget",
    "description": "A sample inventory item",
    "unit": "ea",
    "cost_price": 29.99
  }'
```

## Webhook Testing

1. Create a webhook subscription
2. Use a service like ngrok or webhook.site to receive webhooks
3. Perform inventory operations to trigger webhook deliveries

## Next Steps

- Read the full API documentation at [api.thewarehousehub.com/docs](https://api.thewarehousehub.com/docs)
- Check out our SDKs: [Node.js](https://github.com/thewarehousehub/twh-node-sdk), [Python](https://github.com/thewarehousehub/twh-python-sdk)
- Join our developer community at [community.thewarehousehub.com](https://community.thewarehousehub.com)
