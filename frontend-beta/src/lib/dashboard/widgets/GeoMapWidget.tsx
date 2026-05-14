"use client";

import { Card } from "@/components/dashboard/primitives";
import { InteractiveMap } from "@/components/ui/interactive-map";
import type { WidgetProps } from "../types";

const DUMMY_HOTSPOTS = [
  { lat: 55.75, lng: 37.61, intensity: 1.0, label: "RU" },
  { lat: 35.86, lng: 104.19, intensity: 0.7, label: "CN" },
  { lat: 32.42, lng: 53.68, intensity: 0.5, label: "IR" },
  { lat: -0.78, lng: 113.92, intensity: 0.4, label: "ID" },
  { lat: -14.23, lng: -51.92, intensity: 0.3, label: "BR" },
];

function GeoMapWidget({ widgetId }: WidgetProps) {
  return (
    <Card
      className="h-full flex flex-col widget-drag-handle cursor-grab active:cursor-grabbing"
      title="Geo Threat Origins"
      subtitle="Real-time origin map"
      actions={
        <span className="flex items-center gap-1.5 text-[11px] text-muted font-medium uppercase tracking-widest">
          <span className="w-2 h-2 rounded-full bg-success animate-pulse" /> LIVE
        </span>
      }
    >
      <div className="flex-1 min-h-0 relative rounded-md overflow-hidden bg-transparent">
        <InteractiveMap hotspots={DUMMY_HOTSPOTS} />
      </div>
    </Card>
  );
}

export default GeoMapWidget;
