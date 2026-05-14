"use client";

import { Card, LineChart } from "@/components/dashboard/primitives";
import type { WidgetProps } from "../types";

const DUMMY_DATA = [
  { label: "Mon", value: 120 },
  { label: "Tue", value: 200 },
  { label: "Wed", value: 150 },
  { label: "Thu", value: 300 },
  { label: "Fri", value: 250 },
  { label: "Sat", value: 80 },
  { label: "Sun", value: 110 },
];

function TimelineWidget({ widgetId }: WidgetProps) {
  return (
    <Card 
      title="Threat Volume Trend" 
      subtitle="Last 7 days of email traffic"
      className="h-full"
    >
      <div className="h-full flex items-end">
        <LineChart data={DUMMY_DATA} height={180} />
      </div>
      <div className="absolute top-0 left-0 right-0 h-14 widget-drag-handle cursor-grab active:cursor-grabbing z-10" />
    </Card>
  );
}

export default TimelineWidget;
