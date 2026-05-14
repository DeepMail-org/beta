"use client";

import { Card, BarChart } from "@/components/dashboard/primitives";
import type { WidgetProps } from "../types";

const DUMMY_SENDERS = [
  { label: "apple-id.support", value: 47, tone: "danger" as const },
  { label: "paypal-secure.io", value: 38, tone: "warn" as const },
  { label: "acme-corp.tk", value: 29, tone: "default" as const },
  { label: "dropbox-share.app", value: 24, tone: "default" as const },
  { label: "office365-login.io", value: 19, tone: "default" as const },
];

function TopSendersWidget({ widgetId }: WidgetProps) {
  return (
    <Card 
      title="Top Malicious Domains"
      subtitle="Most frequent sources of threats"
      className="h-full"
    >
      <div className="h-full flex items-end pb-2">
        <BarChart data={DUMMY_SENDERS} height={140} />
      </div>
      <div className="absolute top-0 left-0 right-0 h-14 widget-drag-handle cursor-grab active:cursor-grabbing z-10" />
    </Card>
  );
}

export default TopSendersWidget;
