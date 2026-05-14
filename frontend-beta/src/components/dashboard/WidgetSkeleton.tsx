"use client";

export function WidgetSkeleton() {
  return (
    <div className="h-full w-full rounded-xl border border-border bg-surface/50 p-5 flex flex-col gap-3 animate-pulse">
      <div className="flex items-center justify-between">
        <div className="h-3 w-24 rounded bg-surface-2" />
        <div className="h-3 w-8 rounded bg-surface-2" />
      </div>
      <div className="flex-1 flex flex-col gap-2 mt-2">
        <div className="h-2 w-full rounded bg-surface-2" />
        <div className="h-2 w-3/4 rounded bg-surface-2" />
        <div className="flex-1 rounded bg-surface-2/50 mt-2" />
      </div>
    </div>
  );
}
