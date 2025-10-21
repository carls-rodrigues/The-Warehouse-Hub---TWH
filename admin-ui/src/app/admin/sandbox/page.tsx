"use client"

import AdminLayout from "@/components/admin-layout"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Textarea } from "@/components/ui/textarea"
import { Plus, Users, Settings, Trash2, Play, Pause } from "lucide-react"
import { useState } from "react"

interface Sandbox {
  id: string
  name: string
  tenantId: string
  status: 'active' | 'paused' | 'suspended'
  createdAt: string
  lastActivity: string
  webhookUrl: string
  description?: string
}

export default function SandboxManagement() {
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false)
  const [newSandbox, setNewSandbox] = useState({
    name: '',
    tenantId: '',
    webhookUrl: '',
    description: ''
  })

  // Mock data - in real app this would come from API
  const [sandboxes, setSandboxes] = useState<Sandbox[]>([
    {
      id: "sandbox-001",
      name: "E-commerce Integration Test",
      tenantId: "tenant-abc-123",
      status: "active",
      createdAt: "2024-01-10T08:00:00Z",
      lastActivity: "2024-01-15T14:30:00Z",
      webhookUrl: "https://test.example.com/webhooks",
      description: "Testing inventory sync and order processing"
    },
    {
      id: "sandbox-002",
      name: "Logistics Partner Sandbox",
      tenantId: "tenant-def-456",
      status: "active",
      createdAt: "2024-01-12T10:15:00Z",
      lastActivity: "2024-01-15T12:45:00Z",
      webhookUrl: "https://logistics.test.com/events",
      description: "Shipment tracking and delivery notifications"
    },
    {
      id: "sandbox-003",
      name: "Payment Gateway Test",
      tenantId: "tenant-ghi-789",
      status: "paused",
      createdAt: "2024-01-08T16:20:00Z",
      lastActivity: "2024-01-14T09:10:00Z",
      webhookUrl: "https://payments.test.com/webhooks",
      description: "Payment processing and refund handling"
    }
  ])

  const handleCreateSandbox = () => {
    const sandbox: Sandbox = {
      id: `sandbox-${Date.now()}`,
      name: newSandbox.name,
      tenantId: newSandbox.tenantId,
      status: "active",
      createdAt: new Date().toISOString(),
      lastActivity: new Date().toISOString(),
      webhookUrl: newSandbox.webhookUrl,
      description: newSandbox.description
    }
    setSandboxes([...sandboxes, sandbox])
    setNewSandbox({ name: '', tenantId: '', webhookUrl: '', description: '' })
    setIsCreateDialogOpen(false)
  }

  const handleStatusChange = (id: string, newStatus: Sandbox['status']) => {
    setSandboxes(sandboxes.map(sb =>
      sb.id === id ? { ...sb, status: newStatus } : sb
    ))
  }

  const handleDeleteSandbox = (id: string) => {
    setSandboxes(sandboxes.filter(sb => sb.id !== id))
  }

  const getStatusBadge = (status: Sandbox['status']) => {
    switch (status) {
      case 'active':
        return <Badge className="bg-green-100 text-green-800">Active</Badge>
      case 'paused':
        return <Badge variant="secondary">Paused</Badge>
      case 'suspended':
        return <Badge variant="destructive">Suspended</Badge>
    }
  }

  return (
    <AdminLayout>
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold md:text-2xl">Sandbox Management</h1>
          <p className="text-muted-foreground">Create and manage sandbox environments for testing</p>
        </div>
        <Dialog open={isCreateDialogOpen} onOpenChange={setIsCreateDialogOpen}>
          <DialogTrigger asChild>
            <Button>
              <Plus className="h-4 w-4 mr-2" />
              Create Sandbox
            </Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Create New Sandbox</DialogTitle>
              <DialogDescription>
                Set up a new sandbox environment for testing webhook integrations
              </DialogDescription>
            </DialogHeader>
            <div className="space-y-4">
              <div>
                <Label htmlFor="name">Sandbox Name</Label>
                <Input
                  id="name"
                  value={newSandbox.name}
                  onChange={(e) => setNewSandbox({...newSandbox, name: e.target.value})}
                  placeholder="E-commerce Integration Test"
                />
              </div>
              <div>
                <Label htmlFor="tenantId">Tenant ID</Label>
                <Input
                  id="tenantId"
                  value={newSandbox.tenantId}
                  onChange={(e) => setNewSandbox({...newSandbox, tenantId: e.target.value})}
                  placeholder="tenant-abc-123"
                />
              </div>
              <div>
                <Label htmlFor="webhookUrl">Webhook URL</Label>
                <Input
                  id="webhookUrl"
                  value={newSandbox.webhookUrl}
                  onChange={(e) => setNewSandbox({...newSandbox, webhookUrl: e.target.value})}
                  placeholder="https://test.example.com/webhooks"
                />
              </div>
              <div>
                <Label htmlFor="description">Description (Optional)</Label>
                <Textarea
                  id="description"
                  value={newSandbox.description}
                  onChange={(e) => setNewSandbox({...newSandbox, description: e.target.value})}
                  placeholder="Purpose and testing scope"
                  rows={3}
                />
              </div>
            </div>
            <DialogFooter>
              <Button variant="outline" onClick={() => setIsCreateDialogOpen(false)}>
                Cancel
              </Button>
              <Button onClick={handleCreateSandbox} disabled={!newSandbox.name || !newSandbox.tenantId || !newSandbox.webhookUrl}>
                Create Sandbox
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>

      <div className="grid gap-4 md:gap-8 lg:grid-cols-3">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Sandboxes</CardTitle>
            <Users className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{sandboxes.length}</div>
            <p className="text-xs text-muted-foreground">
              {sandboxes.filter(sb => sb.status === 'active').length} active
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Active Sandboxes</CardTitle>
            <Play className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {sandboxes.filter(sb => sb.status === 'active').length}
            </div>
            <p className="text-xs text-muted-foreground">
              Currently running
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Paused Sandboxes</CardTitle>
            <Pause className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {sandboxes.filter(sb => sb.status === 'paused').length}
            </div>
            <p className="text-xs text-muted-foreground">
              Temporarily stopped
            </p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Sandbox Environments</CardTitle>
          <CardDescription>
            Manage sandbox tenants and their webhook configurations
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Name</TableHead>
                <TableHead>Tenant ID</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Last Activity</TableHead>
                <TableHead>Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {sandboxes.map((sandbox) => (
                <TableRow key={sandbox.id}>
                  <TableCell>
                    <div>
                      <div className="font-medium">{sandbox.name}</div>
                      {sandbox.description && (
                        <div className="text-sm text-muted-foreground">{sandbox.description}</div>
                      )}
                    </div>
                  </TableCell>
                  <TableCell className="font-mono text-sm">{sandbox.tenantId}</TableCell>
                  <TableCell>{getStatusBadge(sandbox.status)}</TableCell>
                  <TableCell className="text-sm">
                    {new Date(sandbox.lastActivity).toLocaleString()}
                  </TableCell>
                  <TableCell>
                    <div className="flex items-center space-x-2">
                      <Select
                        value={sandbox.status}
                        onValueChange={(value: Sandbox['status']) => handleStatusChange(sandbox.id, value)}
                      >
                        <SelectTrigger className="w-24">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="active">Active</SelectItem>
                          <SelectItem value="paused">Pause</SelectItem>
                          <SelectItem value="suspended">Suspend</SelectItem>
                        </SelectContent>
                      </Select>

                      <Button variant="outline" size="sm">
                        <Settings className="h-3 w-3" />
                      </Button>

                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => handleDeleteSandbox(sandbox.id)}
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