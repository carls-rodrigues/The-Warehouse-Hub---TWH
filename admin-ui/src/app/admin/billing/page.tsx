"use client"

import AdminLayout from "@/components/admin-layout"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Progress } from "@/components/ui/progress"
import { DollarSign, TrendingUp, Users, Database, Package, MapPin, ShoppingCart, Truck, Webhook, Calendar, AlertCircle } from "lucide-react"
import { useEffect, useState } from "react"
import { adminApi, ApiError } from "@/lib/api"

interface BillingMetrics {
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
}

export default function BillingManagement() {
  const [metrics, setMetrics] = useState<BillingMetrics | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const fetchMetrics = async () => {
      try {
        setLoading(true)
        const data = await adminApi.getBillingMetrics()
        setMetrics(data)
      } catch (err) {
        if (err instanceof ApiError) {
          setError(`Failed to load billing metrics: ${err.message}`)
        } else {
          setError('Failed to load billing metrics')
        }
      } finally {
        setLoading(false)
      }
    }

    fetchMetrics()
  }, [])

  if (loading) {
    return (
      <AdminLayout>
        <div className="flex items-center justify-center h-64">
          <div className="text-lg">Loading billing metrics...</div>
        </div>
      </AdminLayout>
    )
  }

  if (error) {
    return (
      <AdminLayout>
        <div className="flex items-center justify-center h-64">
          <div className="text-red-600 flex items-center gap-2">
            <AlertCircle className="h-5 w-5" />
            {error}
          </div>
        </div>
      </AdminLayout>
    )
  }

  if (!metrics) {
    return (
      <AdminLayout>
        <div className="flex items-center justify-center h-64">
          <div className="text-gray-600">No billing data available</div>
        </div>
      </AdminLayout>
    )
  }

  const webhookSuccessRate = metrics.webhook_deliveries.total > 0
    ? (metrics.webhook_deliveries.successful / metrics.webhook_deliveries.total) * 100
    : 0

  return (
    <AdminLayout>
      <div className="flex items-center">
        <h1 className="text-lg font-semibold md:text-2xl">Billing & Usage</h1>
      </div>

      <div className="grid gap-4 md:grid-cols-2 md:gap-8 lg:grid-cols-4 mt-6">
        {/* Billing Period */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Billing Period</CardTitle>
            <Calendar className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{metrics.billing_period.days_remaining}</div>
            <p className="text-xs text-muted-foreground">
              days remaining
            </p>
            <div className="mt-2 text-xs">
              {new Date(metrics.billing_period.start_date).toLocaleDateString()} - {new Date(metrics.billing_period.end_date).toLocaleDateString()}
            </div>
          </CardContent>
        </Card>

        {/* Active Tenants */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Active Tenants</CardTitle>
            <Users className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{metrics.active_tenants}</div>
            <p className="text-xs text-muted-foreground">
              total active tenants
            </p>
          </CardContent>
        </Card>

        {/* API Calls */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">API Calls</CardTitle>
            <TrendingUp className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{metrics.total_api_calls.toLocaleString()}</div>
            <p className="text-xs text-muted-foreground">
              this billing period
            </p>
          </CardContent>
        </Card>

        {/* Storage Used */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Storage Used</CardTitle>
            <Database className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{metrics.storage_used_gb} GB</div>
            <p className="text-xs text-muted-foreground">
              total storage used
            </p>
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3 mt-6">
        {/* Inventory Stats */}
        <Card>
          <CardHeader>
            <CardTitle className="text-sm font-medium">Inventory Overview</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Package className="h-4 w-4 text-muted-foreground" />
                <span className="text-sm">Items</span>
              </div>
              <span className="font-semibold">{metrics.total_items.toLocaleString()}</span>
            </div>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <MapPin className="h-4 w-4 text-muted-foreground" />
                <span className="text-sm">Locations</span>
              </div>
              <span className="font-semibold">{metrics.total_locations.toLocaleString()}</span>
            </div>
          </CardContent>
        </Card>

        {/* Transaction Stats */}
        <Card>
          <CardHeader>
            <CardTitle className="text-sm font-medium">Transactions</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <ShoppingCart className="h-4 w-4 text-muted-foreground" />
                <span className="text-sm">Orders</span>
              </div>
              <span className="font-semibold">{metrics.total_orders.toLocaleString()}</span>
            </div>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Truck className="h-4 w-4 text-muted-foreground" />
                <span className="text-sm">Transfers</span>
              </div>
              <span className="font-semibold">{metrics.total_transfers.toLocaleString()}</span>
            </div>
          </CardContent>
        </Card>

        {/* Webhook Stats */}
        <Card>
          <CardHeader>
            <CardTitle className="text-sm font-medium">Webhook Deliveries</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between">
              <span className="text-sm">Total</span>
              <span className="font-semibold">{metrics.webhook_deliveries.total.toLocaleString()}</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm text-green-600">Successful</span>
              <span className="font-semibold text-green-600">{metrics.webhook_deliveries.successful.toLocaleString()}</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm text-red-600">Failed</span>
              <span className="font-semibold text-red-600">{metrics.webhook_deliveries.failed.toLocaleString()}</span>
            </div>
            <div className="space-y-2">
              <div className="flex justify-between text-sm">
                <span>Success Rate</span>
                <span>{webhookSuccessRate.toFixed(1)}%</span>
              </div>
              <Progress value={webhookSuccessRate} className="h-2" />
            </div>
          </CardContent>
        </Card>
      </div>
    </AdminLayout>
  )
}