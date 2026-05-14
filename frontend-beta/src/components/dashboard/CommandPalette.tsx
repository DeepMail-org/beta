"use client";

import { useEffect } from "react";
import { Command } from "cmdk";
import { useDashboardStore } from "@/stores/dashboard-store";
import { widgetRegistry } from "@/lib/dashboard/registry";

export function CommandPalette() {
  const isOpen = useDashboardStore((s) => s.isCommandPaletteOpen);
  const setOpen = useDashboardStore((s) => s.setCommandPaletteOpen);
  const setMarketplaceOpen = useDashboardStore((s) => s.setMarketplaceOpen);
  const addWidget = useDashboardStore((s) => s.addWidget);

  useEffect(() => {
    function onKeyDown(e: KeyboardEvent) {
      if (e.key === "k" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        setOpen(!isOpen);
      }
      if (e.key === "Escape" && isOpen) {
        setOpen(false);
      }
    }
    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, [isOpen, setOpen]);

  if (!isOpen) return null;

  const allWidgets = widgetRegistry.getAll();

  return (
    <>
      <div
        className="fixed inset-0 bg-black/50 z-50"
        onClick={() => setOpen(false)}
        aria-hidden
      />
      <div className="fixed inset-x-0 top-[20%] z-50 flex justify-center px-4">
        <Command className="w-full max-w-lg rounded-xl border border-white/10 bg-black shadow-2xl overflow-hidden glass-strong">
          <Command.Input
            placeholder="Search commands..."
            className="w-full px-4 py-3 text-sm text-white bg-transparent border-b border-white/10 focus:outline-none placeholder:text-muted"
            autoFocus
          />
          <Command.List className="max-h-72 overflow-y-auto p-2">
            <Command.Empty className="py-4 text-center text-sm text-muted">
              No results found.
            </Command.Empty>

            <Command.Group heading="Actions" className="[&_[cmdk-group-heading]]:text-[10px] [&_[cmdk-group-heading]]:uppercase [&_[cmdk-group-heading]]:tracking-wider [&_[cmdk-group-heading]]:text-muted [&_[cmdk-group-heading]]:px-2 [&_[cmdk-group-heading]]:py-1.5">
              <Command.Item
                onSelect={() => { setMarketplaceOpen(true); setOpen(false); }}
                className="px-3 py-2 text-sm rounded-lg cursor-pointer text-white data-[selected]:bg-white/10 flex items-center gap-2"
              >
                <span className="text-muted">+</span> Open Widget Marketplace
              </Command.Item>
            </Command.Group>

            <Command.Group heading="Add Widget" className="[&_[cmdk-group-heading]]:text-[10px] [&_[cmdk-group-heading]]:uppercase [&_[cmdk-group-heading]]:tracking-wider [&_[cmdk-group-heading]]:text-muted [&_[cmdk-group-heading]]:px-2 [&_[cmdk-group-heading]]:py-1.5 mt-1">
              {allWidgets.map((w) => {
                const Icon = w.icon;
                return (
                  <Command.Item
                    key={w.id}
                    onSelect={() => { addWidget(w); setOpen(false); }}
                    className="px-3 py-2 text-sm rounded-lg cursor-pointer text-white data-[selected]:bg-white/10 flex items-center gap-2"
                  >
                    <Icon className="w-4 h-4 text-muted" /> {w.name}
                    <span className="ml-auto text-[10px] text-muted">{w.category}</span>
                  </Command.Item>
                );
              })}
            </Command.Group>
          </Command.List>
        </Command>
      </div>
    </>
  );
}
