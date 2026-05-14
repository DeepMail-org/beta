"use client";

import { Suspense, memo, useCallback } from "react";
import { WidgetErrorBoundary } from "./WidgetErrorBoundary";
import { WidgetSkeleton } from "./WidgetSkeleton";
import { useWidgetData } from "@/hooks/useWidgetData";
import { useDashboardStore } from "@/stores/dashboard-store";
import { widgetRegistry } from "@/lib/dashboard/registry";
import type { WidgetDefinition } from "@/lib/dashboard/types";

interface WidgetSlotProps {
  widgetId: string;
  definitionId: string;
}

function WidgetSlotInner({ widgetId, definitionId }: WidgetSlotProps) {
  const definition = widgetRegistry.get(definitionId);
  if (!definition) {
    return (
      <div className="h-full w-full rounded-xl border border-border bg-surface/50 flex items-center justify-center text-sm text-muted">
        Unknown widget: {definitionId}
      </div>
    );
  }

  return (
    <WidgetErrorBoundary widgetId={widgetId}>
      <Suspense fallback={<WidgetSkeleton />}>
        <WidgetContent widgetId={widgetId} definition={definition} />
      </Suspense>
    </WidgetErrorBoundary>
  );
}

function WidgetContent({ widgetId, definition }: { widgetId: string; definition: WidgetDefinition }) {
  const { data, isLoading, error, refetch } = useWidgetData(definition);
  const removeWidget = useDashboardStore((s) => s.removeWidget);
  const Component = definition.component;

  const handleRemove = useCallback(() => {
    removeWidget(widgetId);
  }, [removeWidget, widgetId]);

  return (
    <div className="h-full w-full relative group">
      <button
        type="button"
        onClick={handleRemove}
        className="absolute top-2 right-2 z-10 w-7 h-7 rounded-lg glass border border-white/5 text-muted hover:text-white hover:bg-white/10 hover:border-white/20 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-all shadow-sm"
        aria-label="Remove widget"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5">
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </button>
      <Component
        widgetId={widgetId}
        data={data}
        isLoading={isLoading}
        error={error}
        refetch={refetch}
      />
    </div>
  );
}

export const WidgetSlot = memo(WidgetSlotInner);
