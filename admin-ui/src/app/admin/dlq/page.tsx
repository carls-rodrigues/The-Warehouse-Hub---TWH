"use client"

import AdminLayout from "@/components/admin-layout"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog"
import { Textarea } from "@/components/ui/textarea"
import { AlertTriangle, RefreshCw, Eye, Clock, CheckCircle, AlertCircle } from "lucide-react"
import { useEffect, useState } from "react"
import { adminApi, ApiError } from "@/lib/api"

interface DLQDelivery {
  id: string
  webhook_id: string
  event_id: string
  status: 'PENDING' | 'SUCCESS' | 'FAILED' | 'TIMEOUT' | 'DLQ'
  attempt_count: number
  last_attempt_at?: string
  response_status?: number
  error_message?: string
  created_at: string
}

interface DLQResponse {
  deliveries: DLQDelivery[]
  pagination: {
    page: number
    per_page: number
    total: number
    total_pages: number
  }
}

export default function DLQManagement() {
  const [dlqData, setDlqData] = useState<DLQResponse | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [selectedItem, setSelectedItem] = useState<DLQDelivery | null>(null)
  const [isReplaying, setIsReplaying] = useState<string | null>(null)
  const [replayResult, setReplayResult] = useState<{ success: boolean; message: string } | null>(null)

  useEffect(() => {
    fetchDlqDeliveries()
  }, [])

  const fetchDlqDeliveries = async () => {
    try {
      setLoading(true)
      const data = await adminApi.getDlqDeliveries()
      setDlqData(data)
    } catch (err) {
      if (err instanceof ApiError) {
        setError(`Failed to load DLQ deliveries: ${err.message}`)
      } else {
        setError('Failed to load DLQ deliveries')
      }
    } finally {
      setLoading(false)
    }
  }

  const handleReplay = async (deliveryId: string) => {
    try {
      setIsReplaying(deliveryId)
      setReplayResult(null)
      const result = await adminApi.replayDlqDelivery(deliveryId)
      setReplayResult({ success: result.success, message: result.message })

      // Refresh the list after replay
      await fetchDlqDeliveries()
    } catch (err) {
      if (err instanceof ApiError) {
        setReplayResult({ success: false, message: `Failed: ${err.message}` })
      } else {
        setReplayResult({ success: false, message: 'Failed to replay delivery' })
      }
    } finally {
      setIsReplaying(null)
    }
  }

  const getStatusBadge = (status: DLQDelivery['status']) => {
    switch (status) {
      case 'FAILED':
        return <Badge variant="destructive">Failed</Badge>
      case 'TIMEOUT':
        return <Badge variant="secondary">Timeout</Badge>
      case 'DLQ':
        return <Badge className="bg-orange-100 text-orange-800">In DLQ</Badge>
      case 'PENDING':
        return <Badge variant="secondary">Pending</Badge>
      case 'SUCCESS':
        return <Badge className="bg-green-100 text-green-800">Success</Badge>
      default:
        return <Badge variant="outline">{status}</Badge>
    }
  }

  if (loading) {
    return (
      <AdminLayout>
        <div className="flex items-center justify-center h-64">
          <div className="text-lg">Loading DLQ deliveries...</div>
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

  const deliveries = dlqData?.deliveries || []

  return (
    <AdminLayout>
      <div className="flex items-center justify-between">
        <h1 className="text-lg font-semibold md:text-2xl">DLQ Management</h1>
        <Button onClick={fetchDlqDeliveries} variant="outline" size="sm">
          <RefreshCw className="mr-2 h-4 w-4" />
          Refresh
        </Button>
      </div>

      <div className="grid gap-4 md:grid-cols-2 md:gap-8 lg:grid-cols-4 mt-6">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Failed Deliveries</CardTitle>
            <AlertTriangle className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{dlqData?.pagination.total || 0}</div>
            <p className="text-xs text-muted-foreground">
              deliveries in DLQ
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Failed Today</CardTitle>
            <Clock className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {deliveries.filter(d => {
                const today = new Date().toDateString()
                return new Date(d.created_at).toDateString() === today
              }).length}
            </div>
            <p className="text-xs text-muted-foreground">
              failed deliveries today
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Avg Retry Count</CardTitle>
            <RefreshCw className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {deliveries.length > 0
                ? (deliveries.reduce((sum, d) => sum + d.attempt_count, 0) / deliveries.length).toFixed(1)
                : '0'
              }
            </div>
            <p className="text-xs text-muted-foreground">
              attempts per delivery
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Success Rate</CardTitle>
            <CheckCircle className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {deliveries.length > 0
                ? ((deliveries.filter(d => d.status === 'SUCCESS').length / deliveries.length) * 100).toFixed(1)
                : '0'
              }%
            </div>
            <p className="text-xs text-muted-foreground">
              replay success rate
            </p>
          </CardContent>
        </Card>
      </div>

      <Card className="mt-6">
        <CardHeader>
          <CardTitle>Failed Webhook Deliveries</CardTitle>
          <CardDescription>
            Review and manually replay failed webhook deliveries from the Dead Letter Queue
          </CardDescription>
        </CardHeader>
        <CardContent>
          {deliveries.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              No failed deliveries in the DLQ
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Event ID</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>Attempts</TableHead>
                  <TableHead>Created</TableHead>
                  <TableHead>Error</TableHead>
                  <TableHead>Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {deliveries.map((delivery) => (
                  <TableRow key={delivery.id}>
                    <TableCell className="font-mono text-sm">
                      {delivery.event_id.slice(0, 8)}...
                    </TableCell>
                    <TableCell>{getStatusBadge(delivery.status)}</TableCell>
                    <TableCell>{delivery.attempt_count}</TableCell>
                    <TableCell>
                      {new Date(delivery.created_at).toLocaleString()}
                    </TableCell>
                    <TableCell className="max-w-xs truncate">
                      {delivery.error_message || 'N/A'}
                    </TableCell>
                    <TableCell>
                      <div className="flex items-center gap-2">
                        <Dialog>
                          <DialogTrigger asChild>
                            <Button
                              variant="outline"
                              size="sm"
                              onClick={() => setSelectedItem(delivery)}
                            >
                              <Eye className="mr-2 h-4 w-4" />
                              Details
                            </Button>
                          </DialogTrigger>
                          <DialogContent className="max-w-2xl">
                            <DialogHeader>
                              <DialogTitle>Delivery Details</DialogTitle>
                              <DialogDescription>
                                Failed webhook delivery information
                              </DialogDescription>
                            </DialogHeader>
                            <div className="space-y-4">
                              <div className="grid grid-cols-2 gap-4">
                                <div>
                                  <label className="text-sm font-medium">Delivery ID</label>
                                  <p className="font-mono text-sm">{selectedItem?.id}</p>
                                </div>
                                <div>
                                  <label className="text-sm font-medium">Event ID</label>
                                  <p className="font-mono text-sm">{selectedItem?.event_id}</p>
                                </div>
                                <div>
                                  <label className="text-sm font-medium">Webhook ID</label>
                                  <p className="font-mono text-sm">{selectedItem?.webhook_id}</p>
                                </div>
                                <div>
                                  <label className="text-sm font-medium">Status</label>
                                  <div className="mt-1">
                                    {selectedItem && getStatusBadge(selectedItem.status)}
                                  </div>
                                </div>
                                <div>
                                  <label className="text-sm font-medium">Attempt Count</label>
                                  <p>{selectedItem?.attempt_count}</p>
                                </div>
                                <div>
                                  <label className="text-sm font-medium">Created At</label>
                                  <p>{selectedItem && new Date(selectedItem.created_at).toLocaleString()}</p>
                                </div>
                              </div>
                              {selectedItem?.last_attempt_at && (
                                <div>
                                  <label className="text-sm font-medium">Last Attempt</label>
                                  <p>{new Date(selectedItem.last_attempt_at).toLocaleString()}</p>
                                </div>
                              )}
                              {selectedItem?.response_status && (
                                <div>
                                  <label className="text-sm font-medium">Response Status</label>
                                  <p>{selectedItem.response_status}</p>
                                </div>
                              )}
                              {selectedItem?.error_message && (
                                <div>
                                  <label className="text-sm font-medium">Error Message</label>
                                  <Textarea
                                    value={selectedItem.error_message}
                                    readOnly
                                    className="mt-1"
                                  />
                                </div>
                              )}
                            </div>
                            <DialogFooter>
                              <Button
                                onClick={() => selectedItem && handleReplay(selectedItem.id)}
                                disabled={isReplaying === selectedItem?.id}
                              >
                                {isReplaying === selectedItem?.id ? (
                                  <>
                                    <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                                    Replaying...
                                  </>
                                ) : (
                                  <>
                                    <RefreshCw className="mr-2 h-4 w-4" />
                                    Replay Delivery
                                  </>
                                )}
                              </Button>
                            </DialogFooter>
                          </DialogContent>
                        </Dialog>

                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => handleReplay(delivery.id)}
                          disabled={isReplaying === delivery.id}
                        >
                          {isReplaying === delivery.id ? (
                            <RefreshCw className="h-4 w-4 animate-spin" />
                          ) : (
                            <RefreshCw className="h-4 w-4" />
                          )}
                        </Button>
                      </div>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      {replayResult && (
        <div className={`mt-4 p-4 rounded-md ${replayResult.success ? 'bg-green-50 text-green-800' : 'bg-red-50 text-red-800'}`}>
          {replayResult.message}
        </div>
      )}
    </AdminLayout>
  )
}