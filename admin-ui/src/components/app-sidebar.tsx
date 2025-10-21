"use client"

import * as React from "react"
import {
  Frame,
  GalleryVerticalEnd,
  Map,
  PieChart,
  SquareTerminal,
  Database,
  Webhook,
  CreditCard,
  Users,
} from "lucide-react"

import { NavMain } from "@/components/nav-main"
import { NavProjects } from "@/components/nav-projects"
import { NavUser } from "@/components/nav-user"
import { TeamSwitcher } from "@/components/team-switcher"
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarRail,
} from "@/components/ui/sidebar"

// This is sample data.
const data = {
  user: {
    name: "Admin User",
    email: "admin@twh.com",
    avatar: "/avatars/admin.jpg",
  },
  teams: [
    {
      name: "The Warehouse Hub",
      logo: GalleryVerticalEnd,
      plan: "Admin Console",
    },
  ],
  navMain: [
    {
      title: "Dashboard",
      url: "/admin",
      icon: SquareTerminal,
      isActive: true,
    },
    {
      title: "DLQ Management",
      url: "/admin/dlq",
      icon: Database,
    },
    {
      title: "Sandbox Management",
      url: "/admin/sandbox",
      icon: Users,
    },
    {
      title: "Billing & Usage",
      url: "/admin/billing",
      icon: CreditCard,
    },
    {
      title: "Webhooks",
      url: "/admin/webhooks",
      icon: Webhook,
    },
  ],
  projects: [
    {
      name: "Production",
      url: "#",
      icon: Frame,
    },
    {
      name: "Staging",
      url: "#",
      icon: PieChart,
    },
    {
      name: "Development",
      url: "#",
      icon: Map,
    },
  ],
}

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  return (
    <Sidebar collapsible="icon" {...props}>
      <SidebarHeader>
        <TeamSwitcher teams={data.teams} />
      </SidebarHeader>
      <SidebarContent>
        <NavMain items={data.navMain} />
        <NavProjects projects={data.projects} />
      </SidebarContent>
      <SidebarFooter>
        <NavUser user={data.user} />
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  )
}