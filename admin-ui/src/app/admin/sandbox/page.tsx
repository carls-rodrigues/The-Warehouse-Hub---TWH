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
import { Plus, Users, Settings, Trash2, Play, Pause, AlertCircle, RefreshCw } from "lucide-react"
import { useEffect, useState } from "react"
import { adminApi, ApiError } from "@/lib/api"

interface SandboxTenant {
  id: string
  name: string
  status: 'PROVISIONING' | 'ACTIVE' | 'EXPIRED' | 'SUSPENDED' | 'DELETING'
  created_at: string
  expires_at: string
}

interface SandboxesResponse {
  sandboxes: SandboxTenant[]
}

export default function SandboxManagement() {
  const [sandboxesData, setSandboxesData] = useState<SandboxesResponse | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false)
  const [isCleaningUp, setIsCleaningUp] = useState(false)
  const [cleanupResult, setCleanupResult] = useState<{ success: boolean; message: string } | null>(null)
  const [newSandbox, setNewSandbox] = useState({
    name: '',
    tenantId: '',
    webhookUrl: '',
    description: ''
  })

  useEffect(() => {
    fetchSandboxes()
  }, [])

  const fetchSandboxes = async () => {
    try {
      setLoading(true)
      const data = await adminApi.getSandboxes()
      setSandboxesData(data)
    } catch (err) {
      if (err instanceof ApiError) {
        setError(`Failed to load sandboxes: ${err.message}`)
      } else {
        setError('Failed to load sandboxes')
      }
    } finally {
      setLoading(false)
    }
  }

  const handleCleanupExpired = async () => {
    try {
      setIsCleaningUp(true)
      setCleanupResult(null)
      const result = await adminApi.cleanupExpiredSandboxes()
      setCleanupResult({ success: true, message: result.message })

      // Refresh the list after cleanup
      await fetchSandboxes()
    } catch (err) {
      if (err instanceof ApiError) {
        setCleanupResult({ success: false, message: `Failed: ${err.message}` })
      } else {
        setCleanupResult({ success: false, message: 'Failed to cleanup expired sandboxes' })
      }
    } finally {
      setIsCleaningUp(false)
    }
  }

  const getStatusBadge = (status: SandboxTenant['status']) => {
    switch (status) {
      case 'ACTIVE':
        return <Badge className="bg-green-100 text-green-800">Active</Badge>
      case 'PROVISIONING':
        return <Badge variant="secondary">Provisioning</Badge>
      case 'EXPIRED':
        return <Badge variant="destructive">Expired</Badge>
      case 'SUSPENDED':
        return <Badge className="bg-yellow-100 text-yellow-800">Suspended</Badge>
      case 'DELETING':
        return <Badge className="bg-red-100 text-red-800">Deleting</Badge>
      default:
        return <Badge variant="outline">{status}</Badge>
    }
  }

  const getDaysUntilExpiry = (expiresAt: string) => {
    const now = new Date()
    const expiry = new Date(expiresAt)
    const diffTime = expiry.getTime() - now.getTime()
    const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24))
    return diffDays
  }

  if (loading) {
    return (
      <AdminLayout>
        <div className="flex items-center justify-center h-64">
          <div className="text-lg">Loading sandboxes...</div>
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

  const sandboxes = sandboxesData?.sandboxes || []
  const activeSandboxes = sandboxes.filter(s => s.status === 'ACTIVE').length
  const expiredSandboxes = sandboxes.filter(s => s.status === 'EXPIRED').length
  const provisioningSandboxes = sandboxes.filter(s => s.status === 'PROVISIONING').length

  return (
    <AdminLayout>
      <div className="flex items-center justify-between">
        <h1 className="text-lg font-semibold md:text-2xl">Sandbox Management</h1>
        <div className="flex items-center gap-2">
          <Button onClick={fetchSandboxes} variant="outline" size="sm">
            <RefreshCw className="mr-2 h-4 w-4" />
            Refresh
          </Button>
          <Button onClick={handleCleanupExpired} disabled={isCleaningUp} variant="outline" size="sm">
            {isCleaningUp ? (
              <>
                <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                Cleaning...
              </>
            ) : (
              <>
                <Trash2 className="mr-2 h-4 w-4" />
                Cleanup Expired
              </>
            )}
          </Button>
          <Dialog open={isCreateDialogOpen} onOpenChange={setIsCreateDialogOpen}>
            <DialogTrigger asChild>
              <Button size="sm">
                <Plus className="mr-2 h-4 w-4" />
                Create Sandbox
              </Button>
            </DialogTrigger>
            <DialogContent>
              <DialogHeader>
                <DialogTitle>Create New Sandbox</DialogTitle>
                <DialogDescription>
                  Create a new sandbox tenant for testing and development
                </DialogDescription>
              </DialogHeader>
              <div className="space-y-4">
                <div>
                  <Label htmlFor="name">Sandbox Name</Label>
                  <Input
                    id="name"
                    value={newSandbox.name}
                    onChange={(e) => setNewSandbox(prev => ({ ...prev, name: e.target.value }))}
                    placeholder="Enter sandbox name"
                  />
                </div>
                <div>
                  <Label htmlFor="tenantId">Tenant ID (Optional)</Label>
                  <Input
                    id="tenantId"
                    value={newSandbox.tenantId}
                    onChange={(e) => setNewSandbox(prev => ({ ...prev, tenantId: e.target.value }))}
                    placeholder="Auto-generated if empty"
                  />
                </div>
                <div>
                  <Label htmlFor="webhookUrl">Webhook URL</Label>
                  <Input
                    id="webhookUrl"
                    value={newSandbox.webhookUrl}
                    onChange={(e) => setNewSandbox(prev => ({ ...prev, webhookUrl: e.target.value }))}
                    placeholder="https://your-app.com/webhooks"
                  />
                </div>
                <div>
                  <Label htmlFor="description">Description</Label>
                  <Textarea
                    id="description"
                    value={newSandbox.description}
                    onChange={(e) => setNewSandbox(prev => ({ ...prev, description: e.target.value }))}
                    placeholder="Optional description"
                  />
                </div>
              </div>
              <DialogFooter>
                <Button variant="outline" onClick={() => setIsCreateDialogOpen(false)}>
                  Cancel
                </Button>
                <Button onClick={() => setIsCreateDialogOpen(false)}>
                  Create Sandbox
                </Button>
              </DialogFooter>
            </DialogContent>
          </Dialog>
        </div>
      </div>

      <div className="grid gap-4 md:grid-cols-2 md:gap-8 lg:grid-cols-4 mt-6">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Sandboxes</CardTitle>
            <Users className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{sandboxes.length}</div>
            <p className="text-xs text-muted-foreground">
              total sandbox tenants
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Active</CardTitle>
            <Play className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{activeSandboxes}</div>
            <p className="text-xs text-muted-foreground">
              currently active
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Provisioning</CardTitle>
            <Settings className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{provisioningSandboxes}</div>
            <p className="text-xs text-muted-foreground">
              being set up
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Expired</CardTitle>
            <Pause className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{expiredSandboxes}</div>
            <p className="text-xs text-muted-foreground">
              need cleanup
            </p>
          </CardContent>
        </Card>
      </div>

      <Card className="mt-6">
        <CardHeader>
          <CardTitle>Sandbox Tenants</CardTitle>
          <CardDescription>
            Manage sandbox environments for testing and development
          </CardDescription>
        </CardHeader>
        <CardContent>
          {sandboxes.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              No sandboxes found
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>Created</TableHead>
                  <TableHead>Expires</TableHead>
                  <TableHead>Days Left</TableHead>
                  <TableHead>Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {sandboxes.map((sandbox) => {
                  const daysLeft = getDaysUntilExpiry(sandbox.expires_at)
                  return (
                    <TableRow key={sandbox.id}>
                      <TableCell className="font-medium">{sandbox.name}</TableCell>
                      <TableCell>{getStatusBadge(sandbox.status)}</TableCell>
                      <TableCell>
                        {new Date(sandbox.created_at).toLocaleDateString()}
                      </TableCell>
                      <TableCell>
                        {new Date(sandbox.expires_at).toLocaleDateString()}
                      </TableCell>
                      <TableCell>
                        <span className={daysLeft < 7 ? 'text-red-600 font-semibold' : daysLeft < 30 ? 'text-yellow-600' : ''}>
                          {daysLeft} days
                        </span>
                      </TableCell>
                      <TableCell>
                        <div className="flex items-center gap-2">
                          <Button variant="outline" size="sm">
                            <Settings className="h-4 w-4" />
                          </Button>
                          {sandbox.status === 'EXPIRED' && (
                            <Button variant="outline" size="sm">
                              <Trash2 className="h-4 w-4" />
                            </Button>
                          )}
                        </div>
                      </TableCell>
                    </TableRow>
                  )
                })}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      {cleanupResult && (
        <div className={`mt-4 p-4 rounded-md ${cleanupResult.success ? 'bg-green-50 text-green-800' : 'bg-red-50 text-red-800'}`}>
          {cleanupResult.message}
        </div>
      )}
    </AdminLayout>
  )
}