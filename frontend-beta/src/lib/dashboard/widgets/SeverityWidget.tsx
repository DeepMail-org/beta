"use client";

import { Card, Donut } from "@/components/dashboard/primitives";
import type { WidgetProps } from "../types";

const DUMMY_SEGMENTS = [
  { label: "Safe", value: 850, color: "rgba(255,255,255,0.7)" },
  { label: "Suspicious", value: 120, color: "#fbbf24" },
  { label: "Malicious", value: 30, color: "#f87171" },
];

function SeverityWidget({ widgetId }: WidgetProps) {
  return (
    <Card 
      title="Severity Breakdown"
      subtitle="Threats analyzed today"
      className="h-full"
    >
      <div className="h-full flex items-center justify-center pt-2">
        <Donut 
          segments={DUMMY_SEGMENTS} 
          size={140} 
          thickness={16} 
          centerValue="1,000" 
          centerLabel="TOTAL" 
        />
      </div>
      <div className="absolute top-0 left-0 right-0 h-14 widget-drag-handle cursor-grab active:cursor-grabbing z-10" />
    </Card>
  );
}

export default SeverityWidget;
