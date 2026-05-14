"use client";

import { Card, Severity, Btn } from "@/components/dashboard/primitives";
import type { WidgetProps } from "../types";
import type { SeverityLevel } from "@/components/dashboard/primitives";

interface Threat {
  id: string;
  subject: string;
  sender: string;
  level: SeverityLevel;
  score: number;
  when: string;
}

const DUMMY_THREATS: Threat[] = [
  { id: "ALR-7821", subject: "URGENT: Wire transfer authorization required", sender: "cfo@acme-corp.tk", level: "critical", score: 96, when: "2m ago" },
  { id: "ALR-7820", subject: "Your Apple ID password was reset", sender: "no-reply@apple-id.support", level: "critical", score: 91, when: "9m ago" },
  { id: "ALR-7819", subject: "Invoice #1224 — overdue", sender: "billing@vendor-portal.app", level: "warning", score: 78, when: "17m ago" },
  { id: "ALR-7818", subject: "Shared file: Q3-Forecast.xlsx", sender: "alerts@dropbox-share.app", level: "warning", score: 71, when: "24m ago" },
  { id: "ALR-7817", subject: "Re: Vendor onboarding documents", sender: "ops@known-vendor.com", level: "caution", score: 54, when: "38m ago" },
];

function RecentThreatsWidget({ widgetId }: WidgetProps) {
  return (
    <Card
      className="h-full flex flex-col widget-drag-handle cursor-grab active:cursor-grabbing"
      title="Recent Threats"
      subtitle="Click to inspect"
      actions={<Btn size="sm">View All</Btn>}
    >
      <ul className="space-y-2 overflow-y-auto flex-1 min-h-0">
        {DUMMY_THREATS.map((t) => (
          <li
            key={t.id}
            className="p-3 rounded-xl border border-transparent hover:border-[rgba(255,200,60,0.1)] hover:bg-white/[0.02] transition-all cursor-pointer"
          >
            <div className="flex items-center justify-between gap-2 mb-1">
              <span className="text-[10px] font-mono text-muted">{t.id}</span>
              <Severity level={t.level} />
            </div>
            <div className="text-xs font-medium truncate text-white">{t.subject}</div>
            <div className="flex items-center justify-between gap-2 mt-2 text-[10px] text-muted">
              <span className="truncate text-secondary">{t.sender}</span>
              <span>{t.when}</span>
            </div>
          </li>
        ))}
      </ul>
    </Card>
  );
}

export default RecentThreatsWidget;
