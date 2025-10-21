import AdminLayout from "@/components/admin-layout"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Progress } from "@/components/ui/progress"
import { DollarSign, TrendingUp, Users, CreditCard, Download, Calendar } from "lucide-react"

interface Invoice {
  id: string
  tenantId: string
  tenantName: string
  amount: number
  status: 'paid' | 'pending' | 'overdue'
  issuedAt: string
  dueAt: string
  period: string
}

interface UsageMetric {
  tenantId: string
  tenantName: string
  webhooksSent: number
  webhooksFailed: number
  dataTransferred: number // in MB
  apiCalls: number
  currentMonthSpend: number
}

export default function BillingManagement() {
  // Mock data - in real app this would come from API
  const invoices: Invoice[] = [
    {
      id: "inv-001",
      tenantId: "tenant-abc-123",
      tenantName: "E-commerce Corp",
      amount: 1250.00,
      status: "paid",
      issuedAt: "2024-01-01T00:00:00Z",
      dueAt: "2024-01-15T00:00:00Z",
      period: "December 2023"
    },
    {
      id: "inv-002",
      tenantId: "tenant-def-456",
      tenantName: "Logistics Plus",
      amount: 890.50,
      status: "paid",
      issuedAt: "2024-01-01T00:00:00Z",
      dueAt: "2024-01-15T00:00:00Z",
      period: "December 2023"
    },
    {
      id: "inv-003",
      tenantId: "tenant-ghi-789",
      tenantName: "Payment Solutions Inc",
      amount: 2100.75,
      status: "pending",
      issuedAt: "2024-01-15T00:00:00Z",
      dueAt: "2024-01-30T00:00:00Z",
      period: "January 2024"
    },
    {
      id: "inv-004",
      tenantId: "tenant-jkl-012",
      tenantName: "Retail Chain Ltd",
      amount: 675.25,
      status: "overdue",
      issuedAt: "2024-01-01T00:00:00Z",
      dueAt: "2024-01-15T00:00:00Z",
      period: "December 2023"
    }
  ]

  const usageMetrics: UsageMetric[] = [
    {
      tenantId: "tenant-abc-123",
      tenantName: "E-commerce Corp",
      webhooksSent: 45230,
      webhooksFailed: 45,
      dataTransferred: 1250.5,
      apiCalls: 89200,
      currentMonthSpend: 1250.00
    },
    {
      tenantId: "tenant-def-456",
      tenantName: "Logistics Plus",
      webhooksSent: 28900,
      webhooksFailed: 12,
      dataTransferred: 780.2,
      apiCalls: 45600,
      currentMonthSpend: 890.50
    },
    {
      tenantId: "tenant-ghi-789",
      tenantName: "Payment Solutions Inc",
      webhooksSent: 67800,
      webhooksFailed: 23,
      dataTransferred: 2100.8,
      apiCalls: 125000,
      currentMonthSpend: 2100.75
    }
  ]

  const totalRevenue = invoices.filter(inv => inv.status === 'paid').reduce((sum, inv) => sum + inv.amount, 0)
  const pendingRevenue = invoices.filter(inv => inv.status === 'pending').reduce((sum, inv) => sum + inv.amount, 0)
  const overdueRevenue = invoices.filter(inv => inv.status === 'overdue').reduce((sum, inv) => sum + inv.amount, 0)

  const getStatusBadge = (status: Invoice['status']) => {
    switch (status) {
      case 'paid':
        return <Badge className="bg-green-100 text-green-800">Paid</Badge>
      case 'pending':
        return <Badge variant="secondary">Pending</Badge>
      case 'overdue':
        return <Badge variant="destructive">Overdue</Badge>
    }
  }

  return (
    <AdminLayout>
      <div className="flex items-center">
        <h1 className="text-lg font-semibold md:text-2xl">Billing & Usage</h1>
      </div>

      <div className="grid gap-4 md:gap-8 lg:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Revenue</CardTitle>
            <DollarSign className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">${totalRevenue.toFixed(2)}</div>
            <p className="text-xs text-muted-foreground">
              +12% from last month
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Pending Payments</CardTitle>
            <CreditCard className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">${pendingRevenue.toFixed(2)}</div>
            <p className="text-xs text-muted-foreground">
              Due this month
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Overdue</CardTitle>
            <TrendingUp className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">${overdueRevenue.toFixed(2)}</div>
            <p className="text-xs text-muted-foreground">
              Requires attention
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Active Tenants</CardTitle>
            <Users className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{usageMetrics.length}</div>
            <p className="text-xs text-muted-foreground">
              Paying customers
            </p>
          </CardContent>
        </Card>
      </div>

      <Tabs defaultValue="invoices" className="space-y-4">
        <TabsList>
          <TabsTrigger value="invoices">Invoices</TabsTrigger>
          <TabsTrigger value="usage">Usage Analytics</TabsTrigger>
        </TabsList>

        <TabsContent value="invoices" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Invoice Management</CardTitle>
              <CardDescription>
                View and manage customer invoices and payment status
              </CardDescription>
            </CardHeader>
            <CardContent>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Invoice ID</TableHead>
                    <TableHead>Customer</TableHead>
                    <TableHead>Period</TableHead>
                    <TableHead>Amount</TableHead>
                    <TableHead>Status</TableHead>
                    <TableHead>Due Date</TableHead>
                    <TableHead>Actions</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {invoices.map((invoice) => (
                    <TableRow key={invoice.id}>
                      <TableCell className="font-mono text-sm">{invoice.id}</TableCell>
                      <TableCell>
                        <div>
                          <div className="font-medium">{invoice.tenantName}</div>
                          <div className="text-sm text-muted-foreground">{invoice.tenantId}</div>
                        </div>
                      </TableCell>
                      <TableCell>{invoice.period}</TableCell>
                      <TableCell className="font-medium">${invoice.amount.toFixed(2)}</TableCell>
                      <TableCell>{getStatusBadge(invoice.status)}</TableCell>
                      <TableCell>
                        <div className="flex items-center space-x-1">
                          <Calendar className="h-3 w-3 text-muted-foreground" />
                          <span className="text-sm">
                            {new Date(invoice.dueAt).toLocaleDateString()}
                          </span>
                        </div>
                      </TableCell>
                      <TableCell>
                        <Button variant="outline" size="sm">
                          <Download className="h-3 w-3 mr-1" />
                          Download
                        </Button>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="usage" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Usage Analytics</CardTitle>
              <CardDescription>
                Monitor tenant usage and spending patterns
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-6">
                {usageMetrics.map((metric) => (
                  <div key={metric.tenantId} className="space-y-4">
                    <div className="flex items-center justify-between">
                      <div>
                        <h3 className="font-medium">{metric.tenantName}</h3>
                        <p className="text-sm text-muted-foreground">{metric.tenantId}</p>
                      </div>
                      <div className="text-right">
                        <div className="text-lg font-bold">${metric.currentMonthSpend.toFixed(2)}</div>
                        <p className="text-xs text-muted-foreground">Current month</p>
                      </div>
                    </div>

                    <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                      <div>
                        <div className="text-sm font-medium">Webhooks Sent</div>
                        <div className="text-2xl font-bold">{metric.webhooksSent.toLocaleString()}</div>
                        <Progress value={(metric.webhooksSent / 100000) * 100} className="mt-2" />
                      </div>

                      <div>
                        <div className="text-sm font-medium">Success Rate</div>
                        <div className="text-2xl font-bold">
                          {((metric.webhooksSent - metric.webhooksFailed) / metric.webhooksSent * 100).toFixed(1)}%
                        </div>
                        <Progress
                          value={((metric.webhooksSent - metric.webhooksFailed) / metric.webhooksSent * 100)}
                          className="mt-2"
                        />
                      </div>

                      <div>
                        <div className="text-sm font-medium">Data Transferred</div>
                        <div className="text-2xl font-bold">{metric.dataTransferred.toFixed(1)} MB</div>
                        <Progress value={(metric.dataTransferred / 2500) * 100} className="mt-2" />
                      </div>

                      <div>
                        <div className="text-sm font-medium">API Calls</div>
                        <div className="text-2xl font-bold">{metric.apiCalls.toLocaleString()}</div>
                        <Progress value={(metric.apiCalls / 150000) * 100} className="mt-2" />
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </AdminLayout>
  )
}