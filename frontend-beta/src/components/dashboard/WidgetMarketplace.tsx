"use client";

import { useState, useMemo } from "react";
import { useDashboardStore } from "@/stores/dashboard-store";
import { widgetRegistry } from "@/lib/dashboard/registry";
import type { WidgetCategory, WidgetDefinition } from "@/lib/dashboard/types";

const CATEGORIES: { id: WidgetCategory | "all"; label: string }[] = [
  { id: "all", label: "All" },
  { id: "analytics", label: "Analytics" },
  { id: "threats", label: "Threats" },
  { id: "pipeline", label: "Pipeline" },
  { id: "intelligence", label: "Intelligence" },
  { id: "compliance", label: "Compliance" },
];

export function WidgetMarketplace() {
  const isOpen = useDashboardStore((s) => s.isMarketplaceOpen);
  const setOpen = useDashboardStore((s) => s.setMarketplaceOpen);
  const addWidget = useDashboardStore((s) => s.addWidget);
  const activeWidgets = useDashboardStore((s) => s.activeWidgets);
  const [filter, setFilter] = useState<WidgetCategory | "all">("all");
  const [search, setSearch] = useState("");

  const definitions = useMemo(() => {
    let all = widgetRegistry.getAll();
    if (filter !== "all") {
      all = all.filter((d) => d.category === filter);
    }
    if (search.trim()) {
      const q = search.toLowerCase();
      all = all.filter(
        (d) => d.name.toLowerCase().includes(q) || d.description.toLowerCase().includes(q)
      );
    }
    return all;
  }, [filter, search]);

  const activeIds = useMemo(
    () => new Set(activeWidgets.map((w) => w.definitionId)),
    [activeWidgets]
  );

  if (!isOpen) return null;

  return (
    <>
      <div
        className="fixed inset-0 bg-black/40 z-40"
        onClick={() => setOpen(false)}
        aria-hidden
      />
      <aside className="fixed right-0 top-0 bottom-0 w-full max-w-md bg-background border-l border-border z-50 flex flex-col overflow-hidden">
        <header className="px-5 py-4 border-b border-border flex items-center justify-between">
          <h2 className="text-sm font-display font-semibold text-foreground">Widget Marketplace</h2>
          <button
            type="button"
            onClick={() => setOpen(false)}
            className="w-7 h-7 rounded-md border border-border flex items-center justify-center text-muted hover:text-foreground transition-colors"
            aria-label="Close marketplace"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </header>

        <div className="px-5 py-3 border-b border-border space-y-3">
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search widgets..."
            className="w-full bg-surface-2 border border-border text-foreground text-sm rounded-lg px-3 py-2 focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent"
          />
          <div className="flex gap-1.5 flex-wrap">
            {CATEGORIES.map((cat) => (
              <button
                key={cat.id}
                type="button"
                onClick={() => setFilter(cat.id)}
                className={`text-[11px] px-2.5 py-1 rounded-md font-medium transition-colors ${
                  filter === cat.id
                    ? "bg-accent/10 text-accent border border-accent/20"
                    : "bg-surface-2 text-muted border border-border hover:text-foreground"
                }`}
              >
                {cat.label}
              </button>
            ))}
          </div>
        </div>

        <div className="flex-1 overflow-y-auto p-5 space-y-3">
          {definitions.map((def) => (
            <WidgetCard
              key={def.id}
              definition={def}
              isActive={activeIds.has(def.id)}
              onAdd={() => {
                addWidget(def);
              }}
            />
          ))}
          {definitions.length === 0 && (
            <p className="text-sm text-muted text-center py-8">No widgets match your search</p>
          )}
        </div>
      </aside>
    </>
  );
}

function WidgetCard({
  definition,
  isActive,
  onAdd,
}: {
  definition: WidgetDefinition;
  isActive: boolean;
  onAdd: () => void;
}) {
  return (
    <div className="rounded-xl border border-border bg-surface/50 p-4 flex items-start gap-3">
      <div className="w-9 h-9 rounded-lg bg-surface-2 border border-border flex items-center justify-center shrink-0">
        <definition.icon className="w-4 h-4 text-muted" />
      </div>
      <div className="flex-1 min-w-0">
        <h3 className="text-sm font-medium text-foreground">{definition.name}</h3>
        <p className="text-xs text-muted mt-0.5 line-clamp-2">{definition.description}</p>
        <div className="flex items-center gap-2 mt-2">
          <span className="text-[10px] px-2 py-0.5 rounded bg-surface-2 border border-border text-muted uppercase tracking-wider">
            {definition.category}
          </span>
          <span className="text-[10px] text-muted">
            {definition.defaultSize.w}x{definition.defaultSize.h}
          </span>
        </div>
      </div>
      <button
        type="button"
        onClick={onAdd}
        disabled={isActive}
        className={`flex-shrink-0 text-xs px-3 py-1.5 rounded-lg font-medium transition-colors ${
          isActive
            ? "bg-success/10 text-success border border-success/20 cursor-default"
            : "bg-accent text-white hover:bg-accent/90"
        }`}
      >
        {isActive ? "Added" : "Add"}
      </button>
    </div>
  );
}
