# Purchase Orders API Guide

## Overview

The Purchase Orders API provides complete lifecycle management for procurement operations, including creation, tracking, and receiving of goods with automatic inventory updates.

## API Endpoints

### Create Purchase Order

**Endpoint:** `POST /purchase_orders`

Creates a new purchase order with line items. Requires authentication.

**Request:**
```json
{
  "supplier_id": "550e8400-e29b-41d4-a716-446655440000",
  "expected_date": "2024-12-01T00:00:00Z",
  "lines": [
    {
      "item_id": "550e8400-e29b-41d4-a716-446655440001",
      "qty_ordered": 100,
      "unit_cost": 9.50
    },
    {
      "item_id": "550e8400-e29b-41d4-a716-446655440002",
      "qty_ordered": 50,
      "unit_cost": 22.00
    }
  ]
}
```

**Response (201):**
```json
{
  "id": "8c731dc9-bf85-463c-a9b9-6175a3a98a08",
  "po_number": "PO-1760962802",
  "supplier_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "DRAFT",
  "total_amount": 2050.0,
  "lines": [
    {
      "id": "1b4a0477-4c2f-4dfe-8709-5818909a506b",
      "item_id": "550e8400-e29b-41d4-a716-446655440001",
      "qty_ordered": 100,
      "qty_received": 0,
      "unit_cost": 9.5,
      "line_total": 950.0
    }
  ],
  "created_at": "2025-10-20T12:20:02.987354891Z"
}
```

### Get Purchase Order

**Endpoint:** `GET /purchase_orders/{poId}`

Retrieves a purchase order by ID.

**Response (200):**
```json
{
  "id": "8c731dc9-bf85-463c-a9b9-6175a3a98a08",
  "po_number": "PO-1760962802",
  "supplier_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "PARTIAL_RECEIVED",
  "expected_date": "2024-12-01T00:00:00Z",
  "total_amount": 2050.0,
  "lines": [...],
  "created_by": "550e8400-e29b-41d4-a716-446655440000",
  "created_at": "2025-10-20T12:20:02.987354Z",
  "updated_at": "2025-10-20T12:21:14.516047Z"
}
```

### Receive Purchase Order Items

**Endpoint:** `POST /purchase_orders/{poId}/receive`

Records receipt of items against a purchase order, automatically creating stock movements.

**Request:**
```json
{
  "received_lines": [
    {
      "po_line_id": "1b4a0477-4c2f-4dfe-8709-5818909a506b",
      "qty_received": 50
    },
    {
      "po_line_id": "c2032b62-acc1-43a3-82c3-b00ed53c63c6",
      "qty_received": 25
    }
  ],
  "receive_date": "2024-10-20T12:30:00Z",
  "destination_location_id": "550e8400-e29b-41d4-a716-446655440010"
}
```

**Response (200):**
```json
{
  "po": {
    "id": "8c731dc9-bf85-463c-a9b9-6175a3a98a08",
    "status": "PARTIAL_RECEIVED",
    "lines": [...]
  },
  "stock_movements": [
    {
      "id": "a791b161-0b80-46ff-b278-f1e3bb307d34",
      "item_id": "550e8400-e29b-41d4-a716-446655440001",
      "location_id": "550e8400-e29b-41d4-a716-446655440010",
      "quantity": 50,
      "movement_type": "inbound",
      "reference_type": "purchase_order",
      "reference_id": "8c731dc9-bf85-463c-a9b9-6175a3a98a08",
      "reason": "PO-PO-1760962802",
      "created_at": "2025-10-20T12:21:14.551778341Z"
    }
  ]
}
```

## Status Flow

```
DRAFT → OPEN → RECEIVING → PARTIAL_RECEIVED → RECEIVED
   ↓      ↓         ↓             ↓              ↓
   └─ Initial creation ─┴─ First receive ─┴─ Complete ─┘
```

## Business Rules

- **Partial Receiving:** Allowed - orders can be received in multiple shipments
- **Over-Receiving:** Not allowed - cannot receive more than ordered
- **Stock Integration:** All receipts automatically create INBOUND stock movements
- **Audit Trail:** Complete history maintained through stock movements ledger
- **Data Consistency:** All operations are transactional

## Error Handling

- `400 Bad Request`: Invalid quantities, missing required fields
- `404 Not Found`: Purchase order doesn't exist
- `409 Conflict`: Concurrent modification (use ETag for updates)

## Integration Points

- **Stock Ledger:** Automatic stock movement creation
- **Items:** References validated against item catalog
- **Locations:** Destination location must exist and be active
- **Suppliers:** Supplier reference for procurement tracking</content>
<parameter name="filePath">/home/cerf/development/The-Warehouse-Hub---TWH/docs/PURCHASE_ORDERS_API.md