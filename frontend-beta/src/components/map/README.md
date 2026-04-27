# `frontend-beta/src/components/map` Folder Guide

This folder contains all map-specific UI components used by the frontend prototype.

## Files

| File | Main export | Purpose |
| --- | --- | --- |
| `WorldMap.tsx` | `WorldMap` | Dynamic import boundary that keeps Leaflet out of server-side rendering paths. |
| `WorldMapCanvas.tsx` | `WorldMapCanvas` | Main map canvas that renders markers, hop paths, zooming, and interaction state. |
| `IpMarker.tsx` | `IpMarker` | Marker component for one IP/hop point with risk-based presentation. |
| `IpSidebar.tsx` | `IpSidebar` | Sidebar panel for the selected IP or hop node. |
| `HopTimeline.tsx` | `HopTimeline` | Timeline/playback UI for walking the received-hop chain. |

## Data Contract

These components depend on backend-provided analysis data, especially:

- `geo_points`
- `hop_timeline`

The folder intentionally avoids client-side IP geolocation lookups.

## Editing Guidance

- Keep Leaflet-specific concerns isolated here.
- If the backend result contract changes, update this README and `src/lib/contracts/results.ts` together.
