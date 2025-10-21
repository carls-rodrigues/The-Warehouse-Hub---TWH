"use client"

import AdminLayout from "@/components/admin-layout"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Plus, Webhook, Settings, Trash2, TestTube, CheckCircle, XCircle } from "lucide-react"
import { useState } from "react"

interface WebhookConfig {
  id: string
  tenantId: string
  tenantName: string
  url: string
  events: string[]
  status: 'active' | 'inactive' | 'failed'
  createdAt: string
  lastSuccess?: string
  lastFailure?: string
  retryCount: number
}

export default function WebhooksManagement() {
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false)
  const [newWebhook, setNewWebhook] = useState({
    tenantId: '',
    tenantName: '',
    url: '',
    events: [] as string[]
  })

  // Mock data - in real app this would come from API
  const [webhooks, setWebhooks] = useState<WebhookConfig[]>([
    {
      id: "wh-001",
      tenantId: "tenant-abc-123",
      tenantName: "E-commerce Corp",
      url: "https://api.ecommerce.com/webhooks/inventory",
      events: ["inventory.updated", "order.created", "shipment.delivered"],
      status: "active",
      createdAt: "2024-01-10T08:00:00Z",
      lastSuccess: "2024-01-15T14:30:00Z",
      retryCount: 0
    },
    {
      id: "wh-002",
      tenantId: "tenant-def-456",
      tenantName: "Logistics Plus",
      url: "https://hooks.logistics.com/events",
      events: ["shipment.status_changed", "order.fulfilled"],
      status: "active",
      createdAt: "2024-01-12T10:15:00Z",
      lastSuccess: "2024-01-15T12:45:00Z",
      retryCount: 0
    },
    {
      id: "wh-003",
      tenantId: "tenant-ghi-789",
      tenantName: "Payment Solutions Inc",
      url: "https://webhooks.payments.com/notifications",
      events: ["payment.completed", "refund.processed"],
      status: "failed",
      createdAt: "2024-01-08T16:20:00Z",
      lastFailure: "2024-01-14T09:10:00Z",
      retryCount: 3
    }
  ])

  const availableEvents = [
    "inventory.updated",
    "order.created",
    "order.fulfilled",
    "shipment.delivered",
    "shipment.status_changed",
    "payment.completed",
    "refund.processed",
    "customer.created",
    "customer.updated"
  ]

  const handleCreateWebhook = () => {
    const webhook: WebhookConfig = {
      id: `wh-${Date.now()}`,
      tenantId: newWebhook.tenantId,
      tenantName: newWebhook.tenantName,
      url: newWebhook.url,
      events: newWebhook.events,
      status: "active",
      createdAt: new Date().toISOString(),
      retryCount: 0
    }
    setWebhooks([...webhooks, webhook])
    setNewWebhook({ tenantId: '', tenantName: '', url: '', events: [] })
    setIsCreateDialogOpen(false)
  }

  const handleTestWebhook = async () => {
    // Simulate webhook test
    await new Promise(resolve => setTimeout(resolve, 1000))
    // In real app, this would send a test event
  }

  const handleDeleteWebhook = (id: string) => {
    setWebhooks(webhooks.filter(wh => wh.id !== id))
  }

  const getStatusBadge = (status: WebhookConfig['status']) => {
    switch (status) {
      case 'active':
        return <Badge className="bg-green-100 text-green-800">Active</Badge>
      case 'inactive':
        return <Badge variant="secondary">Inactive</Badge>
      case 'failed':
        return <Badge variant="destructive">Failed</Badge>
    }
  }

  const toggleEvent = (event: string) => {
    setNewWebhook(prev => ({
      ...prev,
      events: prev.events.includes(event)
        ? prev.events.filter(e => e !== event)
        : [...prev.events, event]
    }))
  }

  return (
    <AdminLayout>
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold md:text-2xl">Webhooks Management</h1>
          <p className="text-muted-foreground">Configure and monitor webhook endpoints</p>
        </div>
        <Dialog open={isCreateDialogOpen} onOpenChange={setIsCreateDialogOpen}>
          <DialogTrigger asChild>
            <Button>
              <Plus className="h-4 w-4 mr-2" />
              Add Webhook
            </Button>
          </DialogTrigger>
          <DialogContent className="max-w-2xl">
            <DialogHeader>
              <DialogTitle>Add New Webhook</DialogTitle>
              <DialogDescription>
                Configure a new webhook endpoint for event notifications
              </DialogDescription>
            </DialogHeader>
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label htmlFor="tenantId">Tenant ID</Label>
                  <Input
                    id="tenantId"
                    value={newWebhook.tenantId}
                    onChange={(e) => setNewWebhook({...newWebhook, tenantId: e.target.value})}
                    placeholder="tenant-abc-123"
                  />
                </div>
                <div>
                  <Label htmlFor="tenantName">Tenant Name</Label>
                  <Input
                    id="tenantName"
                    value={newWebhook.tenantName}
                    onChange={(e) => setNewWebhook({...newWebhook, tenantName: e.target.value})}
                    placeholder="E-commerce Corp"
                  />
                </div>
              </div>
              <div>
                <Label htmlFor="url">Webhook URL</Label>
                <Input
                  id="url"
                  value={newWebhook.url}
                  onChange={(e) => setNewWebhook({...newWebhook, url: e.target.value})}
                  placeholder="https://api.example.com/webhooks"
                />
              </div>
              <div>
                <Label>Events to Subscribe</Label>
                <div className="grid grid-cols-2 gap-2 mt-2">
                  {availableEvents.map((event) => (
                    <div key={event} className="flex items-center space-x-2">
                      <input
                        type="checkbox"
                        id={event}
                        checked={newWebhook.events.includes(event)}
                        onChange={() => toggleEvent(event)}
                        className="rounded"
                      />
                      <Label htmlFor={event} className="text-sm">{event}</Label>
                    </div>
                  ))}
                </div>
              </div>
            </div>
            <DialogFooter>
              <Button variant="outline" onClick={() => setIsCreateDialogOpen(false)}>
                Cancel
              </Button>
              <Button
                onClick={handleCreateWebhook}
                disabled={!newWebhook.tenantId || !newWebhook.tenantName || !newWebhook.url || newWebhook.events.length === 0}
              >
                Create Webhook
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>

      <div className="grid gap-4 md:gap-8 lg:grid-cols-3">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Webhooks</CardTitle>
            <Webhook className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{webhooks.length}</div>
            <p className="text-xs text-muted-foreground">
              Configured endpoints
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Active Webhooks</CardTitle>
            <CheckCircle className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {webhooks.filter(wh => wh.status === 'active').length}
            </div>
            <p className="text-xs text-muted-foreground">
              Successfully delivering
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Failed Webhooks</CardTitle>
            <XCircle className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {webhooks.filter(wh => wh.status === 'failed').length}
            </div>
            <p className="text-xs text-muted-foreground">
              Require attention
            </p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Webhook Configurations</CardTitle>
          <CardDescription>
            Manage webhook endpoints and their event subscriptions
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Tenant</TableHead>
                <TableHead>URL</TableHead>
                <TableHead>Events</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Last Activity</TableHead>
                <TableHead>Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {webhooks.map((webhook) => (
                <TableRow key={webhook.id}>
                  <TableCell>
                    <div>
                      <div className="font-medium">{webhook.tenantName}</div>
                      <div className="text-sm text-muted-foreground font-mono">{webhook.tenantId}</div>
                    </div>
                  </TableCell>
                  <TableCell className="max-w-xs">
                    <span className="text-sm break-all block">{webhook.url}</span>
                  </TableCell>
                  <TableCell>
                    <div className="flex flex-wrap gap-1">
                      {webhook.events.slice(0, 2).map((event) => (
                        <Badge key={event} variant="outline" className="text-xs">
                          {event}
                        </Badge>
                      ))}
                      {webhook.events.length > 2 && (
                        <Badge variant="outline" className="text-xs">
                          +{webhook.events.length - 2} more
                        </Badge>
                      )}
                    </div>
                  </TableCell>
                  <TableCell>{getStatusBadge(webhook.status)}</TableCell>
                  <TableCell className="text-sm">
                    {webhook.lastSuccess && (
                      <div className="text-green-600">
                        Success: {new Date(webhook.lastSuccess).toLocaleString()}
                      </div>
                    )}
                    {webhook.lastFailure && (
                      <div className="text-red-600">
                        Failed: {new Date(webhook.lastFailure).toLocaleString()}
                      </div>
                    )}
                    {!webhook.lastSuccess && !webhook.lastFailure && (
                      <span className="text-muted-foreground">No activity</span>
                    )}
                  </TableCell>
                  <TableCell>
                    <div className="flex items-center space-x-2">
                      <Button variant="outline" size="sm" onClick={() => handleTestWebhook()}>
                        <TestTube className="h-3 w-3" />
                      </Button>

                      <Button variant="outline" size="sm">
                        <Settings className="h-3 w-3" />
                      </Button>

                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => handleDeleteWebhook(webhook.id)}
                      >
                        <Trash2 className="h-3 w-3" />
                      </Button>
                    </div>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </AdminLayout>
  )
}