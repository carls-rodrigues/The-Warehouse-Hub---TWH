const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

class ApiError extends Error {
  constructor(public status: number, message: string) {
    super(message);
    this.name = 'ApiError';
  }
}

async function apiRequest<T>(
  endpoint: string,
  options: RequestInit = {}
): Promise<T> {
  const url = `${API_BASE_URL}${endpoint}`;

  const response = await fetch(url, {
    headers: {
      'Content-Type': 'application/json',
      ...options.headers,
    },
    ...options,
  });

  if (!response.ok) {
    const errorText = await response.text();
    throw new ApiError(response.status, errorText || `HTTP ${response.status}`);
  }

  return response.json();
}

// Admin API endpoints
export const adminApi = {
  // Billing
  getBillingMetrics: (): Promise<{
    total_api_calls: number
    storage_used_gb: number
    active_tenants: number
    total_items: number
    total_locations: number
    total_orders: number
    total_transfers: number
    webhook_deliveries: {
      total: number
      successful: number
      failed: number
    }
    billing_period: {
      start_date: string
      end_date: string
      days_remaining: number
    }
  }> => apiRequest('/admin/billing'),

  // DLQ Management
  getDlqDeliveries: (page = 1, perPage = 50): Promise<{
    deliveries: Array<{
      id: string
      webhook_id: string
      event_id: string
      status: 'PENDING' | 'SUCCESS' | 'FAILED' | 'TIMEOUT' | 'DLQ'
      attempt_count: number
      last_attempt_at?: string
      response_status?: number
      error_message?: string
      created_at: string
    }>
    pagination: {
      page: number
      per_page: number
      total: number
      total_pages: number
    }
  }> => apiRequest(`/admin/dlq?page=${page}&per_page=${perPage}`),

  replayDlqDelivery: (deliveryId: string): Promise<{
    success: boolean
    message: string
    new_status: 'PENDING' | 'SUCCESS' | 'FAILED' | 'TIMEOUT' | 'DLQ'
  }> => apiRequest('/admin/dlq/replay', {
      method: 'POST',
      body: JSON.stringify({ delivery_id: deliveryId }),
    }),

    // Sandbox Management
  getSandboxes: (): Promise<{
    sandboxes: Array<{
      id: string
      name: string
      status: 'PROVISIONING' | 'ACTIVE' | 'EXPIRED' | 'SUSPENDED' | 'DELETING'
      created_at: string
      expires_at: string
    }>
  }> => apiRequest('/admin/sandboxes'),

  cleanupExpiredSandboxes: (): Promise<{
    cleaned_tenant_ids: string[]
    message: string
  }> => apiRequest('/admin/sandboxes/cleanup', {
      method: 'POST',
    }),

  // Dashboard
  getDashboard: () => apiRequest('/admin/dashboard'),
};

export { ApiError };