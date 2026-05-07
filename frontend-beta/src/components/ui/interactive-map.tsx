"use client";

import { useEffect, useState } from "react";
import dynamic from "next/dynamic";
import "leaflet/dist/leaflet.css";

// Dynamic import for the MapComponent to avoid SSR hydration errors with Leaflet
const MapComponent = dynamic(
  () => import("./map-component").then((mod) => mod.MapComponent),
  {
    ssr: false,
    loading: () => (
      <div className="w-full h-full flex items-center justify-center bg-surface/30 rounded-md animate-pulse">
        <span className="text-sm text-muted">Loading map data...</span>
      </div>
    ),
  }
);

export interface Hotspot {
  lat: number;
  lng: number;
  intensity: number;
  label: string;
}

interface InteractiveMapProps {
  hotspots: Hotspot[];
  className?: string;
}

export function InteractiveMap({ hotspots, className }: InteractiveMapProps) {
  // Fix Leaflet's default icon paths not resolving correctly in Next.js
  useEffect(() => {
    (async function init() {
      const L = await import("leaflet");
      delete (L.Icon.Default.prototype as any)._getIconUrl;
      L.Icon.Default.mergeOptions({
        iconRetinaUrl: "https://unpkg.com/leaflet@1.9.4/dist/images/marker-icon-2x.png",
        iconUrl: "https://unpkg.com/leaflet@1.9.4/dist/images/marker-icon.png",
        shadowUrl: "https://unpkg.com/leaflet@1.9.4/dist/images/marker-shadow.png",
      });
    })();
  }, []);

  return (
    <div className={`w-full h-full relative overflow-hidden ${className || ""}`}>
      <MapComponent hotspots={hotspots} />
    </div>
  );
}
