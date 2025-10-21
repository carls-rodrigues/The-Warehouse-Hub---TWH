"use client"

import AdminLayout from "@/components/admin-layout"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog"
import { Textarea } from "@/components/ui/textarea"
import { AlertTriangle, RefreshCw, Eye, Clock, CheckCircle } from "lucide-react"
import { useState } from "react"

interface DLQItem {
  id: string
  tenantId: string
  webhookUrl: string
  eventType: string
  failedAt: string
  errorMessage: string
  retryCount: number
  payload: string
}

export default function DLQManagement() {
  const [selectedItem, setSelectedItem] = useState<DLQItem | null>(null)
  const [isReplaying, setIsReplaying] = useState<string | null>(null)

  // Mock data - in real app this would come from API
  const dlqItems: DLQItem[] = [
    {
      id: "dlq-001",
      tenantId: "tenant-abc-123",
      webhookUrl: "https://api.example.com/webhooks/events",
      eventType: "inventory.updated",
      failedAt: "2024-01-15T10:30:00Z",
      errorMessage: "Connection timeout after 30 seconds",
      retryCount: 3,
      payload: '{"event":"inventory.updated","data":{"productId":"prod-123","quantity":50}}'
    },
    {
      id: "dlq-002",
      tenantId: "tenant-def-456",
      webhookUrl: "https://hooks.company.com/inventory",
      eventType: "order.created",
      failedAt: "2024-01-15T09:15:00Z",
      errorMessage: "HTTP 500 Internal Server Error",
      retryCount: 2,
      payload: '{"event":"order.created","data":{"orderId":"ord-456","total":299.99}}'
    },
    {
      id: "dlq-003",
      tenantId: "tenant-ghi-789",
      webhookUrl: "https://webhooks.acme.com/events",
      eventType: "shipment.delivered",
      failedAt: "2024-01-14T16:45:00Z",
      errorMessage: "SSL certificate verification failed",
      retryCount: 1,
      payload: '{"event":"shipment.delivered","data":{"shipmentId":"ship-789","tracking":"1Z999AA1234567890"}}'
    }
  ]

  const handleReplay = async (itemId: string) => {
    setIsReplaying(itemId)
    // Simulate API call
    await new Promise(resolve => setTimeout(resolve, 2000))
    setIsReplaying(null)
    // In real app, remove from DLQ or mark as resolved
  }

  return (
    <AdminLayout>
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold md:text-2xl">DLQ Management</h1>
          <p className="text-muted-foreground">Manage failed webhook deliveries and replay events</p>
        </div>
        <div className="flex items-center space-x-2">
          <Badge variant="destructive" className="flex items-center space-x-1">
            <AlertTriangle className="h-3 w-3" />
            <span>{dlqItems.length} Failed Deliveries</span>
          </Badge>
        </div>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Failed Webhook Deliveries</CardTitle>
          <CardDescription>
            Events that failed to deliver to tenant webhooks after all retry attempts
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Tenant</TableHead>
                <TableHead>Event Type</TableHead>
                <TableHead>Failed At</TableHead>
                <TableHead>Error</TableHead>
                <TableHead>Retries</TableHead>
                <TableHead>Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {dlqItems.map((item) => (
                <TableRow key={item.id}>
                  <TableCell className="font-mono text-sm">{item.tenantId}</TableCell>
                  <TableCell>
                    <Badge variant="outline">{item.eventType}</Badge>
                  </TableCell>
                  <TableCell>
                    <div className="flex items-center space-x-1">
                      <Clock className="h-3 w-3 text-muted-foreground" />
                      <span className="text-sm">
                        {new Date(item.failedAt).toLocaleString()}
                      </span>
                    </div>
                  </TableCell>
                  <TableCell className="max-w-xs">
                    <span className="text-sm text-muted-foreground truncate block">
                      {item.errorMessage}
                    </span>
                  </TableCell>
                  <TableCell>
                    <Badge variant={item.retryCount >= 3 ? "destructive" : "secondary"}>
                      {item.retryCount}
                    </Badge>
                  </TableCell>
                  <TableCell>
                    <div className="flex items-center space-x-2">
                      <Dialog>
                        <DialogTrigger asChild>
                          <Button variant="outline" size="sm" onClick={() => setSelectedItem(item)}>
                            <Eye className="h-3 w-3 mr-1" />
                            View
                          </Button>
                        </DialogTrigger>
                        <DialogContent className="max-w-2xl">
                          <DialogHeader>
                            <DialogTitle>DLQ Item Details</DialogTitle>
                            <DialogDescription>
                              Failed webhook delivery for {selectedItem?.eventType}
                            </DialogDescription>
                          </DialogHeader>
                          {selectedItem && (
                            <div className="space-y-4">
                              <div className="grid grid-cols-2 gap-4">
                                <div>
                                  <label className="text-sm font-medium">Tenant ID</label>
                                  <p className="font-mono text-sm">{selectedItem.tenantId}</p>
                                </div>
                                <div>
                                  <label className="text-sm font-medium">Event Type</label>
                                  <p className="text-sm">{selectedItem.eventType}</p>
                                </div>
                                <div>
                                  <label className="text-sm font-medium">Webhook URL</label>
                                  <p className="text-sm break-all">{selectedItem.webhookUrl}</p>
                                </div>
                                <div>
                                  <label className="text-sm font-medium">Failed At</label>
                                  <p className="text-sm">{new Date(selectedItem.failedAt).toLocaleString()}</p>
                                </div>
                              </div>
                              <div>
                                <label className="text-sm font-medium">Error Message</label>
                                <p className="text-sm text-red-600 mt-1">{selectedItem.errorMessage}</p>
                              </div>
                              <div>
                                <label className="text-sm font-medium">Payload</label>
                                <Textarea
                                  value={JSON.stringify(JSON.parse(selectedItem.payload), null, 2)}
                                  readOnly
                                  className="mt-1 font-mono text-xs"
                                  rows={6}
                                />
                              </div>
                            </div>
                          )}
                          <DialogFooter>
                            <Button
                              onClick={() => selectedItem && handleReplay(selectedItem.id)}
                              disabled={isReplaying === selectedItem?.id}
                            >
                              {isReplaying === selectedItem?.id ? (
                                <>
                                  <RefreshCw className="h-3 w-3 mr-1 animate-spin" />
                                  Replaying...
                                </>
                              ) : (
                                <>
                                  <RefreshCw className="h-3 w-3 mr-1" />
                                  Replay Event
                                </>
                              )}
                            </Button>
                          </DialogFooter>
                        </DialogContent>
                      </Dialog>

                      <Button
                        size="sm"
                        onClick={() => handleReplay(item.id)}
                        disabled={isReplaying === item.id}
                      >
                        {isReplaying === item.id ? (
                          <RefreshCw className="h-3 w-3 animate-spin" />
                        ) : (
                          <RefreshCw className="h-3 w-3" />
                        )}
                      </Button>
                    </div>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>

          {dlqItems.length === 0 && (
            <div className="text-center py-8">
              <CheckCircle className="h-12 w-12 text-green-500 mx-auto mb-4" />
              <h3 className="text-lg font-medium">No failed deliveries</h3>
              <p className="text-muted-foreground">All webhook deliveries are successful</p>
            </div>
          )}
        </CardContent>
      </Card>
    </AdminLayout>
  )
}