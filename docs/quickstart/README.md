# TWH API Quickstart

This directory contains quickstart materials for The Warehouse Hub API.

## Getting Started

1. **Create a Sandbox Tenant**

   ```bash
   curl -X POST https://sandbox.api.thewarehousehub.com/v1/admin/sandbox \
     -H "Authorization: Bearer YOUR_API_KEY" \
     -H "Content-Type: application/json" \
     -d '{}'
   ```

2. **Get Your API Key**
   - Use the API key returned from the sandbox creation
   - Set it as an environment variable: `export TWH_API_KEY=your_key_here`

3. **Test Basic Connectivity**

   ```bash
   curl -H "Authorization: Bearer $TWH_API_KEY" \
        -H "X-Tenant-ID: YOUR_TENANT_ID" \
        https://sandbox.api.thewarehousehub.com/v1/healthz
   ```

## Sample Data

Your sandbox tenant comes pre-loaded with:

- Sample items (widgets, gadgets)
- Sample locations (warehouse-1, warehouse-2)
- Sample webhook subscriptions

## Postman Collection

Import `twh-api.postman_collection.json` into Postman to explore all API endpoints with pre-configured requests.

### Environment Variables for Postman

Set these variables in your Postman environment:

- `base_url`: `https://sandbox.api.thewarehousehub.com/v1`
- `api_key`: Your sandbox API key
- `tenant_id`: Your sandbox tenant ID

## Example: Create an Item

```bash
curl -X POST https://sandbox.api.thewarehousehub.com/v1/items \
  -H "Authorization: Bearer $TWH_API_KEY" \
  -H "X-Tenant-ID: $TENANT_ID" \
  -H "Idempotency-Key: $(uuidgen)" \
  -H "Content-Type: application/json" \
  -d '{
    "sku": "WIDGET-001",
    "name": "Sample Widget",
    "description": "A sample inventory item",
    "metadata": {
      "category": "electronics",
      "price": 29.99
    }
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
