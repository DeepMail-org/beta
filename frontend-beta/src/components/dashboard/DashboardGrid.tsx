"use client";

import { useCallback, useEffect } from "react";
import {
  Responsive,
  useContainerWidth,
  type Layout,
  type ResponsiveLayouts,
} from "react-grid-layout";
import { fastVerticalCompactor } from "react-grid-layout/extras";
import { useDashboardStore } from "@/stores/dashboard-store";
import { WidgetSlot } from "./WidgetSlot";
import {
  BREAKPOINTS,
  COLS,
  ROW_HEIGHT,
  GRID_MARGIN,
  type DashboardLayoutState,
  type WidgetLayoutItem,
} from "@/lib/dashboard/types";
import "react-grid-layout/css/styles.css";

export function DashboardGrid() {
  const { width, containerRef, mounted } = useContainerWidth({
    measureBeforeMount: true,
    initialWidth: 1280,
  });

  const activeWidgets = useDashboardStore((s) => s.activeWidgets);
  const layouts = useDashboardStore((s) => s.layouts);
  const updateAllLayouts = useDashboardStore((s) => s.updateAllLayouts);
  const hydrate = useDashboardStore((s) => s.hydrate);

  useEffect(() => {
    hydrate();
  }, [hydrate]);

  const handleLayoutChange = useCallback(
    (_currentLayout: Layout, allLayouts: ResponsiveLayouts) => {
      const mapped: DashboardLayoutState = {
        lg: (allLayouts.lg ?? []) as unknown as WidgetLayoutItem[],
        md: (allLayouts.md ?? []) as unknown as WidgetLayoutItem[],
        sm: (allLayouts.sm ?? []) as unknown as WidgetLayoutItem[],
        xs: (allLayouts.xs ?? []) as unknown as WidgetLayoutItem[],
      };
      updateAllLayouts(mapped);
    },
    [updateAllLayouts]
  );

  return (
    <div ref={containerRef} className="w-full min-h-100">
      {mounted && activeWidgets.length > 0 && (
        <Responsive
          width={width}
          layouts={layouts as unknown as ResponsiveLayouts}
          breakpoints={BREAKPOINTS}
          cols={COLS}
          rowHeight={ROW_HEIGHT}
          margin={GRID_MARGIN}
          containerPadding={[0, 0]}
          dragConfig={{ enabled: true, handle: ".widget-drag-handle" }}
          resizeConfig={{ enabled: true, handles: ["se"] }}
          compactor={fastVerticalCompactor}
          onLayoutChange={handleLayoutChange}
        >
          {activeWidgets.map((widget) => (
            <div key={widget.id} className="h-full">
              <WidgetSlot widgetId={widget.id} definitionId={widget.definitionId} />
            </div>
          ))}
        </Responsive>
      )}
      {mounted && activeWidgets.length === 0 && (
        <EmptyDashboard />
      )}
    </div>
  );
}

function EmptyDashboard() {
  const setMarketplaceOpen = useDashboardStore((s) => s.setMarketplaceOpen);

  return (
    <div className="flex flex-col items-center justify-center py-24 gap-4">
      <div className="w-16 h-16 rounded-2xl bg-surface-2 border border-border flex items-center justify-center">
        <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" className="text-muted">
          <rect x="3" y="3" width="7" height="7" rx="1" />
          <rect x="14" y="3" width="7" height="7" rx="1" />
          <rect x="3" y="14" width="7" height="7" rx="1" />
          <rect x="14" y="14" width="7" height="7" rx="1" />
        </svg>
      </div>
      <div className="text-center">
        <p className="text-sm font-medium text-foreground">No widgets added</p>
        <p className="text-xs text-muted mt-1">Add widgets from the marketplace to build your dashboard</p>
      </div>
      <button
        type="button"
        onClick={() => setMarketplaceOpen(true)}
        className="text-xs px-4 py-2 rounded-lg bg-accent text-white hover:bg-accent/90 transition-colors"
      >
        Open Marketplace
      </button>
    </div>
  );
}
