"use client";

import { Card, RadarChart } from "@/components/dashboard/primitives";
import type { WidgetProps } from "../types";

const DUMMY_AXES = [
  { label: "Phishing", value: 0.92 },
  { label: "Malware", value: 0.64 },
  { label: "BEC", value: 0.78 },
  { label: "Spoofing", value: 0.55 },
  { label: "Spam", value: 0.42 },
  { label: "Recon", value: 0.31 },
];

function VectorWidget({ widgetId }: WidgetProps) {
  return (
    <Card 
      title="Attack Vectors"
      subtitle="Distribution of attack categories"
      className="h-full"
    >
      <div className="h-full flex items-center justify-center pt-2">
        <RadarChart axes={DUMMY_AXES} size={150} />
      </div>
      <div className="absolute top-0 left-0 right-0 h-14 widget-drag-handle cursor-grab active:cursor-grabbing z-10" />
    </Card>
  );
}

export default VectorWidget;
