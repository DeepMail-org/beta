import { lazy } from "react";
import type { WidgetDefinition } from "../types";
import { widgetRegistry } from "../registry";

const TimelineWidget = lazy(() => import("./TimelineWidget"));
const SeverityWidget = lazy(() => import("./SeverityWidget"));
const VectorWidget = lazy(() => import("./VectorWidget"));
const TopSendersWidget = lazy(() => import("./TopSendersWidget"));
const GeoMapWidget = lazy(() => import("./GeoMapWidget"));
const RecentThreatsWidget = lazy(() => import("./RecentThreatsWidget"));

import {
  LineChart,
  PieChart,
  Radar,
  MailWarning,
  Globe,
  AlertTriangle,
} from "lucide-react";

export const builtinWidgets: WidgetDefinition[] = [
  {
    id: "timeline",
    name: "Threat Timeline",
    description: "Daily email threat volume over the past week with trend visualization",
    icon: LineChart,
    category: "analytics",
    defaultSize: { w: 6, h: 2 },
    minSize: { w: 4, h: 2 },
    maxSize: { w: 12, h: 4 },
    dataSource: { type: "polling", endpoint: "/dashboard", interval: 30_000 },
    component: TimelineWidget,
  },
  {
    id: "severity",
    name: "Severity Breakdown",
    description: "Distribution of email threats by severity level (Critical, High, Medium, Safe)",
    icon: PieChart,
    category: "analytics",
    defaultSize: { w: 3, h: 2 },
    minSize: { w: 2, h: 2 },
    maxSize: { w: 6, h: 4 },
    dataSource: { type: "polling", endpoint: "/dashboard", interval: 30_000 },
    component: SeverityWidget,
  },
  {
    id: "vector",
    name: "Attack Vectors",
    description: "Radar chart showing threat distribution across attack categories",
    icon: Radar,
    category: "threats",
    defaultSize: { w: 4, h: 2 },
    minSize: { w: 3, h: 2 },
    maxSize: { w: 6, h: 4 },
    dataSource: { type: "polling", endpoint: "/dashboard", interval: 60_000 },
    component: VectorWidget,
  },
  {
    id: "top-senders",
    name: "Top Malicious Senders",
    description: "Ranked list of the most frequently flagged sender addresses",
    icon: MailWarning,
    category: "threats",
    defaultSize: { w: 4, h: 2 },
    minSize: { w: 3, h: 2 },
    maxSize: { w: 6, h: 3 },
    dataSource: { type: "polling", endpoint: "/dashboard", interval: 60_000 },
    component: TopSendersWidget,
  },
  {
    id: "geo-map",
    name: "Geo Threat Origins",
    description: "Real-time map showing geographic origins of threat activity",
    icon: Globe,
    category: "intelligence",
    defaultSize: { w: 6, h: 3 },
    minSize: { w: 4, h: 2 },
    maxSize: { w: 12, h: 5 },
    dataSource: { type: "websocket", channel: "threats" },
    component: GeoMapWidget,
  },
  {
    id: "recent-threats",
    name: "Recent Threats",
    description: "Live feed of the latest detected threats with severity scores",
    icon: AlertTriangle,
    category: "threats",
    defaultSize: { w: 4, h: 3 },
    minSize: { w: 3, h: 2 },
    maxSize: { w: 6, h: 5 },
    dataSource: { type: "websocket", channel: "alerts" },
    component: RecentThreatsWidget,
  },
];

export function registerBuiltinWidgets(): void {
  for (const widget of builtinWidgets) {
    widgetRegistry.register(widget);
  }
}
